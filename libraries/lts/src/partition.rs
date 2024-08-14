//!
//! Utility functions to define partitions of a set, and the corresponding
//! splitters.
//!

//! A partition keeps track of a number of blocks within a fixed set.
//!
//! The invariants are that the union of all blocks is the original set, and
//! that each block contains distinct elements.

use std::fmt;

use crate::IndexedPartition;

pub struct Partition {
    elements: Vec<usize>,
    blocks: Vec<Block>,
}

impl Partition {
    /// Create an initial partition where all the states are in a single block.
    pub fn new(num_of_entries: usize) -> Partition {
        debug_assert!(num_of_entries > 0, "Cannot partition the empty set");

        let blocks = vec![Block {
            begin: 0,
            end: num_of_entries,
        }];
        let mut elements = vec![0; num_of_entries];

        // Fill the elements with 1,2...
        let mut counter = 0;
        for element in &mut elements {
            *element = counter;
            counter += 1;
        }

        Partition { elements, blocks }
    }

    /// Split the given block into two separate block based on the splitter
    /// predicate.
    ///
    /// Note that this function can create empty blocks when splitter holds for
    /// all elements in the block. This does not break the partition, but is
    /// inefficient.
    pub fn split(&mut self, block_index: usize, mut splitter: impl FnMut(usize) -> bool) {
        let mut updated_block = self.blocks[block_index];
        let mut new_block: Option<usize> = None;

        // Loop over all elements, we use a while loop since the index stays the
        // same when a swap takes place.
        let mut element_index = updated_block.begin;
        while element_index < updated_block.end {
            if splitter(self.elements[element_index]) {
                match new_block {
                    None => {
                        // Introduce a new block for the split, containing only the new element.
                        self.blocks.push(Block {
                            begin: updated_block.end - 1,
                            end: updated_block.end,
                        });

                        // Swap the current element to the last place
                        self.elements.swap(element_index, updated_block.end - 1);

                        updated_block.end -= 1;
                        new_block = Some(self.blocks.len() - 1);
                    }
                    Some(new_block_index) => {
                        // Swap the current element to the beginning of the new block.
                        self.blocks[new_block_index].begin -= 1;
                        updated_block.end -= 1;

                        self.elements
                            .swap(element_index, self.blocks[new_block_index].begin);
                    }
                }
            } else {
                // If no swap takes place consider the next index.
                element_index += 1;
            }
        }

        // Update the original block.
        self.blocks[block_index] = updated_block;

        debug_assert!(
            self.is_consistent(),
            "After splitting the partition {:?} is inconsistent",
            self
        );
    }

    /// Return a reference to the given block.
    pub fn block(&self, block_index: usize) -> &Block {
        &self.blocks[block_index]
    }

    /// Returns the number of blocks in the partition.
    pub fn num_of_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Returns an iterator over the elements of a given block.
    pub fn iter_block(&self, block_index: usize) -> BlockIter<'_> {
        BlockIter {
            elements: &self.elements,
            index: self.blocks[block_index].begin,
            end: self.blocks[block_index].end,
        }
    }

    /// Returns true iff the invariants of a partition hold
    fn is_consistent(&self) -> bool {
        let mut marked = vec![false; self.elements.len()];

        for block in &self.blocks {
            for element in block.iter(self) {
                if marked[element] {
                    // This element belongs to another block
                    return false;
                }

                marked[element] = true;
            }
        }

        // Check that every element belongs to a block.
        !marked.contains(&false)
    }
}

impl IndexedPartition for Partition {
    fn block_number(&self, state_index: usize) -> usize {
        for block_index in 0..self.num_of_blocks() {
            for element in self.iter_block(block_index) {
                if element == state_index {
                    return block_index;
                }
            }
        }

        unreachable!("This state is not is any block");
    }

    fn num_of_blocks(&self) -> usize {
        self.blocks.len()
    }
}

impl fmt::Debug for Partition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        let mut first_block = true;
        for block in &self.blocks {
            if !first_block {
                write!(f, ", ")?;
            }
            write!(f, "{{")?;

            let mut first = true;

            for element in block.iter(self) {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", element)?;
                first = false;
            }

            write!(f, "}}")?;
            first_block = false;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    begin: usize,
    end: usize,
}

impl Block {
    /// Returns an iterator over the elements in this block.
    fn iter<'a>(&self, partition: &'a Partition) -> BlockIter<'a> {
        BlockIter {
            elements: &partition.elements,
            index: self.begin,
            end: self.end,
        }
    }

    /// Returns the number of elements in the block.
    pub fn len(&self) -> usize {
        debug_assert!(
            self.begin <= self.end,
            "The range of this block is incorrect"
        );

        self.end - self.begin
    }
}

pub struct BlockIter<'a> {
    elements: &'a Vec<usize>,
    index: usize,
    end: usize,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.end {
            let element = self.elements[self.index];
            self.index += 1;
            Some(element)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_partition() {
        let mut partition = Partition::new(10);

        partition.split(0, |element| element < 3);

        // The new block only has elements that satisfy the predicate.
        for element in partition.iter_block(1) {
            assert!(element < 3);
        }

        for element in partition.iter_block(0) {
            assert!(element >= 3);
        }

        partition.split(0, |element| element < 7);
        for element in partition.iter_block(2) {
            assert!(element >= 3 && element < 7);
        }

        for element in partition.iter_block(0) {
            assert!(element >= 7);
        }

        // Create an empty block, this should be fine but inefficient.
        partition.split(1, |element| element < 7);
    }
}
