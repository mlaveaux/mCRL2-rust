use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mcrl2_rust::atermpp::ATerm;
use mcrl2_rust::{data::DataSpecification, data::JittyRewriter};

/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(name: &str) -> (JittyRewriter, Vec<ATerm>)
{
    let path = String::from(name) + ".dataspec";
    let path_expressions = String::from(name) + ".expressions";

    // Read the data specification
    let data_spec_text = fs::read_to_string(&path).expect("failed to read file");
    let data_spec = DataSpecification::new(&data_spec_text);

    // Create a jitty rewriter;
    let rewriter = JittyRewriter::new(&data_spec);

    // Convert to the rewrite rules that sabre expects.
    //let rewriter = SabreRewriter::new(x);

    // Open the file in read-only mode.
    let file = File::open(path_expressions).unwrap();

    // Read the file line by line, and return an iterator of the lines of the file.
    let expressions: Vec<ATerm> = BufReader::new(file).lines().map(|x| data_spec.parse(&x.unwrap())).collect();

    (rewriter, expressions)
}

pub fn criterion_benchmark(c: &mut Criterion) 
{          
    c.bench_function("add16", 
    |bencher| 
    {        
        let (mut rewriter, expressions) = load_case("cases/add16");

        /*bencher.iter(
        || {            
            let mut amount = 0;
            for expression in expressions.iter()
            {
                return;
                if amount == 1 {
                    return;
                }

                let result = black_box(rewriter.rewrite(expression));
                println!("{}", result);
                amount += 1;
            }
        })*/
    });
}

 	
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);