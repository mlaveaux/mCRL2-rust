use anyhow::Result as AnyResult;
use clap::Parser;

use mcrl2rewrite::{rewrite_data_spec, rewrite_rec};

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A command line rewriting tool",
    long_about = "Can be used to parse and rewrite arbitrary mCRL2 data specifications and REC files"
)]
pub struct Cli {
    #[arg(long)]
    rec: bool,

    #[arg(value_name = "FILE")]
    specification: String,

    #[arg()]
    term: String,
}

fn main() -> AnyResult<()>
{
    let cli = Cli::parse();

    if cli.rec { 
        rewrite_rec(&cli.specification, &cli.term)     
    } else {
        rewrite_data_spec(&cli.specification, "") 
    }

}
