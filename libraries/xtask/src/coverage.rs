//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use fs_extra as fsx;
use glob::glob;
use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf}, error::Error,
};

use duct::cmd;

///
/// Remove a set of files given a glob
///
/// # Errors
/// Fails if listing or removal fails
///
fn clean_files(pattern: &str) -> Result<(), Box<dyn Error>> {
    let files: Result<Vec<PathBuf>, _> = glob(pattern)?.collect();
    files?.iter().try_for_each(remove_file)
}

///
/// Remove a single file
///
/// # Errors
/// Fails if removal fails
///
fn remove_file<P>(path: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    fsx::file::remove(path)?;
    Ok(())
}

///
/// Remove a directory with its contents
///
/// # Errors
/// Fails if removal fails
///
fn remove_dir<P>(path: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    fsx::dir::remove(path)?;
    Ok(())
}

///
/// Run coverage
///
/// # Errors
/// Fails if any command fails
///
pub fn coverage() -> Result<(), Box<dyn Error>> {
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