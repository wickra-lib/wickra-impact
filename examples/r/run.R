# A runnable R example: back-test a buy-and-hold strategy against a thin order
# book and print the market impact the walk measured.
#
#   R CMD INSTALL bindings/r
#   Rscript examples/r/run.R
#
# Every language example runs the same thin_book request and prints the same
# summary — that is the cross-language guarantee.
library(wickraimpact)

spec <- paste0(
  '{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",',
  '"indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},',
  '"sizing":{"type":"fixed_qty","qty":10.0},',
  '"execution":{"order_type":"market","fill_timing":"next_open"}},',
  '"book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}'
)

# The thin_book worked example: the second bar's ask ladder is thin.
cmd <- paste0(
  '{"cmd":"run","data":{"candles":[',
  '{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},',
  '{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],',
  '"books":[{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},',
  '{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":3},',
  '{"price":100.3,"size":3},{"price":100.8,"size":4}]}]}}'
)

impact <- wkimpact_new(spec)
report <- wkimpact_command(impact, cmd)

cat(sprintf("wickra-impact %s\n", wkimpact_version()))
bps <- regmatches(report, regexpr('"avg_slippage_bps":[0-9.]+', report))
entry <- regmatches(report, regexpr('"entry_price":[0-9.]+', report))
cat(sprintf("avg slippage: %s bps\n", sub('.*:', '', bps)))
cat(sprintf("entry price: %s\n", sub('.*:', '', entry)))
