use criterion::{criterion_group, criterion_main};
use rewriter::{criterion_benchmark_jitty, criterion_benchmark_set_automaton};

mod rewriter;

criterion_group!(
    benches,
    criterion_benchmark_jitty,
    criterion_benchmark_set_automaton,
);
criterion_main!(benches);