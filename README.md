# Description

A library that provides Rust wrappers for various library functionality of the mCRL2 toolset ([link](https://github.com/mCRL2org/mCRL2)) to allow writing tools in Rust that interface with mCRL2. For detailed documentation use `cargo doc --open` and visit the page for the `mcrl2-rust` crate, which contains the wrapper implementation. Compiling the library requires at least rustc version 1.58.0 and we use 2021 edition rust.

# Building

First the submodules must initialised to obtain the 3rd-party libraries. Furthermore, we need a C++ compiler to build the mCRL2 toolset. This can be MSVC on Windows, AppleClang on MacOS or either GCC or Clang on Linux. In the latter case it uses whatever compiler is provided by the CXX environment variable. After that the cargo workspace can be build. This will also build the necessary components of the mCRL2 toolset, which can take some time.

```bash
git submodule update --init --recursive
cargo build
```

Since we use submodules it is also necessary to run `git submodule update` after pulling from the remote whenever any of the modules have been updated remotely.

# Tools

The tools directory contains prototypes to show the viability of this approach. The `lpsreach` tool can for example be executed using `cargo run --release --bin lpsreach`.

# Tests

Tests can be performed using `cargo test`. To show print statements during tests use `cargo test -- --nocapture`, and use `cargo test -p sabre --lib` to only run the tests belonging to the Sabre library.

# Benchmarks

For micro benchmarks we use [Criterion.rs](https://crates.io/crates/criterion) and these benchmarks can be executed using `cargo bench`. 

# Address Sanitizer

For Linux targets it is also possible to run the LLVM address sanitizer to detect memory issues in unsafe and C++ code. This requires the nightly version of the rust compiler, which can acquired using `rustup toolchain install nightly`. To show the symbols for the resulting stacktrace it is also convenient to install `llvm-symbolizer`, for example using `sudo apt install llvm` on Ubuntu. Afterwards, the tests can be executed with the address sanitizer enabled using `cargo +nightly xtask sanitizer`.

# Code Coverage

Code coverage can be automatically generated for the full workspace using a cargo task. The code coverage itself is generated by LLVM with source code instrumentation, which requires the preview tools to be installed `rustup component add llvm-tools-preview`, and [grcov](https://github.com/mozilla/grcov), which can be acquired using `cargo install grcov`. To execute the code coverage task use `cargo xtask coverage`. The resulting HTML code coverage report can be found in `target/coverage`. 

# Profiling

The `lpsreach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/lpsreach` can be profiled using any standard executable profiler, for example `Intel VTune` or `perf`. This compilation profile contains debugging information to show where time is being spent, but the code is optimised the same as in a release configuration.

Another useful technique for profiling is to generate a so-called `flamegraph`, which essentially takes the output of `perf` and produces a callgraph of time spent over time. These can be generated using the [flamegraph-rs](https://github.com/flamegraph-rs/flamegraph) tool, which can be acquired using `cargo install flamegraph`. Note that it relies on either `perf` or `dtrace` and as such is only supported on Linux and MacOS.

# Formatting

All source code should be formatted using `rustfmt`, which can installed using `rustup component add rustfmt`. Individual source files can then be formatted using `rustfmt <filename>`.

# Related Work

This library is fully inspired by the work on [mCRL2](https://github.com/mCRL2org/mCRL2).
