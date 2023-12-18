use std::{cell::RefCell, rc::Rc};

use divan::{black_box, AllocProfiler};
use env_logger;

use mcrl2::aterm::{ATerm, TermPool};
use mcrl2::data::DataSpecification;
use sabre::{InnermostRewriter, RewriteEngine};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    env_logger::init();
    divan::main();
}

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

#[divan::bench]
pub fn benchmark_factorial9_innermost(bencher: divan::Bencher) {
    let tp = Rc::new(RefCell::new(TermPool::new()));

    let (data_spec, expressions) = (include_str!("../../../examples/REC/mcrl2/factorial9.dataspec"), include_str!("../../../examples/REC/mcrl2/factorial9.expressions"));
    let (data_spec, expressions) = load_case(&mut tp.borrow_mut(), data_spec, expressions, 1);

    let mut inner = InnermostRewriter::new(tp, &data_spec.into());

    bencher.bench_local(|| {
        let _ = black_box(inner.rewrite(expressions[0].clone()));
    });
}
