//! Python bindings for `wickra-impact`, exposed under the `wickra_impact`
//! package.
//!
//! Thin glue over the impact core's command surface: construct a
//! [`Impact`] from a spec JSON, drive it with a command JSON and read back the
//! response JSON. The same command protocol crosses every binding, so a Python
//! front-end drives the exact same core — and gets the byte-identical report —
//! as the CLI.

// PyO3 protocol methods take `self` by ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use impact_core::Impact;

/// A market-impact backtest driven by JSON commands.
///
/// `unsendable`: the handle caches the last report, so it is bound to the thread
/// that created it.
#[pyclass(name = "Impact", unsendable)]
struct PyImpact {
    inner: Impact,
}

#[pymethods]
impl PyImpact {
    /// Construct a backtest handle from a spec JSON (`"{}"` defers configuration
    /// to a later `set_spec` command).
    #[new]
    fn new(spec_json: &str) -> PyResult<Self> {
        Impact::new(spec_json)
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Apply a command JSON and return the response JSON.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        self.inner
            .command_json(cmd_json)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        impact_core::version()
    }
}

/// The native module (`wickra_impact._wickra_impact`).
#[pymodule]
fn _wickra_impact(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyImpact>()?;
    Ok(())
}
