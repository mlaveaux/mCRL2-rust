use std::{process::ExitCode, any::Any};

use anyhow::{Result as AnyResult, Ok};
use clap::Parser;
use reach::{run, Config};

fn main() -> AnyResult<ExitCode>
{
    let config = Config::parse();

    match run(&config) {
        Result::Ok(num_of_states) => {
            println!("There are {num_of_states} states");
            Ok(ExitCode::SUCCESS)
        },
        Result::Err(err) => {
            Ok(ExitCode::FAILURE)
        }
    }
}