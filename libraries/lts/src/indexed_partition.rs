use std::fmt;

use crate::Partition;

/// Defines a partition based on an explicit indexing of elements to their block
/// number.
#[derive(Debug)]
pub struct IndexedPartition {
    partition: Vec<usize>,

    num_of_blocks: usize,
}

impl IndexedPartition {

    /// Create a new partition where all elements are in a single block.
    pub fn new(num_of_elements: usize) -> IndexedPartition {
        IndexedPartition {
            partition: vec![0; num_of_elements],
            num_of_blocks: 1,
        }
    }

    /// Sets the block number of the given element
    pub fn set_block(&mut self, element_index: usize, block_number: usize) {
        // TODO: This assumes that the blocks are dense, otherwise it overestimates the number of blocks.
        self.num_of_blocks = self.num_of_blocks.max(block_number + 1);

        self.partition[element_index] = block_number;
    }
}

impl fmt::Display for IndexedPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;

        let mut first = true;

        for block_index in 0..self.partition.len() {

            // Print all elements with the same block number.
            let mut first_block = true;          
            for (element_index, _) in self.partition.iter().enumerate().filter(|(_, value)| {
                **value == block_index
            }) {
                if !first_block {
                    write!(f, ", ")?;
                } else {
                    if !first {
                        write!(f, ", ")?;
                    }

                    write!(f, "{{")?;
                }

                write!(f, "{}", element_index)?;
                first_block = false;
            }

            if !first_block {
                write!(f, "}}")?;
                first = false;
            }
        }

        write!(f, " }}")
    }
}

impl Partition for IndexedPartition {
    fn block_number(&self, state_index: usize) -> usize {
        self.partition[state_index]
    }

    fn num_of_blocks(&self) -> usize {
        self.num_of_blocks
    }
}
