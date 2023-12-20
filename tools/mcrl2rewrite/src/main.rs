use std::{rc::Rc, cell::RefCell};

use anyhow::Result as AnyResult;
use clap::Parser;
use env_logger;

use mcrl2::aterm::TermPool;
use mcrl2rewrite::{rewrite_data_spec, rewrite_rec, Rewriter};


#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A command line rewriting tool",
    long_about = "Can be used to parse and rewrite arbitrary mCRL2 data specifications and REC files"
)]
pub struct Cli {
    #[arg(long="rec")]
    rec: bool,

    #[arg(long="rewriter")]
    rewriter: Rewriter,

    #[arg(value_name = "FILE")]
    specification: String,

    #[arg(help="File containing the terms to be rewritten.")]
    terms: Option<String>,

}

fn main() -> AnyResult<()>
{
    env_logger::init();

    let cli = Cli::parse();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    if cli.rec {
        rewrite_rec(cli.rewriter, &cli.specification)?;
    } else {
        
        match &cli.terms {
            Some(expressions) => {
                rewrite_data_spec(tp.clone(), cli.rewriter, &cli.specification, expressions)?;
            }
            None => {
                
            }
        }
    }

    println!("pool: {}", tp.borrow());
    Ok(())
}
