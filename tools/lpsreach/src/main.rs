use std::{process::ExitCode, error::Error};

use clap::Parser;
use lpsreach::{run, Config};

fn main() -> Result<ExitCode, Box<dyn Error>>
{
    env_logger::init();
    
    let config = Config::parse();

    let num_of_states = run(&config)?;
    println!("There are {num_of_states} states");
    
    Ok(ExitCode::SUCCESS)
}