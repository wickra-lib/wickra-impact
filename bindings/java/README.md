# Wickra Impact — Java

JVM bindings for the Wickra Impact market-impact backtester over its C ABI hub,
using the Foreign Function & Memory API (FFM / Panama). An `Impact` is built from
a spec JSON and driven over a JSON boundary, so the result is byte-identical to
every other Wickra Impact binding.

## Requirements

- JDK 22+ (the FFM API is stable since Java 22). Run with
  `--enable-native-access=ALL-UNNAMED`.
- The native C ABI library, built by `cargo build -p wickra-impact-c`.
  The binding loads it from the directory named by the `native.lib.dir` system
  property (the Maven build points it at the workspace `target/debug`).

## Usage

```java
import org.wickra.impact.Impact;

String spec = "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\","
    + "\"timeframe\":\"1h\",\"indicators\":{},"
    + "\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true},"
    + "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
    + "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
    + "\"book_model\":{\"kind\":\"orderbook_walk\"},"
    + "\"participation_cap\":1.0,\"latency_ms\":0}";

try (Impact impact = new Impact(spec)) {
    String response = impact.command("{\"cmd\":\"run\",\"data\":" + data + "}");
    System.out.println(response); // the report carries the impact a naive backtest hides
}
```

## Surface

- **`new Impact(specJson)`** — build a backtest handle (`"{}"` defers to a later
  `set_spec`). Throws `IllegalArgumentException` on an invalid spec.
- **`command(cmdJson)`** — apply a command envelope (`{"cmd":"...", ...}`) and
  return the response JSON. Commands: `set_spec`, `run`, `version`.
- **`Impact.version()`** — the library version.
- **`close()`** — free the native handle (try-with-resources recommended).

## Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## See also

- The main project: <https://github.com/wickra-lib/wickra-impact>
- Documentation: <https://wickra.org>

## License

Dual-licensed under either MIT or Apache-2.0, at your option.
