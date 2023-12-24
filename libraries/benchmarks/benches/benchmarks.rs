use criterion::{criterion_group, criterion_main};
use rewriter::criterion_benchmark_jitty;

mod rewriter;

criterion_group!(
    benches,
    criterion_benchmark_jitty,
);
criterion_main!(benches);