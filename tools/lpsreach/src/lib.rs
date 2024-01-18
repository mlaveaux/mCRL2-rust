extern crate ldd;

mod sylvan_io;

use std::error::Error;

use clap::Parser;

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
    // Initialize the library.
    let mut storage = ldd::Storage::new();
    storage.enable_performance_metrics(true);

    let (initial_state, transitions) = sylvan_io::load_model(&mut storage, &config.filename)?;

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
}