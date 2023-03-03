# Description

A library that provides Rust wrappers for various library functionality of the mCRL2 toolset ([link](https://github.com/mCRL2org/mCRL2)) to allow writing tools in Rust that interface with mCRL2. For detailed documentation use `cargo doc --open` and visit the page for the `mcrl2-rust` crate, which contains the wrapper implementation. Compiling the library requires at least rustc version 1.58.0 and we use 2021 edition rust.

# Building

First the submodules must initialised to obtain the 3rd-party libraries. Furthermore, we need a C++ compiler to build the mCRL2 toolset. This can be MSVC on Windows, AppleClang on MacOS or either GCC or Clang on Linux. In the latter case it uses whatever compiler is provided by the CXX environment variable. After that the cargo workspace can be build. This will also build the necessary components of the mCRL2 toolset, which can take some time.

```bash
git submodule update --init --recursive
cargo build
```

# Tests

Tests can be performed using `cargo test`.

# Tools

This directory contains prototypes to show the viability of this approach. The tool can be executed using for example `cargo run --release tools/lpsreach`.

# Benchmarks

For micro benchmarks we use [Criterion.rs](https://crates.io/crates/criterion) and these benchmarks can be executed using `cargo bench`. For more functionality, such as history reports instead of only comparing to the previous benchmark run, we can install `cargo-criterion` using `cargo install criterion` and then run the benchmarks using `cargo criterion`. Note that this latter option is still experimental and might not always work.

# Profiling

The `lpsreach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/lpsreach` can be profiled using any standard executable profiler. This compilation profile contains debugging information to show where time is being spent, but the code is optimised the same as in a release configuration.

# Related Work

This library is fully inspired by the work on [mCRL2](https://github.com/mCRL2org/mCRL2).
