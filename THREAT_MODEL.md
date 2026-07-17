# Threat Model

## Assets

Wickra Impact holds no secrets, keys or funds. It reads strategy specs, candle
data and recorded L2 order books and emits backtest reports. The asset is
availability: an untrusted request must not exhaust host resources.

## Actors

- **Operator** — runs a backtest on trusted or semi-trusted inputs.
- **Request author** — supplies the `StrategySpec`, the candle series and the
  order books; the untrusted input surface.

## Threats and mitigations

- **Resource exhaustion via oversized books/feeds.** Inputs are validated
  (non-empty, sorted, uncrossed books, as inherited from `wickra-backtest`); the
  order-book walk is bounded by the book depth, so a fill terminates in the
  levels present.
- **Non-determinism.** Every output path is ordered (`BTreeMap`, stably sorted
  level vectors) and floats are rounded before serialisation, so a report is
  byte-identical across runs, threads and language bindings — a property the
  golden corpus enforces.
- **Untrusted native boundary.** The C ABI catches panics at the FFI edge and
  guards null pointers; no input crosses into `unsafe` without a length.

## Out of scope

Order placement, key custody and network trading are not part of Impact — it
reads recorded data only.
