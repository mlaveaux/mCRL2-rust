use std::hint::black_box;
use std::{cell::RefCell, rc::Rc};

use criterion::Criterion;

use mcrl2::aterm::{ATerm, TermPool};
use mcrl2::data::{DataSpecification, JittyRewriter};
use sabre::{InnermostRewriter, RewriteEngine};


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

    for (name, data_spec, expressions) in [
        ("hanoi8", include_str!("../../../examples/REC/mcrl2/hanoi8.dataspec"), include_str!("../../../examples/REC/mcrl2/hanoi8.expressions")),
        ("add8", include_str!("../../../examples/REC/mcrl2/add8.dataspec"), include_str!("../../../examples/REC/mcrl2/add8.expressions")),
    ] {
        let (data_spec, expressions) = load_case(&mut tp.borrow_mut(), data_spec, expressions, 1);

        let mut jitty = JittyRewriter::new(&data_spec);
        let mut inner = InnermostRewriter::new(tp.clone(), &data_spec.into());

        c.bench_function(&format!("innermost {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(inner.rewrite(expressions[0].clone()));
            })
        });

        c.bench_function(&format!("jitty {}", name), |bencher| {
            bencher.iter(|| {
                let _ = black_box(jitty.rewrite(&expressions[0]));
            })
        });
    }
}
