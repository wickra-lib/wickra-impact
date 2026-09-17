# Wickra Impact examples — C#

Runnable C# examples for the [Wickra Impact C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-impact-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Run
```

## The examples

| Example | What it does |
|---------|--------------|
| `Run/Program.cs` | A runnable C# example: back-test a buy-and-hold strategy against a thin order book and print the market impact the walk measured. |
