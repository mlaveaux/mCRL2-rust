use std::{hash::Hash, collections::HashSet};

use rand::{Rng, distributions::uniform::SampleUniform};

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_vector<T: Copy + Default + SampleUniform + PartialOrd>(length: usize, max_value: T) -> Vec<T> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<T> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(T::default()..max_value));
    }

    vector
}

/// Returns a sorted vector of the given length with unique u64 values (from 0..max_value).
pub fn random_sorted_vector(length: usize, max_value: u32) -> Vec<u32> 
{
    use rand::prelude::IteratorRandom;

    let mut rng = rand::thread_rng(); 
    let mut result = (u32::default()..max_value).choose_multiple(&mut rng, length);
    result.sort();
    result
}

/// Returns a set of 'amount' vectors where every vector has the given length.
pub fn random_vector_set<T: Copy + Default + Eq + Hash + SampleUniform + PartialOrd>(amount: usize, length: usize, max_value: T) ->  HashSet<Vec<T>>
{
    let mut result: HashSet<Vec<T>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length, max_value));
    }

    result
}
