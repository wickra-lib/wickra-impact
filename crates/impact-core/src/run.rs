//! The batch run: drive a strategy through the inherited `wickra-backtest`
//! signal / portfolio / metrics machinery, but replace the fill stage with the
//! order-book walk ([`crate::book_model`]).
//!
//! The backtest engine offers no fill-injection hook and its bar loop is private,
//! so — per the product handoff — this module rebuilds the bar loop over the
//! backtest crate's **public** building blocks (`rules::eval_condition`,
//! `registry::build` / `EvalIndicator`, `Portfolio`, `metrics::compute`) and does
//! its own fill. It reimplements no signal or metrics logic: the execution glue
//! (fill timing, sizing, intrabar stops, funding) is ported faithfully from the
//! engine, and a fidelity test pins that a zero-impact run equals the engine's own
//! result.
//!
//! Modelling notes (documented deviations of the walk from the engine's fixed
//! slippage): an entry is sized at the reference price (the walk price depends on
//! the sized quantity, so sizing cannot use the post-fill price); a book too thin
//! for the whole order leaves the remainder unfilled (a partial entry); an exit
//! always closes the full position at the walked price. All three collapse to the
//! engine's behaviour when the book is deep and the impact is zero.

use crate::book_model::{self, BookModel, Side};
use crate::error::{Error, Result};
use crate::latency;
use crate::spec::ImpactSpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wickra_backtest::core::metrics;
use wickra_backtest::core::registry::{self, BarInput, EvalIndicator};
use wickra_backtest::core::report::REPORT_SCHEMA_VERSION;
use wickra_backtest::core::rules::{eval_condition, BarRow, RuleState};
use wickra_backtest::core::spec::{FillTiming, OrderType, Sizing};
use wickra_backtest::{
    core::portfolio::Portfolio, BacktestReport, Candle, DerivativesTick, EquityPoint, OrderBook,
    TradePrint,
};

/// Market-impact statistics unique to this engine.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImpactStats {
    /// Volume-weighted mean per-order slippage vs. mid, in basis points.
    pub avg_slippage_bps: f64,
    /// Total notional liquidity consumed across all fills.
    pub liquidity_consumed: f64,
    /// Count of orders that could not be fully filled (book too thin / cap bound).
    pub orders_partially_filled: usize,
}

/// The impact backtest result: the inherited report plus the impact statistics.
/// Serialize-only, mirroring the inherited `BacktestReport`.
#[derive(Serialize, Clone, Debug)]
pub struct ImpactReport {
    /// The inherited backtest result, with the real order-book fill prices.
    pub report: BacktestReport,
    /// Market-impact statistics.
    pub impact_stats: ImpactStats,
}

fn default_capital() -> f64 {
    100_000.0
}

/// The batch input bundle: candles, capital and the per-bar feeds. `books` is
/// required for the `orderbook_walk` model (one L2 snapshot per bar).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunData {
    /// The candle series.
    pub candles: Vec<Candle>,
    /// Starting capital.
    #[serde(default = "default_capital")]
    pub capital: f64,
    /// One order-book snapshot per bar (required for `orderbook_walk`).
    #[serde(default)]
    pub books: Option<Vec<OrderBook>>,
    /// Reference-series candles for pairwise indicators.
    #[serde(default)]
    pub reference: Option<Vec<Candle>>,
    /// Derivatives ticks for derivatives indicators / funding.
    #[serde(default)]
    pub derivs: Option<Vec<DerivativesTick>>,
    /// Per-bar trade prints for trade-flow indicators.
    #[serde(default)]
    pub trades: Option<Vec<Vec<TradePrint>>>,
}

/// Accumulates volume-weighted slippage, liquidity and fees in trade order.
#[derive(Default)]
struct Accum {
    slippage_weighted: f64,
    weight: f64,
    liquidity: f64,
    partials: usize,
    fees: f64,
}

impl Accum {
    fn record(&mut self, slippage_bps: f64, filled: f64, notional: f64, unfilled: bool) {
        self.slippage_weighted += slippage_bps * filled;
        self.weight += filled;
        self.liquidity += notional;
        if unfilled {
            self.partials += 1;
        }
    }

