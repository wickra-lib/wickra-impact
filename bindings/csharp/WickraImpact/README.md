# Wickra Impact — C#

.NET bindings for the Wickra Impact market-impact backtester over its C ABI hub.
An `Impact` is built from a spec JSON and driven over a JSON boundary, so the
result is byte-identical to every other Wickra Impact binding.

## Install

```bash
dotnet add package Wickra.Impact
```

The package ships the native C ABI library per runtime identifier under
`runtimes/<rid>/native/`. For a local build, `cargo build -p wickra-impact-c --release`
places the library in `target/release/`; the bundled `DllImportResolver` probes
the Cargo `target/` tree, so tests and apps in the repo find it without extra
steps.

## Usage

```csharp
using Wickra.Impact;

const string spec = """
{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",
 "indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},
 "sizing":{"type":"fixed_qty","qty":10.0},
 "execution":{"order_type":"market","fill_timing":"next_open"}},
 "book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}
""";

using var impact = new Impact(spec);
string report = impact.Command($"{{\"cmd\":\"run\",\"data\":{data}}}");
Console.WriteLine(report); // the report carries the market impact a naive backtest hides
```

## Surface

- **`new Impact(specJson)`** — build a backtest handle (`"{}"` defers to a later
  `set_spec`). Throws `ArgumentException` on an invalid spec.
- **`Command(cmdJson)`** — apply a command envelope (`{"cmd":"...", ...}`) and
  return the response JSON. Commands: `set_spec`, `run`, `version`.
- **`Impact.Version()`** — the library version.
- **`Dispose()`** — free the native handle (`using` recommended).

## Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## See also

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
