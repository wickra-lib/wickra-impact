## Plain-R tests for the wickra-impact R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickraimpact)

spec <- paste0(
  '{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",',
  '"indicators":{},"entry":{"ge":[{"price":"close"},0]},',
  '"exit":{"in_position":true},"sizing":{"type":"fixed_qty","qty":10.0},',
  '"execution":{"order_type":"market","fill_timing":"next_open"}},',
  '"book_model":{"kind":"orderbook_walk"},',
  '"participation_cap":1.0,"latency_ms":0}'
)

## The thin-book worked example: the second bar's ask ladder is thin, so a
## market order walks up it and pays 44 bps of slippage a naive backtest hides.
data <- paste0(
  '{"candles":[',
  '{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},',
  '{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],',
  '"books":[',
  '{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},',
  '{"bids":[{"price":99.9,"size":100}],"asks":[',
  '{"price":100.1,"size":3},{"price":100.3,"size":3},',
  '{"price":100.8,"size":4}]}]}'
)

run_cmd <- function() {
  paste0('{"cmd":"run","data":', data, "}")
}

## version
stopifnot(nzchar(wkimpact_version()))

## run returns a report that carries the measured market impact
impact <- wkimpact_new(spec)
out <- wkimpact_command(impact, run_cmd())
stopifnot(grepl('"avg_slippage_bps":44.0', out, fixed = TRUE))
stopifnot(grepl('"entry_price":100.44', out, fixed = TRUE))

## run is byte-identical across handles (the cross-language golden core)
impact2 <- wkimpact_new(spec)
out2 <- wkimpact_command(impact2, run_cmd())
stopifnot(identical(out, out2))

## an invalid spec is a hard error at construction
err <- tryCatch(wkimpact_new("{ not valid json"), error = function(e) e)
stopifnot(inherits(err, "error"))

## set_spec on a deferred handle, then run
deferred <- wkimpact_new("{}")
ok <- wkimpact_command(deferred, paste0('{"cmd":"set_spec","spec":', spec, "}"))
stopifnot(grepl('"ok":true', ok, fixed = TRUE))
stopifnot(grepl('"impact_stats"', wkimpact_command(deferred, run_cmd()), fixed = TRUE))

cat("wickra-impact R tests passed\n")
