use std::{cell::RefCell, fs::{self, File}, rc::Rc};
use std::io::Write;

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

#[derive(clap::Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A command line rewriting tool",
)]
pub(crate) enum Cli {
    Rewrite(RewriteArgs),
    Convert(ConvertArgs)
}

#[derive(clap::Args, Debug)]
#[command(about = "Rewrite mCRL2 data specifications and REC files")]
struct RewriteArgs {
    rewriter: Rewriter,

    #[arg(value_name = "SPEC")]
    specification: String,

    #[arg(help="File containing the terms to be rewritten.")]
    terms: Option<String>,

    #[arg(long="output", default_value_t=false, help="Print the rewritten term(s)")]
    output: bool,
}


#[derive(clap::Args, Debug)]
#[command(about = "Convert input rewrite system to the TRS format")]
struct ConvertArgs {
    #[arg(value_name = "SPEC")]
    specification: String,

    output: String,
}

fn main() -> AnyResult<()>
{
    env_logger::init();

    let cli = Cli::parse();
    let tp = Rc::new(RefCell::new(TermPool::new()));

    match cli {
        Cli::Rewrite(args) => {
            if args.specification.ends_with(".rec") {
                assert!(args.terms.is_none());
                rewrite_rec(args.rewriter, &args.specification, args.output)?;
            } else {        
                match &args.terms {
                    Some(terms) => {
                        rewrite_data_spec(tp.clone(), args.rewriter, &args.specification, terms, args.output)?;
                    }
                    None => {
                        warn!("No expressions given to rewrite!");
                    }
                }           
            }
        },
        Cli::Convert(args) => {
            // Read the data specification
            let data_spec_text = fs::read_to_string(args.specification)?;
            let data_spec = DataSpecification::new(&data_spec_text)?;

            let spec: RewriteSpecification = data_spec.into();
            
            let mut output = File::create(args.output)?;
            write!(output, "{}", TrsFormatter::new(&spec))?;
        }
    }
   
    info!("ATerm pool: {}", tp.borrow());

    #[cfg(feature = "measure-allocs")]
    info!("Allocations: {}",  A.number_of_allocations());
    
    Ok(())
}
