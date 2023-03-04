
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mcrl2_benchmarks::load_case;

pub fn criterion_benchmark(c: &mut Criterion) 
{          
    let (mut rewriter, expressions) = load_case("cases/add16", 100);

    c.bench_function("add16", 
    |bencher| 
    {        
        bencher.iter(
        || {          
            for (i, expression) in expressions.iter().enumerate()
            {
                black_box(rewriter.rewrite(expression));
            }
        })
    });
}

 	
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);