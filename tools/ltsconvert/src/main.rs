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
    env_logger::init();

    let cli = Cli::parse();
    let file = File::open(cli.labelled_transition_system)?;
    let _lts = read_aut(file).unwrap();

    // Check if the LTS is deterministic.
    // let mut deterministic = false;
    // for state in &lts.states {
    //     for (label, outgoing) in &state.outgoing {
    //         if state.outgoing.iter().m

    //     }

    // }

    Ok(())
}
