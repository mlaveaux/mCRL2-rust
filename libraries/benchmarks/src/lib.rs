use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use mcrl2_rust::atermpp::ATerm;
use mcrl2_rust::data::DataSpecification;

/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(name: &str, max_number_expressions: usize) -> (DataSpecification, Vec<ATerm>) {
    let path = String::from(name) + ".dataspec";
    let path_expressions = String::from(name) + ".expressions";

    // Read the data specification
    let data_spec_text = fs::read_to_string(&path).expect("failed to read file");
    let data_spec = DataSpecification::new(&data_spec_text);

    // Open the file in read-only mode.
    let file = File::open(path_expressions).unwrap();

    // Read the file line by line, and return an iterator of the lines of the file.
    let expressions: Vec<ATerm> = BufReader::new(file)
        .lines()
        .take(max_number_expressions)
        .map(|x| data_spec.parse(&x.unwrap()))
        .collect();

    (data_spec, expressions)
}
