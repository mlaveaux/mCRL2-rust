/// The protection set keeps track of nodes that should not be garbage
/// collected since they are being referenced by instances.
pub struct ProtectionSet<T> {
    roots: Vec<Entry<T>>, // The set of root active nodes.
    free: Option<usize>,
    number_of_insertions: u64,
}

enum Entry<T> {
    Filled(T),
    Free(usize),
}

impl<T> ProtectionSet<T> {
    pub fn new() -> Self {
        ProtectionSet {
            roots: vec![],
            free: None,
            number_of_insertions: 0,
        }
    }

    /// Returns the number of insertions into the protection set.
    pub fn number_of_insertions(&self) -> u64 {
        self.number_of_insertions
    }

    /// Returns maximum number of active instances.
    pub fn maximum_size(&self) -> usize {
        self.roots.capacity()
    }

    /// Returns an iterator over all root indices in the protection set.
    pub fn iter(&self) -> ProtSetIter<T> {
        ProtSetIter {
            current: 0,
            protection_set: self,
        }
    }

    /// Protect the given root node to prevent garbage collection.
    pub fn protect(&mut self, object: T) -> usize {
        self.number_of_insertions += 1;

        match self.free {
            Some(first) => {
                match &self.roots[first] {
                    Entry::Free(next) => {
                        if first == *next {
                            // The list is empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(*next);
                        }
                    }
                    Entry::Filled(_) => {
                        panic!("The free list should not point a filled entry");
                    }
                }

                self.roots[first] = Entry::Filled(object);
                first
            }
            None => {
                // If free list is empty insert new entry into roots.
                self.roots.push(Entry::Filled(object));
                self.roots.len() - 1
            }
        }
    }

    /// Remove protection from the given LDD node. Note that index must be the
    /// index returned by the [protect] call.
    pub fn unprotect(&mut self, index: usize) {
        match self.free {
            Some(next) => {
                self.roots[index] = Entry::Free(next);
            }
            None => {
                self.roots[index] = Entry::Free(index);
            }
        };

        self.free = Some(index);
    }
}

pub struct ProtSetIter<'a, T> {
    current: usize,
    protection_set: &'a ProtectionSet<T>,
}

impl<'a, T> Iterator for ProtSetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next valid entry, return it when found or None when end of roots is reached.
        while self.current < self.protection_set.roots.len() {
            if let Entry::Filled(object) = &self.protection_set.roots[self.current] {
                self.current += 1;
                return Some(object);
            } else {
                self.current += 1;
                return None
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_protection_set() {
        let mut protection_set = ProtectionSet::<usize>::new();

        // Protect a number of LDDs and record their indices.
        let mut indices: Vec<usize> = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..5000 {
            indices.push(protection_set.protect(rng.gen_range(0..1000)));
        }

        // Unprotect a number of LDDs.
        for index in 0..250 {
            protection_set.unprotect(indices[index]);
            indices.remove(index);
        }

        for index in indices {
            assert!(
                matches!(protection_set.roots[index], Entry::Filled(_)),
                "All indices that are not unprotected should occur in the protection set"
            );
        }

        for root in protection_set.iter() {
            assert!(*root <= 5000, "Root must be valid");
        }
    }
}
