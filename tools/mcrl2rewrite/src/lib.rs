use std::error::Error;
use std::fs;

use mcrl2_rust::data::DataSpecification;

/// Performs state space exploration of the given model and returns the number of states.
pub fn run(config: &Config) -> Result<usize, Box<dyn Error>>
{
    // Read the data specification
    let data_spec_text = fs::read_to_string(&config.filename_dataspec).expect("failed to read file");
    let data_spec = DataSpecification::from(&data_spec_text);

    // Convert to the rewrite rules that sabre expects.
    
    Ok(0)
}

pub struct Config
{
  pub filename_dataspec: String,
  pub filename_expressions: String
}

impl Config
{
    /// Parses the provided arguments and fills in the configuration.
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str>
    {
        args.next(); // The first argument is the executable's location.

        let filename_dataspec = match args.next() {
            Some(arg) => arg,
            None => return Err("Requires data specification filename")
        };

        let filename_expressions = match args.next() {
            Some(arg) => arg,
            None => return Err("Requires data expressions filename")
        };

        Ok(Config { filename_dataspec, filename_expressions })
    }
}