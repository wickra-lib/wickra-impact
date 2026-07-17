//! CLI-side configuration: a thin wrapper that loads an [`ImpactSpec`] from a
//! file.

use crate::error::Result;
use crate::spec::ImpactSpec;

/// A loaded configuration: the impact spec to run.
pub struct Config {
    /// The impact spec.
    pub spec: ImpactSpec,
}

impl Config {
    /// Load a config from an `ImpactSpec` JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Parse`] if the JSON is malformed.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(Self {
            spec: ImpactSpec::from_json(s)?,
        })
    }
}
