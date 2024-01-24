use std::{rc::Rc, cell::RefCell};

use anyhow::Result as AnyResult;
use clap::Parser;

use log::info;
use mcrl2::aterm::TermPool;
use mcrl2rewrite::{rewrite_data_spec, rewrite_rec, Rewriter};

mod counting_allocator;

#[cfg(feature = "measure-allocs")]
#[global_allocator]
static A: counting_allocator::AllocCounter = counting_allocator::AllocCounter;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "measure-allocs"))]
#[global_allocator] 
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

    #[arg(long="output", default_value_t=false, help="Print the resulting term(s)")]
    output: bool

}

fn main() -> AnyResult<()>
{
    env_logger::init();

    let cli = Cli::parse();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    if cli.rec {
        assert!(cli.terms.is_none());
        rewrite_rec(cli.rewriter, &cli.specification, cli.output)?;
    } else {        
        match &cli.terms {
            Some(terms) => {
                rewrite_data_spec(tp.clone(), cli.rewriter, &cli.specification, terms, cli.output)?;
            }
            None => {
                panic!("For mCRL2 specifications the terms argument is mandatory.");
            }
        }
    }

    info!("ATerm pool: {}", tp.borrow());
    #[cfg(feature = "measure-allocs")]
    info!("Allocations: {}",  A.number_of_allocations());
    
    Ok(())
}
