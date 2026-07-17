# Cookbook

Recipes for common Wickra Impact runs. Every recipe drives the same
`Impact::command_json` surface; only the spec and data change.

## Measure slippage against a real book

Pair an `orderbook_walk` spec with a `RunData` that carries per-bar `books`:

```bash
cargo run -p impact-cli -- --spec golden/specs/thin_book.json --data golden/data/thin_book.json --format text
```

The `market impact` block reports the average slippage, the liquidity consumed and
the partial-fill count.

## Cap your participation

Set `participation_cap` below 1.0 to stop a single order eating more than a
fraction of a bar's liquidity; the rest is left unfilled.

```json
{ "...": "...", "book_model": { "kind": "orderbook_walk" }, "participation_cap": 0.5 }
```

See `golden/specs/thin_book_capped.json`: the 10-unit order fills only 5 and the
report shows one partial fill.

## Model impact without a full book

When you only have top-of-book depth, use an analytic curve:

```json
{ "...": "...", "book_model": { "kind": "square_root", "coef": 0.5 } }
```

## Simulate latency

Delay when a signalled order fills with `latency_ms`; on 1-hour bars, 90 minutes of
latency shifts the fill two bars further forward (plus the `next_open` bar). See
[LATENCY](LATENCY.md).

## Run from any language

Build an `Impact` from the spec and send a `run` command; the response is the same
`ImpactReport` JSON in every language. For example, in Python:

```python
import json
from wickra_impact import Impact

impact = Impact(open("golden/specs/thin_book.json").read())
data = json.load(open("golden/data/thin_book.json"))
report = json.loads(impact.command(json.dumps({"cmd": "run", "data": data})))
print(report["impact_stats"]["avg_slippage_bps"])   # 44.0
```

The equivalent runnable programs for all ten languages are in
[`examples/`](../examples/).

## Re-bless the golden corpus

After an intended change to the fill engine or report shape, regenerate the fixed
`golden/expected/*.json` and review the diff:

```bash
cargo build -p impact-cli --release
for name in thin_book thin_book_capped deep_book linear_impact square_root latency; do
  target/release/wickra-impact --spec golden/specs/$name.json \
    --data golden/data/$name.json --format json > golden/expected/$name.json
done
```
