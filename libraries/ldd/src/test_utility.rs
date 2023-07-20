use crate::{Ldd, Storage, operations::*, Value, iterators::*};

///! Functions in this module are only relevant for testing purposes.

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

/// Prints vectors included in left, but not in right. Returns true iff the difference is non-empty.
pub fn print_left(storage: &Storage, left: &Ldd, right: &Ldd) -> bool
{
    let mut result = true;

    for element in iter(storage, left)
    {
        if !element_of(storage, &element, right)
        {
            result = false;
            eprintln!("{:?}", element);
        }
    }

    result
}

/// Prints the differences in contained vectors between two LDDs.
pub fn print_differences(storage: &Storage, left: &Ldd, right: &Ldd)
{
    // eprintln!("Vectors contained in {:?}, but not in {:?}:", left, right);
    print_left(storage, left, right);
    
    // eprintln!("Vectors contained in {}, but not in {}:", right, left);
    print_left(storage, right, left);    
}

/// Returns project(vector, proj), see [project]. Requires proj to be sorted.
pub fn project_vector(vector: &[Value], proj: &[Value]) -> Vec<Value>
{
    let mut result = Vec::<Value>::new();
    for i in proj
    {
        result.push(vector[*i as usize]);
    }
    result
}