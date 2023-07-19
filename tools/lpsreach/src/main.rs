use std::env;
use std::process;

use reach::{run, Config};

fn main()
{
    let config = Config::new(env::args()).unwrap_or_else(
        |err| 
        { 
            eprintln!("Problem parsing input: {}", err); 
            process::exit(-1); 
        }
    );

    if let Err(err) = run(&config)
    {
        eprintln!("Problem parsing input: {}", err); 
        process::exit(-1);
    }
}