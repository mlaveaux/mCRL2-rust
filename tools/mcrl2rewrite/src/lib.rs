use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use mcrl2_rust::data::{DataSpecification, JittyRewriter};
//use sabre::SabreRewriter;

/// Performs state space exploration of the given model and returns the number of states.
pub fn run(config: &Config) -> Result<usize, Box<dyn Error>>
{
    // Read the data specification
    let data_spec_text = fs::read_to_string(&config.filename_dataspec).expect("failed to read file");
    let data_spec = DataSpecification::from(&data_spec_text);

    // Create a jitty rewriter;
    let mut rewriter = JittyRewriter::new(&data_spec);

    // Convert to the rewrite rules that sabre expects.
    //let rewriter = SabreRewriter::new(x);

    // Open the file in read-only mode.
    let file = File::open(&config.filename_expressions).unwrap(); 

    // Read the file line by line, and return an iterator of the lines of the file.
    for line in BufReader::new(file).lines()
    {
        println!("{}", rewriter.rewrite(&data_spec.parse(&line.unwrap())));
    }
    
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