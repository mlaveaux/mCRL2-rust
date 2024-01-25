use std::{cell::RefCell, fs, rc::Rc};

use anyhow::Result as AnyResult;
use clap::Parser;

use log::{info, warn};
use mcrl2::{aterm::TermPool, data::DataSpecification};
use rewrite::{rewrite_data_spec, rewrite_rec, Rewriter};
use sabre::RewriteSpecification;

use crate::trs_format::TrsFormatter;

mod trs_format;
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
    #[arg(long="rewriter")]
    rewriter: Option<Rewriter>,

    #[arg(value_name = "FILE")]
    specification: String,

    #[arg(help="File containing the terms to be rewritten.")]
    terms: Option<String>,

    #[arg(long="output", default_value_t=false, help="Print the resulting term(s)")]
    output: bool,
}

fn main() -> AnyResult<()>
{
    env_logger::init();

    let cli = Cli::parse();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    match cli.rewriter {
        Some(rewriter) => { 
            if cli.specification.ends_with(".rec") {
                assert!(cli.terms.is_none());
                rewrite_rec(rewriter, &cli.specification, cli.output)?;
            } else {        
                match &cli.terms {
                    Some(terms) => {
                        rewrite_data_spec(tp.clone(), rewriter, &cli.specification, terms, cli.output)?;
                    }
                    None => {
                        warn!("No expressions given to rewrite!");
                    }
                }           
            }
        },
        None => {            
            // Read the data specification
            let data_spec_text = fs::read_to_string(cli.specification)?;
            let data_spec = DataSpecification::new(&data_spec_text)?;

            let spec: RewriteSpecification = data_spec.into();
            
            println!("Specification: \n{}", spec);

            println!("{}", TrsFormatter::new(&spec))


        }
    }
   
    info!("ATerm pool: {}", tp.borrow());
    #[cfg(feature = "measure-allocs")]
    info!("Allocations: {}",  A.number_of_allocations());
    
    Ok(())
}
