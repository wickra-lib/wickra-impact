<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/pypi.svg)](https://pypi.org/project/wickra-impact/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — Python

---

**Part of the [Wickra ecosystem](#ecosystem): — for Python. `pip install wickra-impact` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

Python bindings for the Wickra Impact market-impact backtester, built with PyO3
and maturin. An `Impact` handle is driven over a JSON boundary, so the same
request yields the byte-identical report as every other Wickra Impact binding.

## Install

```bash
pip install wickra-impact
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

### Building from this repository (contributors)

```bash
maturin develop --release
```

## Quick start

```python
import json
from wickra_impact import Impact

spec = {
    "strategy": { "spec_version": 1, "symbol": "IMPACT", "timeframe": "1h",
        "indicators": {},
        "entry": {"ge": [{"price": "close"}, 0]},
        "exit": {"in_position": True},
        "sizing": {"type": "fixed_qty", "qty": 10.0},
        "execution": {"order_type": "market", "fill_timing": "next_open"} },
    "book_model": {"kind": "orderbook_walk"},
    "participation_cap": 1.0, "latency_ms": 0,
}

impact = Impact(json.dumps(spec))
report = json.loads(impact.command(json.dumps({"cmd": "run", "data": data})))
print(report["impact_stats"]["avg_slippage_bps"])  # the impact a naive backtest hides
```

### API

- **`Impact(spec_json)`** — construct a backtest handle from an `ImpactSpec` JSON
  (`"{}"` defers configuration to a later `set_spec`). Raises `ValueError` on an
  invalid spec.
- **`Impact.command(cmd_json)`** — apply a command JSON and return the response
  JSON. Commands: `set_spec`, `run`, `version`. Raises `RuntimeError` on a
  command failure.
- **`Impact.version()`** / **`wickra_impact.__version__`** — the library version.

### Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so the report is byte-identical to the CLI and to every other
language binding.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/python)

Wickra Impact ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-impact/blob/main/SECURITY.md>.

## Disclaimer

Wickra Impact is a research and backtesting tool. It measures the slippage an
order would have paid against recorded market data; it does not place orders,
and a measurement over history is not a prediction about the future. Nothing
here is financial advice. Trading carries risk, including the loss of the
capital committed. Use it at your own risk.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-MIT) at your option.
