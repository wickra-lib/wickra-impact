# Architecture

Wickra Impact is a thin, data-driven layer over the `wickra-backtest` engine. It
changes one thing — how orders fill — and inherits everything else.

## Crates

| Crate           | Role |
|-----------------|------|
| `impact-core`   | The library: the `BookModel` fill engine, the `ImpactSpec` envelope, latency mapping, the `run` loop, and the `Impact` JSON-command handle. |
| `impact-cli`    | `wickra-impact`, the reference CLI (`--request` / `--spec`+`--data` / `--stdin`, `--format text\|json`). |
| `impact-bench`  | Criterion benchmarks for `run`. |
| `bindings/*`    | The ten language bindings (Python, Node.js, WASM native; C, C++, C#, Go, Java, R over the C ABI hub). |

## The inheritance boundary

`impact-core` depends on `wickra-backtest` (a git dependency) and reuses its
**public** building blocks directly: `rules::eval_condition` for signals,
`registry::build` / `EvalIndicator` for indicators, `Portfolio` for position and
cash accounting, and `metrics::compute` for the result metrics. The engine offers
no fill-injection hook and its bar loop is private, so `run` rebuilds the bar loop
over those public parts and substitutes its own fill. It reimplements no signal or
metrics logic — only the execution glue (fill timing, sizing, intrabar stops,
funding) is ported faithfully, and a fidelity test pins that a zero-impact run
reproduces the engine's own `BacktestReport`. See [INHERITANCE](INHERITANCE.md).

## The command surface

The whole library is reachable as data over a JSON boundary. `Impact::command_json`
takes a command envelope (`{"cmd":"set_spec"|"run"|"version", ...}`) and returns a
response JSON string. The C ABI (`wickra_impact_new` / `_command` / `_free` /
`_version`) exposes that surface with a two-call length protocol; every other
binding wraps the C ABI or calls the core directly, forwarding the command string
verbatim. Because the fill engine lives once in the Rust core and every output is
rounded onto a fixed grid, the `ImpactReport` is **byte-identical** across the CLI
and all ten languages — the property the golden corpus pins.

## Determinism

Every reduction runs serially in a fixed level order and every serialised number
is rounded to a `1e-8` grid, so the same inputs yield the same bytes on every run,
every OS and every binding. The `parallel` feature (rayon) never changes a result;
it only affects how independent runs are scheduled.
