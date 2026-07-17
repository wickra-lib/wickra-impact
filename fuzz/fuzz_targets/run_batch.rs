#![no_main]
//! Fuzz a full batch run through the JSON command surface: a parsed spec (with
//! the participation cap clamped into range) drives `run` over a fixed tiny
//! candle + book universe. No input may panic; a domain error comes back in-band
//! as JSON.

use impact_core::{Impact, ImpactSpec};
use libfuzzer_sys::fuzz_target;

// A fixed two-bar universe so the fuzzer varies the spec, not the data.
const RUN_CMD: &str = r#"{"cmd":"run","data":{
  "candles":[
    {"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},
    {"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],
  "books":[
    {"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},
    {"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":3},{"price":100.3,"size":3},{"price":100.8,"size":4}]}]
}}"#;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(mut spec) = ImpactSpec::from_json(text) else {
        return;
    };
    // Keep the participation cap in its valid open interval so validation can pass.
    if !spec.participation_cap.is_finite() || spec.participation_cap <= 0.0 {
        spec.participation_cap = 1.0;
    }
    spec.participation_cap = spec.participation_cap.min(1.0);
    let Ok(spec_json) = serde_json::to_string(&spec) else {
        return;
    };
    let Ok(mut impact) = Impact::new(&spec_json) else {
        return;
    };
    // A command failure is returned as a Result; a panic would be a bug.
    let _ = impact.command_json(RUN_CMD);
});
