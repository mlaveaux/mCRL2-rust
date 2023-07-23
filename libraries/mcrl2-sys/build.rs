use cc::Build;

/// \returns A vector of strings where prefix is prepended to every string slice in paths.
fn add_prefix(prefix: String, paths: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    for path in paths {
        result.push(prefix.clone() + path);
    }

    result
}

/// Add MSVC specific flags and definitions.
#[cfg(windows)]
fn add_platform_flags(build: &mut Build, mcrl2_path: String) {
    build
        .include(mcrl2_path + "build/workarounds/msvc") // These are MSVC workarounds that mCRL2 relies on for compilation.
        .flag_if_supported("/EHs")
        .flag_if_supported("/bigobj")
        .flag_if_supported("/W3")
        .flag_if_supported("/MP")
        .flag_if_supported("/permissive-")
        .define("WIN32", "1")
        .define("NOMINMAX", "1")
        .define("_USE_MATH_DEFINES", "1")
        .define("_CRT_SECURE_CPP_OVERLOAD_STANDARD_NAMES", "1")
        .define("_CRT_SECURE_NO_WARNINGS", "1")
        .define("BOOST_ALL_NO_LIB", "1");
}

/// Add Linux specific flags and definitions.
#[cfg(unix)]
fn add_platform_flags(build: &mut Build, _mcrl2_path: String) {
    build
        .flag_if_supported("-Wall")
        .flag_if_supported("-pipe")
        .flag_if_supported("-pedantic");
}

#[cfg(windows)]
fn add_cpp_flags(build: &mut Build) {
    build.flag_if_supported("/std:c++17");
}

#[cfg(unix)]
fn add_cpp_flags(build: &mut Build) {
    build.flag_if_supported("-std=c++17");
}

fn main() {
    // The mCRL2 source files that we need to build for our Rust wrapper.
    let atermpp_source_files = [
        "aterm_implementation.cpp",
        "aterm_io_binary.cpp",
        "aterm_io_text.cpp",
        "function_symbol.cpp",
        "function_symbol_pool.cpp",
    ];

    let lps_source_files = [
        "lps.cpp",
        "lps_io.cpp",
        //"tools.cpp",
        //"linearise.cpp",
        //"lpsparunfoldlib.cpp",
        //"next_state_generator.cpp",
        //"symbolic_lts_io.cpp",
    ];

    let data_source_files = [
        "data.cpp",
        "data_io.cpp",
        "data_specification.cpp",
        "typecheck.cpp",
        //"detail/prover/smt_lib_solver.cpp",
        "detail/rewrite/jitty.cpp",
        "detail/rewrite/rewrite.cpp",
        "detail/rewrite/strategy.cpp",
    ];

    let utilities_source_files = [
        "bitstream.cpp",
        "cache_metric.cpp",
        //"command_line_interface.cpp",
        "text_utility.cpp",
        //"toolset_version.cpp",
    ];

    let core_source_files = ["dparser.cpp", "core.cpp"];

    let process_source_files = ["process.cpp"];

    let dparser_source_files = [
        "arg.c",
        "parse.c",
        "scan.c",
        "dsymtab.c",
        "util.c",
        "read_binary.c",
        "dparse_tree.c",
    ];

    // Path to the mCRL2 location
    let mcrl2_path = String::from("../../3rd-party/mCRL2/");
    let mcrl2_workarounds_path = String::from("../../3rd-party/mCRL2-workarounds/");

    // Build dparser separately since it's a C library.
    let mut build_dparser = cc::Build::new();
    build_dparser
        .include(mcrl2_path.clone() + "3rd-party/dparser/")
        .files(add_prefix(
            mcrl2_path.clone() + "3rd-party/dparser/",
            &dparser_source_files,
        ));

    add_platform_flags(&mut build_dparser, mcrl2_path.clone());
    build_dparser.compile("dparser");

    // These are the files for which we need to call cxxbuild to produce the bridge code.
    let mut build = cxx_build::bridges(["src/atermpp.rs", "src/data.rs", "src/lps.rs"]);

    // Additional files needed to compile the bridge, basically to build mCRL2 itself.
    build
        .cpp(true)
        .define("MCRL2_NO_RECURSIVE_SOUNDNESS_CHECKS", "1") // These checks overflow the stack, and are extremely slow.
        .define("LPS_NO_RECURSIVE_SOUNDNESS_CHECKS", "1")
        .includes(add_prefix(
            mcrl2_path.clone(),
            &[
                "3rd-party/dparser/",
                "libraries/atermpp/include",
                "libraries/core/include",
                "libraries/data/include",
                "libraries/lps/include",
                "libraries/process/include",
                "libraries/utilities/include",
            ],
        ))
        .include(mcrl2_workarounds_path.clone() + "include/")
        .include("../../3rd-party/boost-include-only/")
        .include("dparser")
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/atermpp/source/",
            &atermpp_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/lps/source/",
            &lps_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/data/source/",
            &data_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/utilities/source/",
            &utilities_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/core/source/",
            &core_source_files,
        ))
        .files(add_prefix(
            mcrl2_path.clone() + "libraries/process/source/",
            &process_source_files,
        ))
        .file(mcrl2_workarounds_path + "mcrl2_syntax.c"); // This is to avoid generating the dparser grammer.

    // Disable assertions and other checks in release mode.
    let profile = std::env::var("PROFILE").expect("cargo should always set this variable");
    match profile.as_str() {
        "debug" => (),
        "release" => {
            build.define("NDEBUG", "1");
        }
        _ => (),
    }

    build.define("MCRL2_THREAD_SAFE", "1");

    add_platform_flags(&mut build, mcrl2_path);
    add_cpp_flags(&mut build);

    build.compile("mcrl2-sys");

    // It seems that build changes are detected properly automatically, otherwise adapt this.
    cargo_emit::rerun_if_changed!("");
}
