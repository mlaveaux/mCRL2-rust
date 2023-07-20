use std::{cell::RefCell, rc::Rc};

use criterion::{black_box, Criterion};

use ahash::AHashSet;

use mcrl2_sys::atermpp::{ATerm, TermPool};
use mcrl2_sys::data::{DataSpecification, JittyRewriter};
use rec_tests::load_REC_from_strings;
use sabre::set_automaton::SetAutomaton;
use sabre::utilities::to_data_expression;
use sabre::{InnermostRewriter, RewriteEngine, RewriteSpecification, SabreRewriter};

use std::fs::{self, File};
use std::io::{BufRead, BufReader};


/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(tp: &mut TermPool, name: &str, max_number_expressions: usize) -> (DataSpecification, Vec<ATerm>) {
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
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let (data_spec, expressions) = load_case(&mut tp.borrow_mut(), "cases/add16", 100);

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

pub fn criterion_benchmark_sabre(c: &mut Criterion) {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let cases = vec![(
        vec![
            include_str!("../../rec-tests/tests/REC_files/factorial7.rec"),
            include_str!("../../rec-tests/tests/REC_files/factorial.rec"),
        ],
        "factorial7",
    )];

    for (input, name) in cases {
        let (spec, terms): (RewriteSpecification, Vec<ATerm>) = {
            let (syntax_spec, syntax_terms) = load_REC_from_strings(&mut tp.borrow_mut(), &input);
            let result = syntax_spec.to_rewrite_spec(&mut tp.borrow_mut());
            (
                result,
                syntax_terms
                    .iter()
                    .map(|term| to_data_expression(&mut tp.borrow_mut(), &term, &AHashSet::new()))
                    .collect(),
            )
        };

        // Benchmark the set automaton construction
        c.bench_function(&format!("construct set automaton {:?}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(SetAutomaton::new(&spec, false, false));
            });
        });

        let mut inner = InnermostRewriter::new(tp.clone(), &spec);
        c.bench_function(&format!("innermost benchmark {:?}", name), |bencher| {
            for term in &terms {
                bencher.iter(|| {
                    let _ = black_box(inner.rewrite(term.clone()));
                });
            }
        });
        
        let mut sabre = SabreRewriter::new(tp.clone(), &spec);
        c.bench_function(&format!("sabre benchmark {:?}", name), |bencher| {
            for term in &terms {
                bencher.iter(|| {
                    let _ = black_box(sabre.rewrite(term.clone()));
                });
            }
        });
    }
}