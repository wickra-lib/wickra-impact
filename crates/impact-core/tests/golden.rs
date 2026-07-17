//! The golden invariant, from Rust: every `golden/specs/*.json` paired with its
//! same-named `golden/data/*.json` regenerates `golden/expected/*.json`
//! byte-for-byte. `serde_json::to_string(&run(..))` is exactly what the CLI's
//! `--format json` and every language binding produce, so this file is the
//! reference the whole cross-language corpus is pinned to.
//!
//! A missing expected file is written (bless mode) so the corpus can be
//! regenerated after an intended change; a present file is asserted byte-for-byte.

use std::fs;
use std::path::PathBuf;

use impact_core::{run, ImpactSpec, RunData};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

#[test]
fn every_spec_matches_its_expected_output() {
    let dir = golden_dir();

    let mut specs: Vec<_> = fs::read_dir(dir.join("specs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    specs.sort();

    let mut checked = 0;
    for spec_path in specs {
        let stem = spec_path.file_stem().unwrap().to_str().unwrap().to_owned();
        let spec: ImpactSpec = serde_json::from_str(&fs::read_to_string(&spec_path).unwrap())
            .unwrap_or_else(|e| panic!("parse spec {stem}: {e}"));
        let data_path = dir.join("data").join(format!("{stem}.json"));
        let data: RunData = serde_json::from_str(&fs::read_to_string(&data_path).unwrap())
            .unwrap_or_else(|e| panic!("parse data {stem}: {e}"));

        let report = run(&data, &spec).unwrap_or_else(|e| panic!("run {stem}: {e}"));
        let got = serde_json::to_string(&report).unwrap();

        let expected_path = dir.join("expected").join(format!("{stem}.json"));
        if expected_path.exists() {
            let expected = fs::read_to_string(&expected_path).unwrap();
            assert_eq!(
                got,
                expected.trim_end(),
                "golden mismatch for {stem}: the core output no longer matches the \
                 committed fixture (re-bless if the change is intended)"
            );
        } else {
            fs::write(&expected_path, format!("{got}\n")).unwrap();
        }
        checked += 1;
    }
    assert_eq!(checked, 6, "expected 6 golden specs, checked {checked}");
}
