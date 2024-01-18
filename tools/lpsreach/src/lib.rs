extern crate ldd;

mod sylvan_io;

use std::cell::RefCell;
use std::rc::Rc;
use std::{error::Error, path::Path};
use std::fs::File;

use clap::Parser;
use log::info;
use mcrl2::aterm::TermPool;
use mcrl2::lps::LinearProcessSpecification;
use sabre::InnermostRewriter;

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A command line symbolic reachability tool",
)]
pub struct Config
{
  #[arg(value_name = "FILE")]
  pub filename: String,
}

/// Performs state space exploration of the given model and returns the number of states.
pub fn run(config: &Config) -> Result<usize, Box<dyn Error>>
{
    let file = Path::new(&config.filename);
    if file.extension().unwrap() == ".ldd" {
        info!("Exploring Sylvan model...");
    
        // Initialize the library.
        let mut storage = ldd::Storage::new();
        storage.enable_performance_metrics(true);

        let (initial_state, transitions) = sylvan_io::load_model(&mut storage, &mut File::open(file)?)?;

        let mut todo = initial_state.clone();
        let mut states = initial_state; // The state space.
        let mut iteration = 0;

        while todo != *storage.empty_set()
        {
            let mut todo1 = storage.empty_set().clone();
            for transition in transitions.iter()
            {
                let result = ldd::relational_product(&mut storage, &todo, &transition.relation, &transition.meta);
                todo1 = ldd::union(&mut storage, &todo1, &result);
            }

            todo = ldd::minus(&mut storage, &todo1, &states);
            states = ldd::union(&mut storage, &states, &todo);

            eprintln!("iteration {}", iteration);
            iteration += 1;
        }

        let num_of_states = ldd::len(&mut storage, &states);
        println!("The model has {} states", num_of_states);
        Ok(num_of_states)
    } else {
        info!("Exploring mCRL2 model...");

        let lps = LinearProcessSpecification::read(&file.to_string_lossy())?;
        let mut tp = Rc::new(RefCell::new(TermPool::new()));

        let rewriter = InnermostRewriter::new(tp, &lps.data_specification().into());
        
        Ok(0)
    }

}