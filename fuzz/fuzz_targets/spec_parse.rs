#![no_main]
//! Fuzz the spec-parsing surface: arbitrary bytes are parsed as an `ImpactSpec`
//! (JSON). Malformed input must surface as a clean `Err`, never a panic. A
//! successfully parsed spec re-serializes and re-parses to an equal value.

use impact_core::ImpactSpec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(spec) = ImpactSpec::from_json(text) else {
        return;
    };
    let serialized = serde_json::to_string(&spec).expect("serialize a parsed spec");
    let reparsed: ImpactSpec =
        serde_json::from_str(&serialized).expect("re-parse a serialized spec");
    assert_eq!(reparsed, spec, "ImpactSpec serde round-trip is not stable");
});
