use std::{cell::RefCell, rc::Rc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ahash::AHashSet;

use mcrl2_rust::atermpp::{ATerm, TermPool};
use sabre::set_automaton::SetAutomaton;
use sabre::{SabreRewriter, RewriteEngine, RewriteSpecification, InnermostRewriter};
use sabre::utilities::to_data_expression;
use rec_tests::load_REC_from_strings;
use mcrl2_rust::{data::JittyRewriter};

use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use mcrl2_rust::data::DataSpecification;

/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(name: &str, max_number_expressions: usize) -> (DataSpecification, Vec<ATerm>) {
    let path = String::from(name) + ".dataspec";
    let path_expressions = String::from(name) + ".expressions";

    // Read the data specification
    let data_spec_text = fs::read_to_string(path).expect("failed to read file");
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

pub fn criterion_benchmark_jitty(c: &mut Criterion) {
    let (data_spec, expressions) = load_case("cases/add16", 100);
    
    // Create a jitty rewriter;
    let mut jitty_rewriter = JittyRewriter::new(&data_spec);

    let _term_pool = Rc::new(RefCell::new(TermPool::new()));
    //let sabre_rewriter = SabreRewriter::new(term_pool, );

    c.bench_function("add16", |bencher| {
        bencher.iter(|| {
            for expression in expressions.iter() {
                black_box(jitty_rewriter.rewrite(expression));
            }
        })
    });
}

pub fn criterion_benchmark_sabre(c: &mut Criterion) 
{    
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let cases = vec![
        (vec![include_str!("../../rec-tests/tests/REC_files/benchexpr10.rec"), include_str!("../../rec-tests/tests/REC_files/asfsdfbenchmark.rec")], "benchexpr10")
    ];

    for (input, name) in cases {

            let (spec, terms): (RewriteSpecification, Vec<ATerm>) = { 
                let (syntax_spec, syntax_terms) = load_REC_from_strings(&input);
                let result = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());
                (result, syntax_terms.iter().map(|t| { 
                    let term = t.to_term(&mut tp.borrow_mut());
                    to_data_expression(&mut tp.borrow_mut(), &term, &AHashSet::new()) }).collect())
            };

            // Benchmark the set automaton construction                
            c.bench_function(&format!("construct set automaton {:?}", name), 
            |bencher| 
                {
                    bencher.iter(|| {
                        let _ = black_box(SetAutomaton::new(&spec, false, false));
                    });
                });

            let mut inner = InnermostRewriter::new(tp.clone(), &spec);
            
            c.bench_function(&format!("innermost benchmark {:?}", name),
            |bencher|
            {
                for term in &terms {
                    bencher.iter(|| {
                        let _ = black_box(inner.rewrite(term.clone()));
                    });
                }
            });
    }
}

criterion_group!(benches, criterion_benchmark_jitty, criterion_benchmark_sabre);
criterion_main!(benches);
