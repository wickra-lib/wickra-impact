//! Node.js bindings for `wickra-impact` via napi-rs.
//!
//! A `Impact` is built from a spec JSON; `command` takes a request JSON and
//! returns the response JSON, so Node drives the exact same byte-identical
//! surface — and gets the byte-identical report — as every other binding.

use napi_derive::napi;

/// A market-impact backtest driven by JSON commands.
#[napi]
pub struct Impact(impact_core::Impact);

#[napi]
impl Impact {
    /// Construct a backtest handle from a spec JSON (`"{}"` defers configuration
    /// to a later `set_spec` command). Throws on an invalid spec.
    #[napi(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(spec_json: String) -> napi::Result<Self> {
        impact_core::Impact::new(&spec_json)
            .map(Impact)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Apply a command envelope (`{"cmd":"...", ...}`) and return the response
    /// JSON. Commands: `set_spec`, `run`, `version`.
    #[napi]
    #[allow(clippy::needless_pass_by_value)]
    pub fn command(&mut self, cmd_json: String) -> napi::Result<String> {
        self.0
            .command_json(&cmd_json)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// The crate version.
    #[napi]
    pub fn version(&self) -> &'static str {
        impact_core::version()
    }
}
