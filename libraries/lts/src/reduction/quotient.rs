use rustc_hash::FxHashSet;

use crate::LabelledTransitionSystem;
use crate::State;

/// A trait for partition refinement algorithms that expose the block number for
/// every state. Can be used to compute the quotient labelled transition system.
///
/// The invariants are that the union of all blocks is the original set, and
/// that each block contains distinct elements
pub trait Partition {
    /// Returns the block number for the given state.
    fn block_number(&self, state_index: usize) -> usize;

    /// Returns the number of blocks in the partition.
    fn num_of_blocks(&self) -> usize;

    /// Returns the number of elements in the partition.
    fn len(&self) -> usize;

    /// Returns whether the partition is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true iff the partitions are equal, runs in O(n^2)
    fn equal(&self, other: &impl Partition) -> bool {
        // Check that states in the same block, have a single (unique) number in
        // the other partition.
        for block_index in 0..self.num_of_blocks() {
            let mut other_block_index = None;

            for state_index in (0..self.len()).filter(|&state_index| self.block_number(state_index) == block_index) {
                match other_block_index {
                    None => other_block_index = Some(other.block_number(state_index)),
                    Some(other_block_index) => {
                        if other.block_number(state_index) != other_block_index {
                            return false;
                        }
                    }
                }
            }
        }

        for block_index in 0..other.num_of_blocks() {
            let mut other_block_index = None;

            for state_index in (0..self.len()).filter(|&state_index| other.block_number(state_index) == block_index) {
                match other_block_index {
                    None => other_block_index = Some(self.block_number(state_index)),
                    Some(other_block_index) => {
                        if self.block_number(state_index) != other_block_index {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}

/// Returns a new LTS based on the given partition.
///
/// All states in a single block are replaced by a single representative state.
pub fn quotient_lts(
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    eliminate_tau_loops: bool,
) -> LabelledTransitionSystem {
    // Introduce the transitions based on the block numbers
    let mut states: Vec<State> = vec![State::default(); partition.num_of_blocks()];
    let mut transitions: FxHashSet<(usize, usize, usize)> = FxHashSet::default();

    for (state_index, state) in lts.iter_states() {
        for &(label, to) in &state.outgoing {
            let block = partition.block_number(state_index);
            let to_block = partition.block_number(to);

            // If we eliminate tau loops then check if the to and from end up in the same block
            if !eliminate_tau_loops
                || !(lts.is_hidden_label(label) && block == to_block)
            {
                debug_assert!(
                    partition.block_number(state_index) < partition.num_of_blocks(),
                    "Quotienting assumes that the block numbers do not exceed the number of blocks"
                );

                // Make sure to keep the outgoing transitions sorted.
                transitions.insert((block, label, to_block));
            }
        }
    }

    for &(from, label, to) in &transitions {
        states[from].outgoing.push((label, to));
    }

    LabelledTransitionSystem::new(
        partition.block_number(lts.initial_state_index()),
        states,
        lts.labels().into(),
        lts.hidden_labels().into(),
        transitions.len(),
    )
}
