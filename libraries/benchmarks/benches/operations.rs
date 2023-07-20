use criterion::{black_box, Criterion};
use ldd::*;
use rand::Rng;
use std::collections::HashSet;

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(storage: &mut Storage, iter: I) -> Ldd
    where I: Iterator<Item = &'a Vec<Value>>
{
    let mut result = storage.empty_set().clone();

    for vector in iter
    {
        let single = singleton(storage, vector);
        result = union(storage, result.borrow(), single.borrow());
    }

    result
}

pub fn criterion_benchmark_operations(c: &mut Criterion) 
{      
    c.bench_function("union 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();

            bencher.iter(
            || {
                let set_a = random_vector_set(1000, 10, 10);
                let set_b = random_vector_set(1000, 10, 10);
            
                let a = from_iter(&mut storage, set_a.iter());
                let b = from_iter(&mut storage, set_b.iter());
            
                black_box(union(&mut storage, a.borrow(), b.borrow()));
            })
        });

        
    c.bench_function("minus 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();
            
            bencher.iter(
            || {
                let set_a = random_vector_set(1000, 10, 10);
                let set_b = random_vector_set(1000, 10, 10);
            
                let a = from_iter(&mut storage, set_a.iter());
                let b = from_iter(&mut storage, set_b.iter());
            
                black_box(minus(&mut storage, a.borrow(), b.borrow()));
            })
        });
        

    c.bench_function("relational_product 1000", 
    |bencher| 
        {
            let mut storage = Storage::new();

            bencher.iter(
            || {
                let set = random_vector_set(1000, 10, 10);        
                let relation = random_vector_set(32, 4, 10);

                // Pick arbitrary read and write parameters in order.
                let read_proj = random_sorted_vector(2,9);
                let write_proj = random_sorted_vector(2,9);

                // Compute LDD result.
                let ldd = from_iter(&mut storage, set.iter());
                let rel = from_iter(&mut storage, relation.iter());

                let meta = compute_meta(&mut storage, &read_proj, &write_proj);
                black_box(relational_product(&mut storage, ldd.borrow(), rel.borrow(), meta.borrow()));
            })
        });
}