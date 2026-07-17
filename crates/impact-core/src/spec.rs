//! The `ImpactSpec` envelope: an embedded strategy plus the fill model.

use crate::book_model::BookModel;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use wickra_backtest::core::spec::Slippage;
use wickra_backtest::StrategySpec;

fn default_participation_cap() -> f64 {
    1.0
}

/// What a consumer writes: an embedded `wickra-backtest` strategy, the fill /
/// market-impact model, and the participation and latency controls.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImpactSpec {
    /// The embedded backtest strategy spec (data, not code).
    pub strategy: StrategySpec,
    /// The fill / market-impact model.
    pub book_model: BookModel,
    /// Max fraction of the bar's available book liquidity a single order may
    /// consume, in `(0, 1]`. The remainder is left unfilled (a partial).
    #[serde(default = "default_participation_cap")]
    pub participation_cap: f64,
    /// Simulated latency in milliseconds (shifts which book snapshot a signalled
    /// order fills against).
    #[serde(default)]
    pub latency_ms: u64,
}

impl ImpactSpec {
    /// Parse an [`ImpactSpec`] from JSON.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Parse`] if the JSON is malformed.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Validate the spec: the embedded strategy, the participation cap, the impact
    /// coefficient and that no conflicting slippage is set.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadSpec`] on any invalid field.
    pub fn validate(&self) -> Result<()> {
        self.strategy
            .validate()
            .map_err(|e| Error::BadSpec(e.to_string()))?;
        if !(self.participation_cap > 0.0 && self.participation_cap <= 1.0) {
            return Err(Error::BadSpec("participation_cap must be in (0, 1]".into()));
        }
        match self.book_model {
            BookModel::LinearImpact { coef } | BookModel::SquareRoot { coef } => {
                if !coef.is_finite() || coef < 0.0 {
                    return Err(Error::BadSpec("impact coef must be finite and >= 0".into()));
                }
            }
            BookModel::OrderbookWalk { .. } => {}
        }
        // IMPACT owns the fill stage; a slippage set on the embedded strategy would
        // double-count, so reject it.
        if self.strategy.costs.slippage != (Slippage::FixedBps { bps: 0.0 }) {
            return Err(Error::BadSpec(
                "impact runs override slippage; remove costs.slippage".into(),
            ));
        }
        Ok(())
    }
}
