//! The `Impact` handle and the `command_json` boundary — the single entry point
//! every language binding forwards verbatim.

use crate::error::{Error, Result};
use crate::run::{run, ImpactReport, RunData};
use crate::spec::ImpactSpec;
use serde_json::Value;

/// A market-impact backtest handle: holds an optional spec and dispatches
/// commands over the JSON boundary.
pub struct Impact {
    spec: Option<ImpactSpec>,
}

impl Impact {
    /// Build a handle from an [`ImpactSpec`] JSON string. An empty object (`"{}"`)
    /// or an empty string defers configuration to a later `set_spec`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Parse`] if the JSON is present but malformed.
    pub fn new(spec_json: &str) -> Result<Self> {
        let trimmed = spec_json.trim();
        if trimmed.is_empty() || trimmed == "{}" {
            return Ok(Self { spec: None });
        }
        let spec = ImpactSpec::from_json(spec_json)?;
        Ok(Self { spec: Some(spec) })
    }

    /// Replace the held spec.
    pub fn set_spec(&mut self, spec: ImpactSpec) {
        self.spec = Some(spec);
    }

    /// Run the held spec over `data`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadSpec`] if no spec is set, or any run error.
    pub fn run(&self, data: &RunData) -> Result<ImpactReport> {
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| Error::BadSpec("no spec set".into()))?;
        run(data, spec)
    }

    /// The crate version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Dispatch one command. Always returns a JSON string; internal errors are
    /// reported as `{"ok":false,"error":"..."}` rather than as an `Err`.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] only if a valid response cannot be serialised (never in
    /// normal operation).
    pub fn command_json(&mut self, cmd_json: &str) -> Result<String> {
        match self.dispatch(cmd_json) {
            Ok(s) => Ok(s),
            Err(e) => Ok(error_json(&e.to_string())),
        }
    }

    fn dispatch(&mut self, cmd_json: &str) -> Result<String> {
        let v: Value = serde_json::from_str(cmd_json)?;
        let cmd = v
            .get("cmd")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Parse("missing cmd".into()))?;
        match cmd {
            "set_spec" => {
                let spec: ImpactSpec = serde_json::from_value(
                    v.get("spec")
                        .cloned()
                        .ok_or_else(|| Error::Parse("missing spec".into()))?,
                )?;
                self.spec = Some(spec);
                Ok(r#"{"ok":true}"#.to_string())
            }
            "run" => {
                let data: RunData = serde_json::from_value(
                    v.get("data")
                        .cloned()
                        .ok_or_else(|| Error::Parse("missing data".into()))?,
                )?;
                let report = self.run(&data)?;
                Ok(serde_json::to_string(&report)?)
            }
            "version" => Ok(format!(r#"{{"version":"{}"}}"#, Self::version())),
            other => Err(Error::BadSpec(format!("unknown cmd: {other}"))),
        }
    }
}

fn error_json(msg: &str) -> String {
    format!(
        r#"{{"ok":false,"error":{}}}"#,
        serde_json::to_string(msg).unwrap_or_else(|_| "\"error\"".to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::Impact;

    const SPEC: &str = r#"{
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
    }"#;

    #[test]
    fn version_command() {
        let mut h = Impact::new(SPEC).unwrap();
        assert!(h
            .command_json(r#"{"cmd":"version"}"#)
            .unwrap()
            .contains("0.1"));
    }

    #[test]
    fn unknown_command_is_error_json() {
        let mut h = Impact::new(SPEC).unwrap();
        let out = h.command_json(r#"{"cmd":"nope"}"#).unwrap();
        assert!(out.contains(r#""ok":false"#));
    }

    #[test]
    fn run_command_returns_a_report() {
        let mut h = Impact::new(SPEC).unwrap();
        let cmd = r#"{"cmd":"run","data":{"candles":[
            {"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},
            {"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],
          "books":[
            {"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},
            {"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":3},{"price":100.3,"size":3},{"price":100.8,"size":4}]}]}}"#;
        let out = h.command_json(cmd).unwrap();
        assert!(out.contains(r#""avg_slippage_bps":44"#));
    }
}
