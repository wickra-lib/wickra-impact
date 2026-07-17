#![allow(clippy::cast_precision_loss)]
//! Criterion benchmarks for `impact_core::run`.
//!
//! The default build measures the parallel-capable engine; `--no-default-features`
//! measures the single-threaded path (what WASM and the golden fixtures use).
//! Each case runs a buy-and-hold strategy over `N` bars, filling every signalled
//! order against a per-bar L2 book, for three book models. Throughput is reported
//! in bars/second.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use impact_core::{run, BookModel, ImpactSpec, RunData};
use wickra_backtest::core::data::{Level, OrderBook};
use wickra_backtest::Candle;

fn candles(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let t = i as f64;
            let close = 100.0 + 15.0 * (t / 9.0).sin() + 0.01 * t;
            let open = 100.0 + 15.0 * ((t - 1.0) / 9.0).sin() + 0.01 * (t - 1.0);
            Candle {
                time: 1_700_000_000 + i64::try_from(i).unwrap() * 3600,
                open,
                high: close.max(open) + 1.0,
                low: close.min(open) - 1.0,
                close,
                volume: 1000.0,
            }
        })
        .collect()
}

/// A five-level ladder either side of the bar close — deep enough to fill the
/// small fixed order without exhausting the book.
fn books(candles: &[Candle]) -> Vec<OrderBook> {
    candles
        .iter()
        .map(|c| {
            let mk = |sign: f64| {
                (1..=5)
                    .map(|l| Level {
                        price: c.close + sign * 0.1 * f64::from(l),
                        size: 50.0,
                    })
                    .collect::<Vec<_>>()
            };
            OrderBook {
                bids: mk(-1.0),
                asks: mk(1.0),
            }
        })
        .collect()
}

fn spec(model: BookModel) -> ImpactSpec {
    let json = r#"{
        "strategy": {
            "spec_version": 1, "symbol": "IMPACT", "timeframe": "1h",
            "indicators": {},
            "entry": {"ge": [{"price": "close"}, 0]},
            "exit": {"lt": [{"price": "close"}, 0]},
            "sizing": {"type": "fixed_qty", "qty": 10.0},
            "execution": {"order_type": "market", "fill_timing": "next_open"}
        },
        "book_model": {"kind": "orderbook_walk"},
        "participation_cap": 1.0, "latency_ms": 0
    }"#;
    let mut s: ImpactSpec = serde_json::from_str(json).unwrap();
    s.book_model = model;
    s
}

fn bench_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("impact_run");
    for &n in &[1_000usize, 10_000, 100_000] {
        let candles = candles(n);
        let data = RunData {
            candles: candles.clone(),
            capital: 100_000.0,
            books: Some(books(&candles)),
            reference: None,
            derivs: None,
            trades: None,
        };
        let models = [
            ("orderbook_walk", BookModel::OrderbookWalk { levels: None }),
            ("linear_impact", BookModel::LinearImpact { coef: 0.1 }),
            ("square_root", BookModel::SquareRoot { coef: 0.5 }),
        ];
        group.throughput(Throughput::Elements(n as u64));
        for (name, model) in models {
            let s = spec(model);
            group.bench_with_input(BenchmarkId::new(name, n), &n, |b, _| {
                b.iter(|| run(&data, &s).unwrap());
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_run);
criterion_main!(benches);
