//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use anyhow::Result as AnyResult;
use fs_extra as fsx;
use glob::glob;
use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
};

pub use duct::cmd;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> AnyResult<()> {
    let mut args = env::args();

    // Ignore the first argument (which should be xtask)
    args.next();

    // The name of the task
    let task = args.next();

    // Take the other parameters.
    let other_arguments: Vec<String> = args.collect();

    match task.as_deref() {
        Some("coverage") => coverage()?,
        Some("sanitizer") => sanitizer(other_arguments)?,
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    println!("Unknown task");
}

///
/// Remove a set of files given a glob
///
/// # Errors
/// Fails if listing or removal fails
///
pub fn clean_files(pattern: &str) -> AnyResult<()> {
    let files: Result<Vec<PathBuf>, _> = glob(pattern)?.collect();
    files?.iter().try_for_each(remove_file)
}

///
/// Remove a single file
///
/// # Errors
/// Fails if removal fails
///
pub fn remove_file<P>(path: P) -> AnyResult<()>
where
    P: AsRef<Path>,
{
    fsx::file::remove(path).map_err(anyhow::Error::msg)
}

///
/// Remove a directory with its contents
///
/// # Errors
/// Fails if removal fails
///
pub fn remove_dir<P>(path: P) -> AnyResult<()>
where
    P: AsRef<Path>,
{
    fsx::dir::remove(path).map_err(anyhow::Error::msg)
}

///
/// Run coverage
///
/// # Errors
/// Fails if any command fails
///
fn coverage() -> AnyResult<()> {
    remove_dir("target/coverage")?;
    create_dir_all("target/coverage")?;

    println!("=== running coverage ===");

    // The path from which cargo is called.
    let mut base_directory = env::current_dir().unwrap();
    base_directory.push("target");
    base_directory.push("coverage");

    let mut prof_directory = base_directory.clone();
    prof_directory.push("cargo-test-%p-%m.profraw");

    cmd!("cargo", "test")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", prof_directory)
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let (fmt, file) = ("html", "target/coverage/html");
    cmd!(
        "grcov",
        base_directory,
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        fmt,
        "--branch",
        "--ignore-not-existing",
        "-o",
        file,
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;
    println!("ok.");
    
    println!("report location: {file}");

    Ok(())
}

///
/// Run the tests with the address sanitizer enabled to detect memory issues in unsafe and C++ code.
///
/// This only works under Linux currently and requires the nightly toolchain
///
fn sanitizer(cargo_arguments: Vec<String>) -> AnyResult<()> {
    let mut arguments: Vec<String> = vec![
        "test".to_string(),
        "-Zbuild-std".to_string(),
        "--target".to_string(),
        "x86_64-unknown-linux-gnu".to_string(),
    ];

    arguments.extend(cargo_arguments.into_iter());

    cmd("cargo", arguments)
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Zsanitizer=address")
        .run()?;
    println!("ok.");

    Ok(())
}
