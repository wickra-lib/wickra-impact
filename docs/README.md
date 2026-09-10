# Documentation

These pages are the guides that live beside the code, because they describe how
this repository behaves and have to change in the same commit the behaviour does.

| Page | What it answers |
|------|-----------------|
| [IMPACT_MODELS.md](IMPACT_MODELS.md) | The fill models: the book walk and the analytic curves, and which needs what |
| [BOOK_WALK.md](BOOK_WALK.md) | How an order is walked across levels, and what the fill price means |
| [LATENCY.md](LATENCY.md) | What `latency_ms` delays, and what it does not |
| [INHERITANCE.md](INHERITANCE.md) | What is inherited from `wickra-backtest` and what IMPACT replaces |
| [ARCHITECTURE.md](ARCHITECTURE.md) | The internals: where the fill stage sits in the engine |
| [Cookbook.md](Cookbook.md) | Worked runs |

The API reference for each language is generated from the source rather than
committed here — `cargo doc` for Rust, the `.d.ts` beside the Node binding, the
docstrings in the Python module, the C header. Keeping a second copy in this
repository would drift from the code that generates it, and a reader opening
`docs/` would have no way to tell which copy was current.

The engine IMPACT inherits documents itself at
<https://github.com/wickra-lib/wickra-backtest>.

What stays here is what a generator cannot produce: the meaning of a field, the
reason a case is refused rather than answered, and the worked examples.

Elsewhere in the repository:

- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the crate and binding layout
- [`../BENCHMARKS.md`](../BENCHMARKS.md) — what is measured and how
- [`../golden/README.md`](../golden/README.md) — the cross-language corpus and how to re-bless it
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — how to build, test and propose a change
- [`../THREAT_MODEL.md`](../THREAT_MODEL.md) — what the engine does and does not touch
