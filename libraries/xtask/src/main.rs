//!
//! xtask building block operations such as copy, remove, confirm, and more
//!
//!

use std::{env, error::Error};

mod coverage;
mod sanitizer;

use coverage::coverage;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
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
            println!("Unknown task {}", x);
        },
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    println!("Unknown task");
}