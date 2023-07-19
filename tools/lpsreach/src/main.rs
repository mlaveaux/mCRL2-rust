use std::{process::ExitCode};

use anyhow::Result as AnyResult;
use clap::Parser;
use reach::{run, Config};

fn main() -> AnyResult<ExitCode>
{
    let config = Config::parse();

    run(&config);

    Ok(0.into())
}