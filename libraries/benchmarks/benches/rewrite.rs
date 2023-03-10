use std::{cell::RefCell, rc::Rc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mcrl2_benchmarks::load_case;
use mcrl2_rust::{atermpp::TermPool, data::JittyRewriter};

pub fn criterion_benchmark(c: &mut Criterion) {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
