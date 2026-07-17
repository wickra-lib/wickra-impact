# Golden corpus

The generate-once / replay-everywhere fixtures that pin Wickra Impact's
determinism: for every `specs/<name>.json` paired with `data/<name>.json`, the
`run` report serializes **byte-for-byte** to `expected/<name>.json` — in the Rust
core, the CLI, and every one of the ten language bindings.

> **Never edit `expected/*.json` by hand.** They are generated (blessed) from the
> core via the CLI. A hand edit desyncs the byte-golden and every downstream
> binding test fails.

## Layout

| Dir         | Contents                                                            |
|-------------|---------------------------------------------------------------------|
| `data/`     | Fixed deterministic `RunData` — candles plus per-bar L2 books.      |
| `specs/`    | Canonical `ImpactSpec` envelopes (strategy + book model + caps).    |
| `expected/` | One blessed `ImpactReport` JSON per spec (`serde_json::to_string`).  |

`specs/<name>.json` pairs with `data/<name>.json` by basename.

## Cases

| Name               | Book model              | What it pins                                             |
|--------------------|-------------------------|----------------------------------------------------------|
| `thin_book`        | `orderbook_walk`        | The worked example: a thin ask ladder → **44 bps**, entry `100.44`. |
| `thin_book_capped` | `orderbook_walk`, cap 0.5 | Participation cap leaves a **partial fill** (5 of 10).  |
| `deep_book`        | `orderbook_walk`        | Deep book absorbs the order at the inside → no walk, only the half-spread. |
| `linear_impact`    | `linear_impact` coef 0.1  | Analytic linear impact `ref·(1+coef·qty/depth)`.       |
| `square_root`      | `square_root` coef 0.5    | Analytic square-root impact `ref·(1+coef·√(qty/depth))`. |
| `latency`          | `orderbook_walk`, latency 1000ms | The latency path selects the reference book.    |

## Data formula

The candle path is a fixed two-bar sequence (a flat bar, then an up bar so the
`close ≥ 0` entry triggers and fills at the next bar's book). The books are hand-
authored L2 ladders — a thin ask ladder for `thin_book` (sizes `3 / 3 / 4` at
`100.1 / 100.3 / 100.8`), a deep ladder (`1000` at the inside) for `deep_book`.
The numbers are fixed, not sampled: the corpus must be reproducible bit-for-bit.

## Bless command

Regenerate every `expected/*.json` from the current core (run after any
intentional change to the fill engine or report shape, and review the diff):

```bash
cargo build -p impact-cli --release
for name in thin_book thin_book_capped deep_book linear_impact square_root latency; do
  target/release/wickra-impact --spec golden/specs/$name.json \
    --data golden/data/$name.json --format json > golden/expected/$name.json
done
```
