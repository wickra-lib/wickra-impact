<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-impact/bindings/go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — Go

---

**Part of the [Wickra ecosystem](#ecosystem): — for Go. `go get github.com/wickra-lib/wickra-impact/bindings/go` — over the C ABI via cgo, prebuilt library bundled in the module.**

Go bindings for the Wickra Impact market-impact backtester over its C ABI hub via
cgo. An `Impact` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Impact binding.

## Install

Use the published **`wickra-impact-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-impact-go
```

`wickra-impact-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_impact.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

The prebuilt C ABI library is staged per platform under `lib/<goos>_<goarch>/`
and the header is vendored under `include/`. For a local build, copy the library
built by `cargo build -p wickra-impact-c --release` into the matching
`lib/<goos>_<goarch>/` directory (on Windows, ensure that directory is on `PATH`
when running tests).

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-impact-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_impact.so bindings/go/lib/linux_amd64/
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

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

### Surface

- **`New(specJSON)`** — build a backtest handle (`"{}"` defers to a later
  `set_spec`). Returns an error on an invalid spec.
- **`(*Impact).Command(cmdJSON)`** — apply a command envelope
  (`{"cmd":"...", ...}`) and return the response JSON. Commands: `set_spec`,
  `run`, `version`.
- **`(*Impact).Close()`** — free the handle (a finalizer also frees it).
- **`Version()`** — the library version.

### Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/go)

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

Wickra Impact ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-impact/blob/main/SECURITY.md>.

## Disclaimer

Wickra Impact is a research and backtesting tool. It measures the slippage an
order would have paid against recorded market data; it does not place orders,
and a measurement over history is not a prediction about the future. Nothing
here is financial advice. Trading carries risk, including the loss of the
capital committed. Use it at your own risk.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-impact/blob/main/LICENSE-MIT) at your option.
