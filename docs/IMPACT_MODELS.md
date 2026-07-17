# Impact models

`ImpactSpec.book_model` selects how a market order becomes a fill price. All three
models are deterministic and round the fill onto the `1e-8` grid.

## `orderbook_walk` — the moat

Walk the real recorded L2 book level by level. For a buy, consume ask size from
the inside price outward until the order quantity is met (or the book / the
participation cap is exhausted); the fill price is the size-weighted average of the
levels taken. This is the only model that reflects the actual shape of the book;
the others are analytic fallbacks for when a full book is not available.

```json
{ "kind": "orderbook_walk", "levels": null }
```

`levels` optionally caps how deep the walk goes (`null` = all provided levels).
See [BOOK_WALK](BOOK_WALK.md) for the worked example.

## `linear_impact`

An analytic curve: the fill price moves linearly with the fraction of top-of-book
depth the order consumes.

```
fill = reference · (1 ± coef · (qty / depth))
```

`+` for a buy, `−` for a sell. `coef` is a finite, non-negative impact
coefficient; `depth` is the inside level's size.

```json
{ "kind": "linear_impact", "coef": 0.1 }
```

## `square_root`

The square-root law of market impact — the empirically common shape for large
orders:

```
fill = reference · (1 ± coef · √(qty / depth))
```

```json
{ "kind": "square_root", "coef": 0.5 }
```

## Participation cap

Independently of the model, `participation_cap ∈ (0, 1]` bounds the fraction of a
bar's available liquidity a single order may take. If the order wants more, the
remainder is left unfilled — a **partial fill**, counted in
`ImpactStats.orders_partially_filled`.
