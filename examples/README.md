# Examples

Runnable examples in every Wickra Impact language. Each one runs the same
`thin_book` request — a buy-and-hold order that lifts a thin ask ladder — and
prints the same summary, the market impact a naive backtest never sees:

```
wickra-impact 0.1.0
avg slippage: 44.0 bps
entry price: 100.44
```

That identical output across ten languages is the cross-language guarantee: the
fill engine lives once in the Rust core and every binding forwards the command
JSON verbatim. See [`golden/README.md`](../golden/README.md) for the byte-golden
corpus these numbers come from.

The canonical spec and the request bundle are also in [`data/`](data/) for the
CLI:

```bash
cargo run -p impact-cli -- --request examples/data/requests/thin_book.json
```

## Per language

- **Rust** — [`rust/`](rust/): `cargo run --manifest-path examples/rust/Cargo.toml`
- **Python** — [`python/run.py`](python/run.py): `pip install wickra-impact && python examples/python/run.py`
- **Node.js** — [`node/`](node/): `cd examples/node && npm install && node run.js`
- **Go** — [`go/`](go/): `go run examples/go/run.go` (with the C ABI library staged, see the Go binding README)
- **C#** — [`csharp/Run/`](csharp/Run/): `dotnet run --project examples/csharp/Run`
- **Java** — [`java/`](java/): `mvn -q compile exec:java -Dexec.mainClass=org.wickra.impact.examples.Run`
- **R** — [`r/run.R`](r/run.R): `R CMD INSTALL bindings/r && Rscript examples/r/run.R`
- **C / C++** — [`c/`](c/): build the C ABI, then CMake + ctest:

  ```bash
  cargo build --release -p wickra-impact-c
  cmake -S examples/c -B examples/c/build
  cmake --build examples/c/build --config Release
  ctest --test-dir examples/c/build -C Release --output-on-failure
  ```

The binding examples install the published `wickra-impact` package for their
language; the Rust and C/C++ examples build against the in-repo core.
