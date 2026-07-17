# Latency

A signal computed on one bar cannot fill on that same bar without look-ahead. Two
knobs decide which order-book snapshot a signalled order fills against, and both
are deterministic integer arithmetic.

## `fill_timing`

The strategy's execution block sets the base offset. `next_open` — the
look-ahead-free default — fills against the **next** bar's book. This is why the
`thin_book` signal on bar 0 fills against bar 1's ladder.

## `latency_ms`

`ImpactSpec.latency_ms` adds simulated latency on top. Given the timeframe's
milliseconds per bar, the order shifts forward by `ceil(latency_ms / bar_ms)`
additional bars:

```
index = signal_bar + (next_open ? 1 : 0) + ceil(latency_ms / bar_ms)
```

If that index runs past the last bar, the order is **cancelled** (it never fills).
The mapping is `latency::snapshot_index`; the timeframe parsing
(`"1h"` → `3_600_000`) is `latency::bar_ms`, both pure integer functions, so the
choice of snapshot is identical on every run and in every binding.

## Example

On 1-hour bars with `latency_ms = 5_400_000` (90 minutes) and `next_open`, a
signal on bar 0 fills against bar `0 + 1 + ceil(90/60) = 3`. Raise the latency far
enough that the target runs off the end of the series and the order is cancelled
rather than filled at a stale price.
