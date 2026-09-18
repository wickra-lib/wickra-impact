# Wickra Impact examples

Runnable examples in every Wickra Impact language. Each one runs the same
`thin_book` request — a buy-and-hold order that lifts a thin ask ladder — and
prints the same summary, the market impact a naive backtest never sees:

## What every example prints

Runnable examples in every Wickra Impact language. Each one runs the same
`thin_book` request — a buy-and-hold order that lifts a thin ask ladder — and
prints the same summary, the market impact a naive backtest never sees:

```
wickra-impact 0.1.3
avg slippage: 44.0 bps
entry price: 100.44
```

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-impact-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `run.c` | A runnable C example: back-test a buy-and-hold strategy against a thin order |
| `run.cpp` | A runnable C++ example: back-test a buy-and-hold strategy against a thin order book and print the market impact. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Run
```

| Example | What it does |
| --- | --- |
| `Run/Program.cs` | A runnable C# example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `run.go` | A runnable Go example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/run.R
```

| Example | What it does |
| --- | --- |
| `run.R` | A runnable R example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q install -DskipTests
mvn -f examples/java/pom.xml -q compile exec:exec  -Dnative.lib.dir="$PWD/target/release"
```

| Example | What it does |
| --- | --- |
| `src/main/java/org/wickra/impact/examples/Run.java` | A runnable example against this binding. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-impact
python examples/python/run.py
```

| Example | What it does |
| --- | --- |
| `run.py` | A runnable Python example: back-test a buy-and-hold strategy against a thin |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/run.js
```

| Example | What it does |
| --- | --- |
| `run.js` | A runnable Node.js example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `slippage.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): . The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).

## Per language

- **Rust** — [`rust/`](rust/): `cargo run --manifest-path examples/rust/Cargo.toml`
- **Python** — [`python/run.py`](python/run.py): `pip install wickra-impact && python examples/python/run.py`
- **Node.js** — [`node/`](node/): `cd examples/node && npm install && node run.js`
- **Go** — [`go/`](go/): `go run examples/go/run.go` (with the C ABI library staged, see the Go binding README)
- **C#** — [`csharp/Run/`](csharp/Run/): `dotnet run --project examples/csharp/Run`
- **Java** — [`java/`](java/): `mvn -q compile exec:java -Dexec.mainClass=org.wickra.impact.examples.Run`
- **R** — [`r/run.R`](r/run.R): `R CMD INSTALL bindings/r && Rscript examples/r/run.R`
- **WASM** — [`wasm/slippage.html`](wasm/slippage.html): `wasm-pack build bindings/wasm --target web`, serve the repository root, then open `examples/wasm/slippage.html`
- **C / C++** — [`c/`](c/): build the C ABI, then CMake + ctest:

  ```bash
  cargo build --release -p wickra-impact-c
  cmake -S examples/c -B examples/c/build
  cmake --build examples/c/build --config Release
  ctest --test-dir examples/c/build -C Release --output-on-failure
  ```

The binding examples install the published `wickra-impact` package for their
language; the Rust and C/C++ examples build against the in-repo core.