    fn finish(&self) -> ImpactStats {
        let avg = if self.weight > 0.0 {
            self.slippage_weighted / self.weight
        } else {
            0.0
        };
        ImpactStats {
            avg_slippage_bps: book_model::round_to(avg, 1e-8),
            liquidity_consumed: book_model::round_to(self.liquidity, 1e-8),
            orders_partially_filled: self.partials,
        }
    }
}

/// A working order decided on a bar's close, filled on a later bar.
enum Action {
    Enter {
        long: bool,
        trigger: Option<(f64, bool)>,
    },
    Exit(&'static str),
}
struct Pending {
    action: Action,
    delay: u32,
}

/// Run an impact backtest: reconstruct the report with the real order-book fills.
///
/// # Errors
///
/// Returns [`Error::BadSpec`] if the spec is invalid, or [`Error::Data`] if a
/// required book feed is missing or a feed length does not match the candles.
#[allow(clippy::too_many_lines)]
pub fn run(data: &RunData, spec: &ImpactSpec) -> Result<ImpactReport> {
    spec.validate()?;
    let candles = &data.candles;
    if candles.is_empty() {
        return Err(Error::Data("no candles".into()));
    }
    let n = candles.len();
    let needs_book = matches!(spec.book_model, BookModel::OrderbookWalk { .. });
    let books = match &data.books {
        Some(bks) if bks.len() == n => bks.clone(),
        Some(_) => return Err(Error::Data("books length must match candles".into())),
        None if needs_book => {
            return Err(Error::Data("orderbook_walk requires a book feed".into()))
        }
        None => Vec::new(),
    };
    let deep_book_missing = books.is_empty();
    let refs = feed_check(data.reference.as_deref(), n, "reference")?;
    let derivs = feed_check(data.derivs.as_deref(), n, "derivs")?;
    let trades = data.trades.as_ref();
    if let Some(tv) = trades {
        if tv.len() != n {
            return Err(Error::Data("trades length must match candles".into()));
        }
    }

    let strat = &spec.strategy;
    let bar_ms = latency::bar_ms(&strat.timeframe)?;
    let taker = strat.costs.taker_bps / 10_000.0;
    let maker = strat.costs.maker_bps / 10_000.0;
    let close_fill = matches!(strat.execution.fill_timing, FillTiming::Close);
    let cap = spec.participation_cap;
    let model = &spec.book_model;

    // Build the indicator set (reuse the engine's registry + warmup).
    let mut indicators: BTreeMap<String, Box<dyn EvalIndicator>> = BTreeMap::new();
    let mut max_warmup = 0usize;
    for (name, ind) in &strat.indicators {
        let built = registry::build(&ind.kind, &ind.params)?;
        max_warmup = max_warmup.max(built.warmup());
        indicators.insert(name.clone(), built);
    }
    let warmup = strat.warmup.map_or(max_warmup, |w| w as usize);

    let mut pf = Portfolio::new(data.capital);
    let mut history: Vec<BarRow> = Vec::with_capacity(n);
    let mut equity: Vec<EquityPoint> = Vec::with_capacity(n);
    let mut pending: Option<Pending> = None;
    let mut entry_bar: Option<usize> = None;
    let mut extreme = 0.0f64;
    let mut acc = Accum::default();

    for t in 0..n {
        let candle = &candles[t];

        // 1. Fill any working order against this bar's book.
        if let Some(mut order) = pending.take() {
            if order.delay > 0 {
                order.delay -= 1;
                pending = Some(order);
            } else {
                let keep = fill_pending(
                    &order.action,
                    t,
                    candle,
                    &books,
                    deep_book_missing,
                    model,
                    cap,
                    spec.latency_ms,
                    bar_ms,
                    n,
                    maker,
                    taker,
                    &strat.sizing,
                    &strat.risk,
                    &history,
                    &mut pf,
                    &mut entry_bar,
                    &mut extreme,
                    &mut acc,
                );
                if keep {
                    pending = Some(order);
                }
            }
        }

        // 2. Update indicators and record the bar.
        let mut values = BTreeMap::new();
        let ref_close = refs.map(|r| r[t].close);
        let deriv = derivs.map(|d| &d[t]);
        let bar_trades: &[TradePrint] = trades.map_or(&[], |tv| tv[t].as_slice());
        let ob = book_at(&books, deep_book_missing, t);
        // Bind the core-form feeds once so the indicator inputs can borrow them.
        let deriv_core = deriv.and_then(|d| d.to_core().ok());
        let ob_core = ob.and_then(|b| b.to_core().ok());
        let trades_core: Vec<_> = bar_trades
            .iter()
            .filter_map(|tp| tp.to_core().ok())
            .collect();
        for (name, ind) in &mut indicators {
            let input = BarInput {
                candle,
                reference: ref_close,
                deriv: deriv_core,
                orderbook: ob_core.as_ref(),
                trades: &trades_core,
                cross_section: None,
            };
            if let Some(v) = ind.update(&input) {
                values.insert(name.clone(), v);
                for (field, fv) in ind.fields() {
                    values.insert(format!("{name}.{field}"), fv);
                }
            }
        }
        history.push(BarRow {
            candle: *candle,
            values,
        });
        let idx = history.len() - 1;

        // 3. Intrabar stop-loss / take-profit / trailing (at the level price).
        if pf.in_position() {
            extreme = if pf.is_long() {
                extreme.max(candle.high)
            } else {
                extreme.min(candle.low)
            };
            if let Some((price, reason)) =
                intrabar_exit(candle, &strat.risk, pf.entry_price, extreme, pf.is_long())
            {
                let fee = pf.qty.abs() * price * taker;
                pf.exit(price, candle.time, fee, reason);
                entry_bar = None;
                acc.fees += fee;
            } else if strat.risk.liquidation {
                let p_liq = -pf.cash / pf.qty;
                let breached = if pf.is_long() {
                    candle.low <= p_liq
                } else {
                    candle.high >= p_liq
                };
                if p_liq > 0.0 && breached {
                    let fee = pf.qty.abs() * p_liq * taker;
                    pf.exit(p_liq, candle.time, fee, "liquidation");
                    entry_bar = None;
                    acc.fees += fee;
                }
            }
        }

        // 3b. Perpetual funding from the derivatives feed.
        if strat.costs.funding && pf.in_position() {
            if let Some(d) = deriv.and_then(|d| d.to_core().ok()) {
                pf.apply_funding(pf.qty * d.mark_price * d.funding_rate);
            }
        }

        // 4. Mark equity at the close.
        equity.push(EquityPoint {
            time: candle.time,
            equity: pf.equity(candle.close),
        });

        // 5. Decide the next signal action (skip warmup).
        if idx < warmup {
            continue;
        }
        let state = RuleState {
            in_position: pf.in_position(),
            bars_since_entry: entry_bar.map(|e| u32::try_from(idx - e).unwrap_or(u32::MAX)),
        };
        if pf.in_position() {
            let cond = if pf.is_long() {
                &strat.exit
            } else {
                strat.short_exit.as_ref().unwrap_or(&strat.exit)
            };
            if eval_condition(cond, &history, idx, state) {
                if close_fill {
                    exit_fill(
                        "signal",
                        t,
                        candle,
                        &books,
                        deep_book_missing,
                        model,
                        cap,
                        taker,
                        &mut pf,
                        &mut entry_bar,
                        &mut acc,
                    );
                } else {
                    pending = Some(Pending {
                        action: Action::Exit("signal"),
                        delay: strat.execution.latency_bars,
                    });
                }
            }
        } else if pending.is_none() {
            let long_fires = eval_condition(&strat.entry, &history, idx, state);
            let short_fires = !long_fires
                && strat
                    .short_entry
                    .as_ref()
                    .is_some_and(|c| eval_condition(c, &history, idx, state));
            if long_fires || short_fires {
                let long = long_fires;
                if close_fill {
                    // Close-to-close: a market fill at this bar's close.
                    do_entry(
                        long,
                        candle.close,
                        false,
                        t,
                        &books,
                        deep_book_missing,
                        model,
                        cap,
                        spec.latency_ms,
                        bar_ms,
                        n,
                        maker,
                        taker,
                        &strat.sizing,
                        &strat.risk,
                        &history,
                        candle.time,
                        &mut pf,
                        &mut entry_bar,
                        &mut extreme,
                        &mut acc,
                    );
                } else {
                    let trig = entry_trigger(&strat.execution, candle.close);
                    pending = Some(Pending {
                        action: Action::Enter {
                            long,
                            trigger: trig,
                        },
                        delay: strat.execution.latency_bars,
                    });
                }
            }
        }
    }

    // Close any open position at the last close.
    if pf.in_position() {
        let last = &candles[n - 1];
        let fee = pf.qty.abs() * last.close * taker;
        pf.exit(last.close, last.time, fee, "end");
        acc.fees += fee;
    }

    let series: Vec<f64> = equity.iter().map(|e| e.equity).collect();
    let computed = metrics::compute(data.capital, &series, &pf.trades);
    let fees_paid = acc.fees;
    let report = BacktestReport {
        schema_version: REPORT_SCHEMA_VERSION,
        metrics: computed,
        trades: pf.trades.clone(),
        equity,
        fees_paid,
        initial_capital: data.capital,
    };
    Ok(ImpactReport {
        report,
        impact_stats: acc.finish(),
    })
}

fn feed_check<'a, T>(feed: Option<&'a [T]>, n: usize, name: &str) -> Result<Option<&'a [T]>> {
    match feed {
        Some(f) if f.len() == n => Ok(Some(f)),
        Some(_) => Err(Error::Data(format!("{name} length must match candles"))),
        None => Ok(None),
    }
}

