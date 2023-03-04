//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use anyhow::Result as AnyResult;
use dialoguer::{theme::ColorfulTheme, Confirm};
use fs_extra as fsx;
use fsx::dir::CopyOptions;
use glob::glob;
use std::{path::{Path, PathBuf}, fs::create_dir_all, env};

pub use duct::cmd;

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
/// Check if path exists
///
pub fn exists<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    std::path::Path::exists(path.as_ref())
}

///
/// Copy entire folder contents
///
/// # Errors
/// Fails if file operations fail
///
pub fn copy_contents<P, Q>(from: P, to: Q, overwrite: bool) -> AnyResult<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut opts = CopyOptions::new();
    opts.content_only = true;
    opts.overwrite = overwrite;
    fsx::dir::copy(from, to, &opts).map_err(anyhow::Error::msg)
}

///
/// Prompt the user to confirm an action
///
/// # Panics
/// Panics if input interaction fails
///
pub fn confirm(question: &str) -> bool 
{
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact()
        .unwrap()
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> AnyResult<()> {
    let task = env::args().nth(1);

    match task.as_deref() {
        Some("coverage") => coverage(false)?,
        _ => print_help(),
    }

    Ok(())
}

fn print_help()
{
    println!("Unknown task");
}

///
/// Run coverage
///
/// # Errors
/// Fails if any command fails
///
fn coverage(devmode: bool) -> AnyResult<()> 
{
    remove_dir("coverage")?;
    create_dir_all("coverage")?;

    println!("=== running coverage ===");
    cmd!("cargo", "test")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let (fmt, file) = if devmode {
        ("html", "coverage/html")
    } else {
        ("lcov", "coverage/tests.lcov")
    };
    cmd!(
        "grcov",
        ".",
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        fmt,
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
        "--ignore",
        "xtask/*",
        "--ignore",
        "*/src/tests/*",
        "-o",
        file,
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;
    println!("ok.");
    if devmode {
        if confirm("open report folder?") {
            cmd!("open", file).run()?;
        } else {
            println!("report location: {file}");
        }
    }

    Ok(())
}