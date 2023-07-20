//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use anyhow::Result as AnyResult;
use std::env;

mod coverage;
mod sanitizer;

use coverage::coverage;
use sanitizer::sanitizer;

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
        Some("coverage") => {
            coverage()?
        },
        Some("sanitizer") => {
            sanitizer(other_arguments)?
        },
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    println!("Unknown task");
}