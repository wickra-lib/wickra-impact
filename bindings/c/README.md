<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Impact — the backtester that knows you would have moved the market" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/ci.svg)](https://github.com/wickra-lib/wickra-impact/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-impact)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/release.svg)](https://github.com/wickra-lib/wickra-impact/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-impact/license.svg)](https://github.com/wickra-lib/wickra-impact#license)

# Wickra Impact — C / C++

---

**Part of the [Wickra ecosystem](#ecosystem): — for C / C++. `cargo build -p wickra-impact-c --release` — a prebuilt shared/static library plus a generated `wickra_impact.h`, no system dependencies.**

The C ABI hub for Wickra Impact. It builds as a `cdylib` and a `staticlib` and
exposes a tiny JSON-over-C surface that every C-capable language (C, C++, C#, Go,
Java, R) links against. The whole evolutionary search lives in the Rust core;
this layer only marshals JSON strings across the boundary, so a fixed seed yields
the byte-identical search in every language.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-impact/releases) — each archive
has `wickra_impact.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-impact-c --release
# -> target/release/libwickra_impact.{so,dylib} or wickra_impact.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

### Building from this repository (contributors)

```bash
cargo build -p wickra-impact-c --release
```

This produces `wickra_impact.{dll,so,dylib}` (and a static library) under
`target/release/`. The header is committed at
[`include/wickra_impact.h`](https://github.com/wickra-lib/wickra-impact/blob/main/bindings/c/include/wickra_impact.h) and regenerated with:

```bash
cbindgen --config cbindgen.toml --crate wickra-impact-c --output include/wickra_impact.h
```

## Quick start

[`examples/c/run.c`](https://github.com/wickra-lib/wickra-impact/blob/main/examples/c/run.c) is the runnable example the CI smoke job executes; in full:

```c
/* A runnable C example: back-test a buy-and-hold strategy against a thin order
 * book through the wickra-impact C ABI and print the market impact. Every
 * language example runs the same thin_book request and prints the same summary. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_impact.h"

static const char *SPEC =
    "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
    "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true},"
    "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
    "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
    "\"book_model\":{\"kind\":\"orderbook_walk\"},\"participation_cap\":1.0,\"latency_ms\":0}";

/* The thin_book worked example: the second bar's ask ladder is thin. */
static const char *RUN_CMD =
    "{\"cmd\":\"run\",\"data\":{\"candles\":["
    "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
    "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}],"
    "\"books\":[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
    "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":3},"
    "{\"price\":100.3,\"size\":3},{\"price\":100.8,\"size\":4}]}]}}";

int main(void) {
    WickraImpact *impact = wickra_impact_new(SPEC);
    if (!impact) {
        fprintf(stderr, "failed to build impact\n");
        return 1;
    }
    int len = wickra_impact_command(impact, RUN_CMD, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        wickra_impact_free(impact);
        return 1;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        wickra_impact_free(impact);
        return 1;
    }
    wickra_impact_command(impact, RUN_CMD, buf, (size_t)len + 1);

    printf("wickra-impact %s\n", wickra_impact_version());
    printf("report bytes: %d\n", len);

    free(buf);
    wickra_impact_free(impact);
    return 0;
}
```

### Surface

```c
typedef struct WickraImpact WickraImpact;

WickraImpact *wickra_impact_new(const char *spec_json);   /* NULL on an invalid spec */
void          wickra_impact_free(WickraImpact *handle);   /* NULL-safe */
int32_t       wickra_impact_command(WickraImpact *handle, const char *cmd_json,
                                    char *out, uintptr_t cap);
const char   *wickra_impact_version(void);                /* static NUL string */
```

- `wickra_impact_new` takes a spec JSON (`"{}"` defers configuration to a later
  `set_spec` command); it returns `NULL` on a null / non-UTF-8 / invalid spec.
- `wickra_impact_command` applies a command envelope (`{"cmd":"...", ...}` —
  `set_spec`, `evolve`, `best`, `version`) and uses the classic two-call
  length-out protocol: call with `out = NULL`, `cap = 0` to learn the response
  length, then allocate `len + 1` and call again. A negative return is an
  unusable argument (`-1` null, `-2` non-UTF-8) or a caught panic (`-3`); a
  non-negative return is the response length. Domain errors come back **in-band**
  as `{"ok":false,"error":...}` JSON.
- `wickra_impact_version` returns a static version string (do not free).

### Determinism

The search's PRNG lives only in the Rust core; this binding forwards the command
string verbatim, so an `evolve` with a fixed seed produces the byte-identical
report here and in every other Wickra Impact binding.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-impact/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-impact>
- **Docs** (guides, spec reference, cookbook): <https://impact.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-impact/tree/main/examples/c)

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
