//! Wickra Impact — node binding. Scaffold; the real surface lands in P-IMP-3.

/// The crate version, forwarded from the core.
#[must_use]
pub fn version() -> &'static str {
    impact_core::version()
}
