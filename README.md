# Description

A library that provides Rust wrappers for various library functionality of the mCRL2 toolset ([link](https://github.com/mCRL2org/mCRL2)) to allow writing tools in Rust that interface with mCRL2. For detailed documentation use `cargo doc --open` and visit the page for the `mcrl2-rust` crate, which contains the wrapper implementation. Compiling the library requires at least rustc version 1.58.0 and we use 2021 edition rust.

# Building

First the submodules must initialised to obtain the 3rd-party libraries. After that the workspace can be build. This will also build the necessary components of the mCRL2 toolset, which can take some time.

```bash
git submodule update --init --recursive
cargo build -j<number_of_cores>
```

# Tests

Tests can be performed using `cargo test`.

# Tools

This directory contains prototypes to show the viability of this approach. The tool can be executed using for example `cargo run --release tools/lpsreach`.

# Profiling

The `lpsreach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/lpsreach` can be profiled using any standard executable profiler. This compilation profile contains debugging information to show where time is being spent, but the code is optimised the same as in a release configuration.

# Related Work

This library is fully inspired by the work on [mCRL2](https://github.com/mCRL2org/mCRL2).