fn book_at(books: &[OrderBook], missing: bool, idx: usize) -> Option<&OrderBook> {
    if missing {
        None
    } else {
        books.get(idx)
    }
}

/// Resting trigger level for an entry, or `None` for a market order.
/// The `bool` is `true` for a limit, `false` for a stop.
fn entry_trigger(
    exec: &wickra_backtest::core::spec::Execution,
    signal_close: f64,
) -> Option<(f64, bool)> {
    match exec.order_type {
        OrderType::Limit => Some((
            signal_close * (1.0 + exec.limit_offset_pct.unwrap_or(0.0) / 100.0),
            true,
        )),
        OrderType::Stop => Some((
            signal_close * (1.0 + exec.stop_offset_pct.unwrap_or(0.0) / 100.0),
            false,
        )),
        _ => None,
    }
}

/// Fill a resting level order against a bar, or `None` if not reached.
fn level_fill(long: bool, trigger: f64, is_limit: bool, c: &Candle) -> Option<f64> {
    match (long, is_limit) {
        (true, true) => (c.low <= trigger).then(|| c.open.min(trigger)),
        (true, false) => (c.high >= trigger).then(|| c.open.max(trigger)),
        (false, true) => (c.high >= trigger).then(|| c.open.max(trigger)),
        (false, false) => (c.low <= trigger).then(|| c.open.min(trigger)),
    }
}

