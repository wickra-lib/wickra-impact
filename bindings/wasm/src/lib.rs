//! WebAssembly bindings for `wickra-impact` (wasm-bindgen).
//!
//! Measure market impact in the browser: create an `Impact` from a spec JSON,
//! drive it with a command JSON (`set_spec`, `run`, `version`) and read back the
//! response JSON. The same command protocol crosses every binding, so a browser
//! front-end runs against the exact same core as the native CLI.
//!
//! The fill engine runs single-threaded here (no rayon thread pool in a browser
//! sandbox), which is byte-identical to the native run — the exact
//! cross-language golden check.

use wasm_bindgen::prelude::*;

use impact_core::Impact as CoreImpact;

/// A market-impact backtest driven by JSON commands.
#[wasm_bindgen]
pub struct Impact {
    inner: CoreImpact,
}

#[wasm_bindgen]
impl Impact {
    /// Construct a backtest handle from a spec JSON (`"{}"` defers configuration
    /// to a later `set_spec` command).
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Impact, JsError> {
        CoreImpact::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Apply a command JSON (`{"cmd":"...", ...}`) and return the response JSON.
    pub fn command(&mut self, cmd_json: &str) -> Result<String, JsError> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// The library version.
    #[wasm_bindgen(js_name = version)]
    pub fn instance_version(&self) -> String {
        impact_core::version().to_string()
    }
}

/// The library version.
#[wasm_bindgen]
pub fn version() -> String {
    impact_core::version().to_string()
}
