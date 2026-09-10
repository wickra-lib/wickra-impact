# wickra-impact (C#)

.NET bindings for [`wickra-impact`](https://github.com/wickra-lib/wickra-impact)
over the C ABI hub, via source-generated P/Invoke. Build an `Impact` from a spec
JSON, run a backtest and read back the report with its impact statistics — the
same protocol the CLI and every other binding speak, returning the same bytes.

```csharp
using Wickra.Impact;

const string spec = """
{"strategy": { … },
 "book_model":{"kind":"orderbook_walk"},
 "participation_cap":1.0,"latency_ms":0}
""";

using var impact = new Impact(spec);
string report = impact.Command("""{"cmd":"run","data":{"candles":[ … ],"books":[ … ]}}""");
```

`orderbook_walk` walks the order across the real historical book and reports the
volume-weighted price it paid — and it **needs** that book. A run whose data
carries none is refused rather than answered with zero slippage from the model
whose entire purpose is to find some. The analytic models (`linear_impact`,
`square_root`) price from a curve and need no book.

Requires .NET 8+. The native library (`wickra_impact`) must be resolvable on the
loader path — `PATH` on Windows, `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH`
on macOS — or beside the assembly, where the bundled resolver finds it.

Licensed under either of [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE) at your option.