#[allow(clippy::cast_precision_loss)]
fn realized_vol(history: &[BarRow], lookback: usize) -> Option<f64> {
    if lookback < 2 || history.len() < lookback {
        return None;
    }
    let closes: Vec<f64> = history[history.len() - lookback..]
        .iter()
        .map(|row| row.candle.close)
        .collect();
    let rets: Vec<f64> = closes
        .windows(2)
        .filter(|w| w[0].abs() > f64::EPSILON)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();
    if rets.is_empty() {
        return None;
    }
    let mean = rets.iter().sum::<f64>() / rets.len() as f64;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rets.len() as f64;
    let sd = var.sqrt();
    (sd > 0.0).then_some(sd)
}

fn size(
    sizing: &Sizing,
    risk: &wickra_backtest::core::spec::Risk,
    equity: f64,
    price: f64,
    rv: Option<f64>,
) -> Option<f64> {
    if price <= 0.0 || equity <= 0.0 {
        return None;
    }
    let qty = match *sizing {
        Sizing::FixedFraction { fraction } => (equity * fraction) / price,
        Sizing::FixedCash { cash } => cash / price,
        Sizing::FixedQty { qty } => qty,
        Sizing::RiskPerTrade { risk_pct } => {
            let stop = risk.stop_loss_pct?;
            if stop <= 0.0 {
                return None;
            }
            (equity * risk_pct / 100.0) / (price * stop / 100.0)
        }
        Sizing::VolTarget { target_vol, .. } => {
            let rv = rv?;
            (equity * target_vol / rv) / price
        }
    };
    if qty <= 0.0 {
        return None;
    }
    let max_leverage = risk.max_leverage.unwrap_or(1.0);
    let mut max_notional = equity * max_leverage;
    if let Some(max_pct) = risk.max_position_pct {
        max_notional = max_notional.min(equity * max_pct / 100.0);
    }
    Some((qty * price).min(max_notional) / price)
}

