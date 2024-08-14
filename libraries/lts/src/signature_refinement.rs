use std::fmt;
use std::mem::swap;

use ahash::AHashMap;
use log::debug;
use log::trace;

use crate::branching_bisim_signature;
use crate::quotient_lts;
use crate::strong_bisim_signature;
use crate::tau_star_partition;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Signature;
use crate::SignatureBuilder;

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> SigrefPartition {
    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::new();

    let partition = signature_refinement(lts, |state_index, partition| {
        strong_bisim_signature(state_index, lts, partition, &mut builder)
    });

    debug_assert!(
        is_valid_refinement(&lts, &partition, |state_index, partition| {
            strong_bisim_signature(state_index, lts, partition, &mut builder)
        }),
        "The resulting partition is not a strong bisimulation partition for LTS {:?}",
        lts
    );

    partition
}

/// Computes a branching bisimulation partitioning using signature refinement
pub fn branching_bisim_sigref(lts: &LabelledTransitionSystem) -> SigrefPartition {
    
    // Remove tau-loops since that is a prerequisite for the branching bisimulation signature.
    let simplified_lts = quotient_lts(lts, &tau_star_partition(lts));

    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::new();
    let mut stack: Vec<usize> = Vec::new();

    let partition = signature_refinement(&simplified_lts, |state_index, partition| {
        branching_bisim_signature(state_index, lts, partition, &mut builder, &mut stack)
    });

    debug_assert!(
        is_valid_refinement(&lts, &partition, |state_index, partition| {
            branching_bisim_signature(state_index, lts, partition, &mut builder, &mut stack)
        }),
        "The resulting partition is not a branching bisimulation partition for LTS {:?}",
        lts
    );

    partition
}

/// General signature refinement algorithm that accepts an arbitrary signature
fn signature_refinement<F>(lts: &LabelledTransitionSystem, mut signature: F) -> SigrefPartition
    where F: FnMut(usize, &SigrefPartition) -> Signature
{
    // Put all the states in the initial partition { S }.
    let mut id: AHashMap<Signature, usize> = AHashMap::new();

    // Assigns the signature to each state.
    let mut partition = SigrefPartition::new(lts.num_of_states());
    let mut next_partition = SigrefPartition::new(lts.num_of_states());

    // Refine partitions until stable.
    let mut old_count = 1;
    let mut iteration = 0;

    while old_count != id.len() {
        old_count = id.len();
        debug!("Iteration {iteration}, found {old_count} blocks");
        swap(&mut partition, &mut next_partition);

        // Clear the current partition to start the next blocks.
        id.clear();

        for (state_index, _) in lts.iter_states() {
            // Compute the signature of a single state
            let signature = signature(state_index, &partition);
            trace!("State {state_index} signature {:?}", signature);

            // Keep track of the index for every state.
            let mut new_id = id.len();
            id.entry(signature)
                .and_modify(|n| {
                    new_id = *n;
                })
                .or_insert_with(|| new_id);

            next_partition.partition[state_index] = new_id;
        }

        iteration += 1;

        debug_assert!(
            iteration <= lts.num_of_states(),
            "There can never be more splits than number of states"
        );
    }

    next_partition
}


/// Returns true iff the given partition is a strong bisimulation partition
pub fn is_valid_refinement<F, P>(lts: &LabelledTransitionSystem, partition: &P, mut compute_signature: F) -> bool
    where F: FnMut(usize, &P) -> Signature,
          P: IndexedPartition
{
    // Check that the partition is indeed stable and as such is a quotient of strong bisimulation
    let mut representative: Vec<usize> = Vec::new();
    for (state_index, _) in lts.iter_states() {
        let block = partition.block_number(state_index);

        if block + 1 > representative.len() {
            representative.resize(block + 1, 0);
            representative[block] = state_index;
        }

        // Check that this block only contains states that are strongly bisimilar to the representative state.
        let representative_index = representative[block];
        let signature = compute_signature(state_index, &partition);
        let representative_signature = compute_signature(representative_index, &partition);

        debug_assert_eq!(signature, representative_signature, "State {state_index} has a different signature then representative state {representative_index}, but are in the same block {block}");
        if signature != representative_signature {
            return false;
        }
    }

    true
}

/// Stores the partition for the signature refinement.
pub struct SigrefPartition {
    partition: Vec<usize>,
}

impl SigrefPartition {

    /// Create a new partition where all elements are in a single block.
    pub fn new(num_of_elements: usize) -> SigrefPartition {
        SigrefPartition {
            partition: vec![0; num_of_elements]
        }
    }

    /// Returns a mutable iterator over all elements in the partition.
    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut usize> + 'a {
        self.partition.iter_mut()
    }

    /// Sets the block number of the given element
    pub fn set_block(&mut self, element_index: usize, block_number: usize) {
        self.partition[element_index] = block_number;
    }
}

impl fmt::Debug for SigrefPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;

        let mut first = true;

        for block_index in 0..self.partition.len() {
            if !first {
                write!(f, ", ")?;
            }

            // Print all elements with the same block number.
            let mut first_block = true;          
            for (element_index, _) in self.partition.iter().enumerate().filter(|(_, value)| {
                **value == block_index
            }) {
                if !first_block {
                    write!(f, ", ")?;
                } else {
                    write!(f, "{{")?;
                }

                write!(f, "{}", element_index)?;
                first_block = false;
            }

            if !first_block {
                write!(f, "}}")?;
            }

            first = false;
        }

        write!(f, "}}")
    }
}

impl IndexedPartition for SigrefPartition {
    fn block_number(&self, state_index: usize) -> usize {
        self.partition[state_index]
    }

    fn num_of_blocks(&self) -> usize {
        // Figure out the highest block number for the number of states.
        // TODO: This assumes that the blocks are dense, otherwise it overestimates the number of blocks.
        match self.partition.iter().max() {
            None => {
                1
            },
            Some(max_block_number) => {
                max_block_number + 1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::random_lts;

    use super::*;

    #[test]
    fn test_random_bisim_sigref() {
        let lts = random_lts(10, 3, 3);
        strong_bisim_sigref(&lts);
    }

    #[test]
    fn test_random_branching_bisim_sigref() {
        let lts = random_lts(10, 3, 3);
        branching_bisim_sigref(&lts);
    }
}
