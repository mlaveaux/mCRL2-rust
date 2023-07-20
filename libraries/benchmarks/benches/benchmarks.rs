use criterion::{criterion_group, criterion_main};

mod operations;
mod rewrite;

use operations::*;
use rewrite::*;

criterion_group!(
    benches,
    criterion_benchmark_operations,
    criterion_benchmark_jitty,
    criterion_benchmark_sabre,
);
criterion_main!(benches);