fn intrabar_exit(
    candle: &Candle,
    risk: &wickra_backtest::core::spec::Risk,
    entry: f64,
    extreme: f64,
    is_long: bool,
) -> Option<(f64, &'static str)> {
    if entry <= 0.0 {
        return None;
    }
    if is_long {
        if let Some(p) = risk.stop_loss_pct {
            let level = entry * (1.0 - p / 100.0);
            if candle.low <= level {
                return Some((level.min(candle.open), "stop_loss"));
            }
        }
        if let Some(p) = risk.trailing_stop_pct {
            let level = extreme * (1.0 - p / 100.0);
            if candle.low <= level {
                return Some((level.min(candle.open), "trailing_stop"));
            }
        }
        if let Some(p) = risk.take_profit_pct {
            let level = entry * (1.0 + p / 100.0);
            if candle.high >= level {
                return Some((level.max(candle.open), "take_profit"));
            }
        }
    } else {
        if let Some(p) = risk.stop_loss_pct {
            let level = entry * (1.0 + p / 100.0);
            if candle.high >= level {
                return Some((level.max(candle.open), "stop_loss"));
            }
        }
        if let Some(p) = risk.trailing_stop_pct {
            let level = extreme * (1.0 + p / 100.0);
            if candle.high >= level {
                return Some((level.max(candle.open), "trailing_stop"));
            }
        }
        if let Some(p) = risk.take_profit_pct {
            let level = entry * (1.0 - p / 100.0);
            if candle.low <= level {
                return Some((level.min(candle.open), "take_profit"));
            }
        }
    }
    None
}

/// The bar index whose book a fill lands on given the latency shift, or `None` if
/// it runs off the end of the recorded history (the order is cancelled).
fn book_index(t: usize, latency_ms: u64, bar_ms: u64, n: usize) -> Option<usize> {
    let extra = if bar_ms == 0 {
        0
    } else {
        usize::try_from(latency_ms.div_ceil(bar_ms)).unwrap_or(usize::MAX)
    };
    let idx = t.saturating_add(extra);
    (idx < n).then_some(idx)
}

