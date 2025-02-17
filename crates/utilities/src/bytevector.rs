use std::fmt;
use std::marker::PhantomData;

/// A vector data structure that stores objects in a byte compressed format
#[derive(Debug, Default)]
pub struct ByteCompressedVec<T> {
    data: Vec<u8>,
    bytes_per_entry: usize,
    _marker: PhantomData<T>,
}

impl<T: Entry + fmt::Debug> ByteCompressedVec<T> {
    pub fn new() -> ByteCompressedVec<T> {
        ByteCompressedVec {
            data: Vec::new(),
            bytes_per_entry: 0,
            _marker: PhantomData,
        }
    }

    /// Adds a new entry to the vector.
    pub fn push(&mut self, entry: T) {
        self.resize_entries(entry.bytes_required());

        // Add the new entry to the end of the vector.
        let old_len = self.data.len();
        self.data.resize(old_len + self.bytes_per_entry, 0);
        entry.to_bytes(&mut self.data[old_len..]);
    }

    /// Returns the entry at the given index.
    pub fn get(&self, index: usize) -> T {
        let start = index * self.bytes_per_entry;
        let end = start + self.bytes_per_entry;
        T::from_bytes(&self.data[start..end])
    }

    /// Sets the entry at the given index.
    pub fn set(&mut self, index: usize, entry: T) {
        self.resize_entries(entry.bytes_required());

        let start = index * self.bytes_per_entry;
        let end = start + self.bytes_per_entry;
        entry.to_bytes(&mut self.data[start..end]);
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        if self.bytes_per_entry == 0 {
            0
        } else {
            debug_assert!(self.data.len() % self.bytes_per_entry == 0);
            self.data.len() / self.bytes_per_entry
        }
    }

    /// Returns true if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the elements in the vector.
    pub fn iter(&self) -> ByteCompressedVecIterator<T> {
        ByteCompressedVecIterator {
            vector: self,
            current: 0,
        }
    }

    /// Resizes all entries in the vector to the given length.
    fn resize_entries(&mut self, new_bytes_required: usize) {
        if new_bytes_required > self.bytes_per_entry {
            let mut new_data: Vec<u8> = vec![0; self.len() * new_bytes_required];

            if self.bytes_per_entry > 0 {
                // Resize all the existing elements because the new entry requires more bytes.
                for (index, entry) in self.iter().enumerate() {
                    let start = index * new_bytes_required;
                    let end = start + new_bytes_required;
                    entry.to_bytes(&mut new_data[start..end]);
                }
            }

            self.bytes_per_entry = new_bytes_required;
            self.data = new_data;
        }
    }
}

impl<T: Entry + Clone + fmt::Debug> ByteCompressedVec<T> {
    pub fn from_elem(entry: T, n: usize) -> ByteCompressedVec<T> {
        let mut vec = ByteCompressedVec::new();
        for _ in 0..n {
            vec.push(entry.clone());
        }
        vec
    }
}

pub struct ByteCompressedVecIterator<'a, T> {
    vector: &'a ByteCompressedVec<T>,
    current: usize,
}

impl<T: Entry + fmt::Debug> Iterator for ByteCompressedVecIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.vector.len() {
            let result = self.vector.get(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub trait Entry {
    // Returns the entry as a byte vector
    fn to_bytes(&self, bytes: &mut [u8]);

    // Creates an entry from a byte vector
    fn from_bytes(bytes: &[u8]) -> Self;

    // Returns the number of bytes required to store the current entry
    fn bytes_required(&self) -> usize;
}

impl Entry for usize {
    fn to_bytes(&self, bytes: &mut [u8]) {
        let array = &self.to_le_bytes();
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = array[i];
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut array = [0; 8];
        for (i, byte) in bytes.iter().enumerate() {
            array[i] = *byte;
        }
        usize::from_le_bytes(array)
    }

    fn bytes_required(&self) -> usize {
        ((self + 1).ilog2() / u8::BITS) as usize + 1
    }
}

#[cfg(test)]
mod tests {
    use crate::bytevec;
    use rand::distr::Uniform;
    use rand::Rng;
    use test_log::test;

    use super::*;

    #[test]
    fn test_index_bytevector() {
        let mut vec = ByteCompressedVec::new();
        vec.push(1);
        assert_eq!(vec.len(), 1);

        vec.push(1024);
        assert_eq!(vec.len(), 2);

        assert_eq!(vec.get(0), 1);
        assert_eq!(vec.get(1), 1024);
    }

    #[test]
    fn test_random_bytevector() {
        let rng = rand::rng();

        let range = Uniform::new(0, usize::MAX).unwrap();
        let expected_vector: Vec<usize> = rng.sample_iter(range).take(100).collect();
        let mut vector = ByteCompressedVec::new();

        for element in &expected_vector {
            vector.push(*element);

            for (expected, element) in expected_vector.iter().zip(vector.iter()) {
                assert_eq!(*expected, element);
            }
        }
    }

    #[test]
    fn test_random_setting_bytevector() {
        let rng = rand::rng();

        let range = Uniform::new(0, usize::MAX).unwrap();
        let expected_vector: Vec<usize> = rng.sample_iter(range).take(100).collect();
        let mut vector = bytevec![0; 100];

        for (index, element) in expected_vector.iter().enumerate() {
            vector.set(index, *element);
        }

        for (expected, element) in expected_vector.iter().zip(vector.iter()) {
            assert_eq!(*expected, element);
        }
    }

    #[test]
    fn test_usize_entry() {
        let mut rng = rand::rng();

        for _ in 0..100 {
            let value = rng.random_range(0..1024);
            assert!(value.bytes_required() <= 2);

            let mut bytes = [0; 2];
            value.to_bytes(&mut bytes);
            assert_eq!(usize::from_bytes(&bytes), value);
        }
    }
}
