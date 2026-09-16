# Wickra Impact examples — R

Runnable R examples for the [Wickra Impact R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-impact-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/run.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `run.R` | A runnable R example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |
