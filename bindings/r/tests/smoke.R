# Shipped with the package and run by `R CMD check`, so it must work inside the
# built tarball -- where no repository sits above it and no fixture file exists.
# It touches nothing but the package: load the library, take a handle, drive one
# command through the boundary, and read the version back.
#
# The cross-language golden parity test lives in run_tests.R, which is excluded
# from the tarball by .Rbuildignore and run from the repository by CI. That one
# needs golden/ above it; this one needs nothing.

library(wickraimpact)

v <- wkimpact_version()
stopifnot(is.character(v), length(v) == 1L, nzchar(v))

h <- wkimpact_new('{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h","indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},"sizing":{"type":"fixed_qty","qty":10.0},"execution":{"order_type":"market","fill_timing":"next_open"}},"book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}')
out <- wkimpact_command(h, '{"cmd":"version"}')
stopifnot(is.character(out), length(out) == 1L)
stopifnot(grepl("version", out, fixed = TRUE))

cat("wickra-impact R package smoke: ok (version ", v, ")\n", sep = "")