/// A synthetic deep book used when no book feed is present (analytic models):
/// falls back to the reference price with no walk effect.
fn empty_book() -> OrderBook {
    OrderBook {
        bids: Vec::new(),
        asks: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_pending(
    action: &Action,
    t: usize,
    candle: &Candle,
    books: &[OrderBook],
    missing: bool,
    model: &BookModel,
    cap: f64,
    latency_ms: u64,
    bar_ms: u64,
    n: usize,
    maker: f64,
    taker: f64,
    sizing: &Sizing,
    risk: &wickra_backtest::core::spec::Risk,
    history: &[BarRow],
    pf: &mut Portfolio,
    entry_bar: &mut Option<usize>,
    extreme: &mut f64,
    acc: &mut Accum,
) -> bool {
    match action {
        Action::Enter { long, trigger } => {
            let is_maker = trigger.is_some_and(|(_, is_limit)| is_limit);
            let raw = match trigger {
                None => Some(candle.open),
                Some((trig, is_limit)) => level_fill(*long, *trig, *is_limit, candle),
            };
            match raw {
                Some(px) => {
                    do_entry(
                        *long,
                        px,
                        is_maker,
                        t,
                        books,
                        missing,
                        model,
                        cap,
                        latency_ms,
                        bar_ms,
                        n,
                        maker,
                        taker,
                        sizing,
                        risk,
                        history,
                        candle.time,
                        pf,
                        entry_bar,
                        extreme,
                        acc,
                    );
                    false
                }
                None => true, // level not reached; keep working
            }
        }
        Action::Exit(reason) => {
            exit_fill_at(
                reason,
                candle.open,
                candle.time,
                t,
                books,
                missing,
                model,
                cap,
                latency_ms,
                bar_ms,
                n,
                taker,
                pf,
                entry_bar,
                acc,
            );
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn do_entry(
    long: bool,
    raw_price: f64,
    is_maker: bool,
    t: usize,
    books: &[OrderBook],
    missing: bool,
    model: &BookModel,
    cap: f64,
    latency_ms: u64,
    bar_ms: u64,
    n: usize,
    maker: f64,
    taker: f64,
    sizing: &Sizing,
    risk: &wickra_backtest::core::spec::Risk,
    history: &[BarRow],
    time: i64,
    pf: &mut Portfolio,
    entry_bar: &mut Option<usize>,
    extreme: &mut f64,
    acc: &mut Accum,
) {
    let Some(idx) = book_index(t, latency_ms, bar_ms, n) else {
        return; // order cancelled (ran off the end)
    };
    let rv = match *sizing {
        Sizing::VolTarget { lookback, .. } => realized_vol(history, lookback as usize),
        _ => None,
    };
    let Some(base) = size(sizing, risk, pf.cash, raw_price, rv) else {
        return;
    };
    let side = if long { Side::Buy } else { Side::Sell };
    let fallback = empty_book();
    let book = book_at(books, missing, idx).unwrap_or(&fallback);
    let fill = book_model::fill(side, base, raw_price, book, model, cap);
    if fill.qty_filled <= 0.0 {
        return;
    }
    let dir = if long { 1.0 } else { -1.0 };
    let rate = if is_maker { maker } else { taker };
    let fee = fill.qty_filled * fill.price * rate;
    pf.enter(dir * fill.qty_filled, fill.price, time, fee);
    *entry_bar = Some(t);
    *extreme = fill.price;
    acc.fees += fee;
    acc.record(
        fill.slippage_bps,
        fill.qty_filled,
        fill.notional,
        fill.qty_unfilled > 0.0,
    );
}

#[allow(clippy::too_many_arguments)]
fn exit_fill_at(
    reason: &'static str,
    raw_price: f64,
    time: i64,
    t: usize,
    books: &[OrderBook],
    missing: bool,
    model: &BookModel,
    cap: f64,
    latency_ms: u64,
    bar_ms: u64,
    n: usize,
    taker: f64,
    pf: &mut Portfolio,
    entry_bar: &mut Option<usize>,
    acc: &mut Accum,
) {
    if !pf.in_position() {
        return;
    }
    let book_opt =
        book_index(t, latency_ms, bar_ms, n).and_then(|idx| book_at(books, missing, idx));
    let fallback = empty_book();
    let book = book_opt.unwrap_or(&fallback);
    // Long exit sells (hits bids); short exit buys (lifts asks).
    let side = if pf.is_long() { Side::Sell } else { Side::Buy };
    let qty = pf.qty.abs();
    let fill = book_model::fill(side, qty, raw_price, book, model, cap);
    let price = if fill.qty_filled > 0.0 {
        fill.price
    } else {
        raw_price
    };
    let fee = qty * price * taker;
    pf.exit(price, time, fee, reason);
    *entry_bar = None;
    acc.fees += fee;
    acc.record(fill.slippage_bps, qty, price * qty, false);
}

#[allow(clippy::too_many_arguments)]
fn exit_fill(
    reason: &'static str,
    t: usize,
    candle: &Candle,
    books: &[OrderBook],
    missing: bool,
    model: &BookModel,
    cap: f64,
    taker: f64,
    pf: &mut Portfolio,
    entry_bar: &mut Option<usize>,
    acc: &mut Accum,
) {
    if !pf.in_position() {
        return;
    }
    let book = book_at(books, missing, t);
    let fallback = empty_book();
    let book = book.unwrap_or(&fallback);
    let side = if pf.is_long() { Side::Sell } else { Side::Buy };
    let qty = pf.qty.abs();
    let fill = book_model::fill(side, qty, candle.close, book, model, cap);
    let price = if fill.qty_filled > 0.0 {
        fill.price
    } else {
        candle.close
    };
    let fee = qty * price * taker;
    pf.exit(price, candle.time, fee, reason);
    *entry_bar = None;
    acc.fees += fee;
    acc.record(fill.slippage_bps, qty, price * qty, false);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, clippy::many_single_char_names)]
    use super::{run, RunData};
    use crate::spec::ImpactSpec;
    use wickra_backtest::core::data::{Level, OrderBook};
    use wickra_backtest::Candle;

    fn candle(time: i64, o: f64, h: f64, l: f64, c: f64, v: f64) -> Candle {
        Candle {
            time,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
        }
    }

    fn book(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> OrderBook {
        let lv = |s: &[(f64, f64)]| {
            s.iter()
                .map(|&(price, size)| Level { price, size })
                .collect()
        };
        OrderBook {
            bids: lv(bids),
            asks: lv(asks),
        }
    }

    #[test]
    fn thin_book_worked_example_shows_44_bps() {
        let spec: ImpactSpec = serde_json::from_str(
            r#"{
              "strategy": {
                "spec_version": 1, "symbol": "IMPACT", "timeframe": "1h",
                "indicators": {},
                "entry": {"ge": [{"price": "close"}, 0]},
                "exit": {"in_position": true},
                "sizing": {"type": "fixed_qty", "qty": 10.0},
                "execution": {"order_type": "market", "fill_timing": "next_open"}
              },
              "book_model": {"kind": "orderbook_walk"},
              "participation_cap": 1.0, "latency_ms": 0
            }"#,
        )
        .unwrap();
        let data = RunData {
            candles: vec![
                candle(0, 100.0, 100.0, 100.0, 100.0, 1000.0),
                candle(3600, 100.0, 103.0, 100.0, 102.0, 1000.0),
            ],
            capital: 100_000.0,
            books: Some(vec![
                book(&[(99.9, 100.0)], &[(100.1, 100.0)]),
                book(
                    &[(99.9, 100.0)],
                    &[(100.1, 3.0), (100.3, 3.0), (100.8, 4.0)],
                ),
            ]),
            reference: None,
            derivs: None,
            trades: None,
        };
        let out = run(&data, &spec).unwrap();
        assert_eq!(out.impact_stats.avg_slippage_bps, 44.0);
        assert_eq!(out.impact_stats.liquidity_consumed, 1004.4);
        assert_eq!(out.impact_stats.orders_partially_filled, 0);
        // The entry filled at the walked VWAP, not the naive next-open of 100.0.
        assert_eq!(out.report.trades[0].entry_price, 100.44);
    }

    #[test]
    fn zero_impact_matches_the_inherited_engine() {
        // A deep book + LinearImpact{coef:0} must reproduce the engine's own
        // result with zero slippage — the fidelity anchor for the rebuilt loop.
        let strat_json = r#"{
            "spec_version": 1, "symbol": "IMPACT", "timeframe": "1h",
            "indicators": {},
            "entry": {"ge": [{"price": "close"}, 0]},
            "exit": {"in_position": true},
            "sizing": {"type": "fixed_qty", "qty": 5.0},
            "costs": {"taker_bps": 5.0},
            "execution": {"order_type": "market", "fill_timing": "next_open"}
        }"#;
        let candles = vec![
            candle(0, 100.0, 101.0, 99.0, 100.5, 1000.0),
            candle(3600, 100.5, 102.0, 100.0, 101.5, 1000.0),
            candle(7200, 101.5, 103.0, 101.0, 102.5, 1000.0),
        ];
        let deep = book(&[(99.0, 1e6)], &[(100.0, 1e6)]);
        let books: Vec<OrderBook> = candles.iter().map(|_| deep.clone()).collect();

        let impact_spec: ImpactSpec = serde_json::from_str(&format!(
            r#"{{"strategy": {strat_json}, "book_model": {{"kind": "linear_impact", "coef": 0.0}}, "participation_cap": 1.0, "latency_ms": 0}}"#
        ))
        .unwrap();
        let data = RunData {
            candles: candles.clone(),
            capital: 100_000.0,
            books: Some(books),
            reference: None,
            derivs: None,
            trades: None,
        };
        let mine = run(&data, &impact_spec).unwrap();

        let strat: wickra_backtest::StrategySpec = serde_json::from_str(strat_json).unwrap();
        let engine = wickra_backtest::run_with_capital(&strat, &candles, 100_000.0).unwrap();

        assert_eq!(mine.report.trades.len(), engine.trades.len());
        for (a, b) in mine.report.trades.iter().zip(&engine.trades) {
            assert!((a.entry_price - b.entry_price).abs() < 1e-9);
            assert!((a.exit_price - b.exit_price).abs() < 1e-9);
            assert!((a.pnl - b.pnl).abs() < 1e-6);
        }
        assert!((mine.report.metrics.pnl - engine.metrics.pnl).abs() < 1e-6);
        // With coef 0 the fill price equals the engine's (asserted above); the
        // impact stat measures execution vs. book mid, which is non-zero here only
        // because the synthetic book's mid differs from the fill reference.
    }
}
