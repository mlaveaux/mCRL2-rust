use std::fs::File;

use anyhow::Result;
use clap::Parser;

use io::aut::read_aut;

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A lts conversion tool",
)]
pub struct Cli {
    #[arg(value_name = "FILE")]
    labelled_transition_system: String,

}

fn main() -> Result<()>
{
    let cli = Cli::parse();

    let file = File::open(cli.labelled_transition_system)?;

    let _lts = read_aut(file).unwrap();

    Ok(())
}
