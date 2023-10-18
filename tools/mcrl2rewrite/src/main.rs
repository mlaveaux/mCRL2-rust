use std::{rc::Rc, cell::RefCell};

use anyhow::Result as AnyResult;
use clap::Parser;

use mcrl2::atermpp::TermPool;
use mcrl2rewrite::{rewrite_data_spec, rewrite_rec};

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A command line rewriting tool",
    long_about = "Can be used to parse and rewrite arbitrary mCRL2 data specifications and REC files"
)]
pub struct Cli {
    #[arg(long="rec")]
    rec: bool,

    #[arg(value_name = "FILE")]
    specification: String,

    #[arg()]
    term: String,

}

fn main() -> AnyResult<()>
{
    let cli = Cli::parse();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    if cli.rec { 
        rewrite_rec(&cli.specification, &cli.term)     
    } else {
        rewrite_data_spec(tp, &cli.specification, &cli.term) 
    }
}
