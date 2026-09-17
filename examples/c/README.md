# Wickra Impact — C / C++ examples

The Wickra Impact C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_impact.h`](../../bindings/c/include/wickra_impact.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_impact.hpp`](../../bindings/c/include/wickra_impact.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-impact-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_impact.so`     | `-lwickra_impact` |
| macOS    | `libwickra_impact.dylib`  | `-lwickra_impact` |
| Windows (MSVC) | `wickra_impact.dll` | `wickra_impact.dll.lib` (import lib) |

A static library (`libwickra_impact.a` / `wickra_impact.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/run.c -I bindings/c/include -L target/release -lwickra_impact -lm -o run
LD_LIBRARY_PATH=target/release ./run        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/run.c -I bindings/c/include target/release/wickra_impact.dll -lm -o run.exe
```

## The examples

| Example | What it does |
|---------|--------------|
| `run.c` | A runnable C example: back-test a buy-and-hold strategy against a thin order |
| `run.cpp` | A runnable C++ example: back-test a buy-and-hold strategy against a thin order book and print the market impact. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_impact.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
