# Wickra Impact — Go

Go bindings for the Wickra Impact market-impact backtester over its C ABI hub via
cgo. An `Impact` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Impact binding.

## Install

```bash
go get github.com/wickra-lib/wickra-impact/bindings/go
```

The prebuilt C ABI library is staged per platform under `lib/<goos>_<goarch>/`
and the header is vendored under `include/`. For a local build, copy the library
built by `cargo build -p wickra-impact-c --release` into the matching
`lib/<goos>_<goarch>/` directory (on Windows, ensure that directory is on `PATH`
when running tests).

## Usage

```go
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-impact/bindings/go"
)

func main() {
	spec := `{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",` +
		`"indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},` +
		`"sizing":{"type":"fixed_qty","qty":10.0},` +
		`"execution":{"order_type":"market","fill_timing":"next_open"}},` +
		`"book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}`

	impact, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer impact.Close()

	resp, err := impact.Command(`{"cmd":"run","data":` + data + `}`)
	if err != nil {
		panic(err)
	}
	fmt.Println(resp) // the report carries the market impact a naive backtest hides
}
```

## Surface

- **`New(specJSON)`** — build a backtest handle (`"{}"` defers to a later
  `set_spec`). Returns an error on an invalid spec.
- **`(*Impact).Command(cmdJSON)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `run`, `version`.
- **`(*Impact).Close()`** — free the handle (a finalizer also frees it).
- **`Version()`** — the library version.

## Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## See also

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
