/// \returns A vector of strings where prefix is prepended to every string slice in paths.
fn add_prefix(prefix: String, paths: &[&str]) -> Vec<String>
{
  let mut result: Vec<String> = vec![];

  for path in paths
  {
    result.push(prefix.clone() + path);
  }

  return result;
}

fn main() {

  // The mCRL2 source files that we need to build for our Rust wrapper.
  let atermpp_source_files = [
    "aterm_implementation.cpp",
    "aterm_io_binary.cpp",
    "aterm_io_text.cpp",
    "function_symbol.cpp",
    "function_symbol_pool.cpp"
  ];

  let lps_source_files = [
    "lps.cpp",
    "lps_io.cpp",
    "tools.cpp",
    "linearise.cpp",
    "lpsparunfoldlib.cpp",
    "next_state_generator.cpp",
    "symbolic_lts_io.cpp"
  ];

  let data_source_files = [    
    "data.cpp",
    "data_io.cpp",
    "data_specification.cpp",
    "typecheck.cpp",
    "detail/prover/smt_lib_solver.cpp",
    "detail/rewrite/jitty.cpp",
    "detail/rewrite/rewrite.cpp",
    "detail/rewrite/strategy.cpp"
  ];

  let utilities_source_files = [
    "bitstream.cpp",
    "cache_metric.cpp",
    "command_line_interface.cpp",
    "logger.cpp",
    "text_utility.cpp",
    "toolset_version.cpp"
  ];

  let core_source_files = [
    "dparser.cpp",
    "core.cpp"
  ];

  let process_source_files = [
    "process.cpp"
  ];

  let dparser_source_files = [
    "arg.c",
    "parse.c",
    "scan.c",
    "dsymtab.c",
    "util.c",
    "read_binary.c",
    "dparse_tree.c"
  ];

  // Path to the mCRL2 location
  let mcrl2_path = String::from("../3rd-party/mCRL2/");
  let mcrl2_workarounds_path = String::from("../3rd-party/mCRL2-workarounds/");

  // These are the files for which we need to call cxxbuild to produce the bridge code.
  let mut build = cxx_build::bridges([ "src/lps.rs", "src/atermpp.rs" ]);

  // Additional files needed to compile the bridge, basically to build mCRL2 itself.
  build.cpp(true)
      .flag_if_supported("-std=c++17")
      .includes(add_prefix(mcrl2_path.clone(), &[
        "3rd-party/dparser/",
        "build/workarounds/msvc", // These are MSVC workarounds that mCRL2 relies on for compilation.  
        "libraries/atermpp/include",
        "libraries/core/include",
        "libraries/data/include",
        "libraries/lps/include",
        "libraries/process/include",
        "libraries/utilities/include",  
        ]))
      .include(mcrl2_workarounds_path.clone() + "include/")
      .files(add_prefix(mcrl2_path.clone() + "libraries/atermpp/source/", &atermpp_source_files))
      .files(add_prefix(mcrl2_path.clone() + "libraries/lps/source/", &lps_source_files))
      .files(add_prefix(mcrl2_path.clone() + "libraries/data/source/", &data_source_files))
      .files(add_prefix(mcrl2_path.clone() + "libraries/utilities/source/", &utilities_source_files))
      .files(add_prefix(mcrl2_path.clone() + "libraries/core/source/", &core_source_files))
      .files(add_prefix(mcrl2_path.clone() + "libraries/process/source/", &process_source_files))      
      .files(add_prefix(mcrl2_path.clone() + "3rd-party/dparser/", &dparser_source_files))
      .include("../3rd-party/boost-include-only/")
      .file(mcrl2_workarounds_path.clone() + "mcrl2_syntax.c"); // This is to avoid generating the dparser grammer.

  // Disable assertions and other checks.
  build.define("DNDEBUG", "1");
  //build.define("MCRL2_THREAD_SAFE", "1");

  // Add MSVC specific flags and definitions.
  build.flag_if_supported("/std:c++17")
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

  build.compile("mcrl2-rust");

  // Only run this build script if the bridge changes.
  println!("cargo:rerun-if-changed=src/atermpp.rs");
  println!("cargo:rerun-if-changed=src/lib.rs");
  println!("cargo:rerun-if-changed=src/lps.rs");
  println!("cargo:rerun-if-changed=atermpp/aterm.h");
}