<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-impact)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — Java

---

**Part of the [Wickra ecosystem](#ecosystem): — for Java. `org.wickra:wickra-impact` — prebuilt native library inside the jar, no JNI, no system dependencies.**

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

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-impact</artifactId>
  <version>0.1.4</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-impact:0.1.4")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

## Quick start

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

### Surface

- **`new Impact(specJson)`** — build a backtest handle (`"{}"` defers to a later
  `set_spec`). Throws `IllegalArgumentException` on an invalid spec.
- **`command(cmdJson)`** — apply a command envelope (`{"cmd":"...", ...}`) and
  return the response JSON. Commands: `set_spec`, `run`, `version`.
- **`Impact.version()`** — the library version.
- **`close()`** — free the native handle (try-with-resources recommended).

### Determinism

The fill engine lives only in the Rust core; this binding forwards the command
string verbatim, so a given request produces the byte-identical report here and
in every other binding — the exact cross-language golden invariant.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/java)

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
