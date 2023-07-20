/// The protection set keeps track of nodes that should not be garbage
/// collected since they are being referenced by instances.
#[derive(Default)]
pub struct ProtectionSet
{    
    roots: Vec<(usize, bool)>, // The set of root active nodes.
    free: Option<usize>,
    number_of_insertions: u64,
}

impl ProtectionSet
{
    pub fn new() -> Self
    {
        ProtectionSet::default()
    }

    /// Returns the number of insertions into the protection set.
    pub fn number_of_insertions(&self) -> u64 
    {
        self.number_of_insertions 
    }

    /// Returns maximum number of active [Ldd] instances.
    pub fn maximum_size(&self) -> usize 
    {
        self.roots.capacity() 
    }

    /// Returns an iterator over all root indices in the protection set.
    pub fn iter(&self) -> ProtSetIter
    {
        ProtSetIter {
            current: 0,
            protection_set: self,
        }
    }

    /// Protect the given root node to prevent garbage collection.
    pub fn protect(&mut self, root: usize) -> usize
    {
        self.number_of_insertions += 1;

        match self.free {
            Some(first) => {
                let next = self.roots[first];
                if first == next.0 {
                    // The list is empty as its first element points to itself.
                    self.free = None;
                } else {
                    // Update free to be the next element in the list.
                    self.free = Some(next.0);
                }

                self.roots[first] = (root, true);
                first
            }
            None => {
                // If free list is empty insert new entry into roots.
                self.roots.push((root, true));
                self.roots.len() - 1
            }
        }
    }
    
    /// Remove protection from the given LDD node. Note that index must be the
    /// index returned by the [protect] call.
    pub fn unprotect(&mut self, index: usize)
    {
        match self.free {
            Some(next) => {
                self.roots[index] = (next, false);
            }
            None => {
                self.roots[index] = (index, false);
            }
        };
        
        self.free = Some(index);
    }
}

pub struct ProtSetIter<'a>
{
    current: usize,
    protection_set: &'a ProtectionSet,
}

impl Iterator for ProtSetIter<'_>
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item>
    {
        // Find the next valid entry, return it when found or None when end of roots is reached.
        while self.current < self.protection_set.roots.len()
        {
            let (root, valid) = self.protection_set.roots[self.current];
            self.current += 1;
            if valid {
                return Some(root);
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{test_utility::*};

    #[test]
    fn test_protection_set()
    {
        let mut protection_set = ProtectionSet::new();

        // Protect a number of LDDs and record their indices.
        let root_variables = random_vector(1000, 5000);
        let mut indices: Vec<usize> = Vec::new();

        for variable in root_variables
        {
            indices.push(protection_set.protect(variable as usize));
        }

        // Unprotect a number of LDDs.
        for index in 0..250
        {
            protection_set.unprotect(indices[index]);
            indices.remove(index);
        }
        
        for index in indices
        {
            let (_, valid) = protection_set.roots[index];
            assert!(valid, "All indices that are not unprotected should occur in the protection set");
        }

        for root in protection_set.iter()
        {
            assert!(root <= 5000, "Root must be valid");
        }
    }
}