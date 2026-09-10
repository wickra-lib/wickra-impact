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


## The book the walk needs, through the same boundary.
##
## There is no streaming half in IMPACT: a run is a batch computation. What
## stands in its place is the property the whole repository exists for -- that
## slippage is measured rather than guessed -- and the one way it can fail
## quietly: `orderbook_walk` with no book. The core refuses it. A binding that
## swallowed the refusal would report zero slippage from the model whose entire
## purpose is to find some.

book_strategy <- paste0(
  '{"spec_version":1,"symbol":"IMPACT","timeframe":"1h","indicators":{},',
  '"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},',
  '"sizing":{"type":"fixed_qty","qty":10.0},',
  '"execution":{"order_type":"market","fill_timing":"next_open"}}'
)

book_candles <- paste0(
  '[{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},',
  '{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}]'
)

book_levels <- paste0(
  '[{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},',
  '{"bids":[{"price":99.9,"size":100}],"asks":[',
  '{"price":100.1,"size":3},{"price":100.3,"size":3},{"price":100.8,"size":4}]}]'
)

book_spec <- function(model) {
  paste0('{"strategy":', book_strategy, ',"book_model":', model,
         ',"participation_cap":1.0,"latency_ms":0}')
}

book_run <- function(model, data) {
  tryCatch({
    handle <- wkimpact_new(book_spec(model))
    wkimpact_command(handle, paste0('{"cmd":"run","data":', data, "}"))
  }, error = function(e) conditionMessage(e))
}

## A walk with no book is refused, not answered with zero slippage.
no_book <- book_run('{"kind":"orderbook_walk"}', paste0('{"candles":', book_candles, "}"))
stopifnot(grepl("book", no_book, fixed = TRUE))
stopifnot(!grepl('"impact_stats"', no_book, fixed = TRUE))

## With the book, the walk pays 100.44 rather than the 100.10 top of book.
with_book <- book_run('{"kind":"orderbook_walk"}',
                      paste0('{"candles":', book_candles, ',"books":', book_levels, "}"))
stopifnot(grepl('"avg_slippage_bps":44', with_book, fixed = TRUE))

## An analytic model prices from a curve and needs no book.
analytic <- book_run('{"kind":"linear_impact","coef":0.1}',
                     paste0('{"candles":', book_candles, "}"))
stopifnot(grepl('"impact_stats"', analytic, fixed = TRUE))

## The batch run is reproducible.
book_data <- paste0('{"candles":', book_candles, ',"books":', book_levels, "}")
stopifnot(identical(book_run('{"kind":"orderbook_walk"}', book_data),
                    book_run('{"kind":"orderbook_walk"}', book_data)))

cat("wickra-impact R tests passed\n")
