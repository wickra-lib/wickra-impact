# Fuzzing Wickra Impact

[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) harnesses for the parsing and stateful entry points of Wickra Impact. Fuzzing requires a nightly Rust toolchain; CI runs every target for 30 seconds on the family's pinned `nightly-2026-07-01`.

## Setup

```bash
cargo install cargo-fuzz
rustup toolchain install nightly-2026-07-01
```

The date is the family's fuzz nightly, pinned in `ci.yml`: a rolling `nightly`
regressed with a codegen ICE unrelated to this code, so every repository moves
the date together, on purpose.

## Targets

| Target | What it exercises |
| --- | --- |
| `spec_parse` | The spec-parsing surface: arbitrary bytes are parsed as an `ImpactSpec` (JSON). |
| `book_walk` | The fill engine: a JSON-encoded order (quantity, reference, cap) and a book (bid/ask ladders) drive `book_model::fill`. |
| `run_batch` | A full batch run through the JSON command surface: a parsed spec (with the participation cap clamped into range) drives `run` over a fixed tiny candle + book universe. |
| `latency_select` | The latency / timeframe arithmetic: an arbitrary timeframe string is parsed to milliseconds (malformed input is a clean `Err`, never a panic), and the snapshot-index selection is exercised over bounded parameters. |

## Run

```bash
# From the repository root:
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu book_walk
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu run_batch
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu latency_select
```

Each run continues until a crash is found or it is interrupted. A short
time-boxed smoke run is what CI does:

```bash
cargo +nightly-2026-07-01 fuzz run --target x86_64-unknown-linux-gnu spec_parse -- -max_total_time=30
```

The expectation for every target is that it never panics: malformed or
adversarial input must surface as an `Err` or an in-band error, never a crash.
