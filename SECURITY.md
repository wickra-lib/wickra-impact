# Security policy

## Supported versions

This project is pre-1.0 (alpha). Security fixes are applied to the latest
released version, `0.1.3`, only; please upgrade to the newest release before
reporting an issue.

| Version | Supported |
|---------|-----------|
| 0.1.3 (latest) | ✅ |

## Reporting a vulnerability

Please report suspected vulnerabilities privately via
[GitHub Security Advisories](https://github.com/wickra-lib/wickra-impact/security/advisories/new),
not in public issues. We aim to acknowledge within a few days and to coordinate a
fix and disclosure timeline with you.

## Codegen note (read before running the build step)

Wickra Impact **generates code** and can **invoke `cargo` on it**. Treat it like
any code generator that runs a compiler:

- Run it only on **trusted specs**. A spec becomes source that a build step
  compiles and runs (`build.rs`, proc-macros, the resulting artifact).
- Code generation itself does **no network access** and spawns no processes; only
  the optional build step (the `build` feature, or the CLI without `--dry-run`)
  invokes `cargo`, which then fetches and builds dependencies.
- Compile untrusted third-party specs in a sandbox with no secrets and restricted
  network/file-system access.

See [THREAT_MODEL.md](THREAT_MODEL.md) for the full analysis.
