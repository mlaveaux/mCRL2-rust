use std::{cell::RefCell, rc::Rc};

use criterion::{black_box, Criterion};

use ahash::AHashSet;

use mcrl2::aterm::{ATerm, TermPool};
use mcrl2::data::{DataSpecification, JittyRewriter};
use rec_tests::load_REC_from_strings;
use sabre::set_automaton::SetAutomaton;
use sabre::utilities::to_data_expression;
use sabre::{InnermostRewriter, RewriteEngine, RewriteSpecification, SabreRewriter};

use std::fs::{self, File};
use std::io::{BufRead, BufReader};

/// Creates a rewriter and a vector of ATerm expressions for the given case.
pub fn load_case(
    _: &mut TermPool,
    data_spec_text: &str,
    expressions_text: &str,
    max_number_expressions: usize,
) -> (DataSpecification, Vec<ATerm>) {
    // Read the data specification
    let data_spec = DataSpecification::new(&data_spec_text);

    // Read the file line by line, and return an iterator of the lines of the file.
    let expressions: Vec<ATerm> = expressions_text
        .lines()
        .take(max_number_expressions)
        .map(|x| data_spec.parse(x))
        .collect();

    (data_spec, expressions)
}

pub fn criterion_benchmark_jitty(c: &mut Criterion) {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let (data_spec, expressions) = load_case(
        &mut tp.borrow_mut(),
        include_str!("../cases/add16.dataspec"),
        include_str!("../cases/add16.expressions"),
        100,
    );

    // Create a jitty rewriter;
    let mut jitty_rewriter = JittyRewriter::new(&data_spec.clone());

    // let tp = Rc::new(RefCell::new(TermPool::new()));
    // let mut sabre_rewriter = SabreRewriter::new(tp, &data_spec.into());

    c.bench_function("add16 jitty", |bencher| {
        bencher.iter(|| {
            for expression in expressions.iter() {
                black_box(jitty_rewriter.rewrite(expression));
            }
        })
    });

    // c.bench_function("add16 sabre", |bencher| {
    //     bencher.iter(|| {
    //         for expression in expressions.iter() {
    //             black_box(sabre_rewriter.rewrite(expression.clone()));
    //         }
    //     })
    // });
}

pub fn criterion_benchmark_sabre(c: &mut Criterion) {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let cases = vec![(
        vec![
            include_str!("../../rec-tests/REC_files/factorial7.rec"),
            include_str!("../../rec-tests/REC_files/factorial.rec"),
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
                let _ = black_box(SetAutomaton::new(&mut tp.borrow_mut(), &spec, false, false));
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
