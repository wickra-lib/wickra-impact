# Wickra Impact — Python

Python bindings for the Wickra Impact market-impact backtester, built with PyO3
and maturin. An `Impact` handle is driven over a JSON boundary, so the same
request yields the byte-identical report as every other Wickra Impact binding.

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/wickra-lib/wickra-impact#license)
[![PyO3](https://img.shields.io/badge/bindings-PyO3-3b82f6)](https://pyo3.rs)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

## Install

```bash
pip install wickra-impact
```

## Build from source

```bash
maturin develop --release
```

## Usage

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

## API

- **`Impact(spec_json)`** — construct a backtest handle from an `ImpactSpec` JSON
  (`"{}"` defers configuration to a later `set_spec`). Raises `ValueError` on an
  invalid spec.
- **`Impact.command(cmd_json)`** — apply a command JSON and return the response
  JSON. Commands: `set_spec`, `run`, `version`. Raises `RuntimeError` on a
  command failure.
- **`Impact.version()`** / **`wickra_impact.__version__`** — the library version.

## Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so the report is byte-identical to the CLI and to every other
language binding.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-APACHE).
