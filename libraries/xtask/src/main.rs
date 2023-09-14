//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use std::{env, error::Error, process::{ExitCode, ExitStatus}};

mod coverage;
mod sanitizer;

use coverage::coverage;

fn main() -> Result<ExitCode, Box<dyn Error>> {
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
        Some("address-sanitizer") => {
            sanitizer::address_sanitizer(other_arguments)?
        },
        Some("thread-sanitizer") => {
            sanitizer::thread_sanitizer(other_arguments)?
        },
        Some(x) => {
            println!("Unknown task {x}");
            println!();
            print_help();
            return Ok(ExitCode::FAILURE)
        },
        _ => print_help(),
    }

    Ok(ExitCode::SUCCESS)
}

fn print_help() {
    println!("Available tasks: coverage, address-sanitizer, thread-sanitizer");
}