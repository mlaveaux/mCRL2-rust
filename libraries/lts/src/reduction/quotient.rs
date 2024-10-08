use log::debug;

use crate::LabelledTransitionSystem;
use crate::State;

/// A trait for partition refinment algorithms that expose the block number for
/// every state. Can be used to compute the quotient labelled transition system.
///
/// The invariants are that the union of all blocks is the original set, and
/// that each block contains distinct elements
pub trait Partition {
    /// Returns the block number for the given state.
    fn block_number(&self, state_index: usize) -> usize;

    /// Returns the number of blocks in the partition.
    fn num_of_blocks(&self) -> usize;
}

/// Returns a new LTS based on the given partition.
///
/// All states in a single block are replaced by a single representative state.
pub fn quotient_lts(
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    eliminate_tau_loops: bool,
) -> LabelledTransitionSystem {
    let start = std::time::Instant::now();

    // Introduce the transitions based on the block numbers
    let mut num_of_transitions = 0;
    let mut states: Vec<State> = vec![State::default(); partition.num_of_blocks()];
    for (state_index, state) in lts.iter_states() {
        for (label, to) in &state.outgoing {
            // If we eliminate tau loops then check if the to and from end up in the same block
            if !eliminate_tau_loops
                || !(lts.is_hidden_label(*label)
                    && partition.block_number(state_index) == partition.block_number(*to))
            {
                debug_assert!(
                    partition.block_number(state_index) < partition.num_of_blocks(),
                    "Quotienting assumes that the block numbers do not exceed the number of blocks"
                );

                let outgoing = &mut states[partition.block_number(state_index)].outgoing;
                let to_block = partition.block_number(*to);

                // Make sure to keep the outgoing transitions sorted.
                match outgoing.binary_search(&(*label, to_block)) {
                    Ok(_) => {} // element already in vector
                    Err(pos) => {
                        outgoing.insert(pos, (*label, to_block));
                        num_of_transitions += 1;
                    }
                }
            }
        }
    }

    debug!("Time quotient_lts: {:.3}s", start.elapsed().as_secs_f64());
    LabelledTransitionSystem::new(
        partition.block_number(lts.initial_state_index()),
        states,
        lts.labels().into(),
        lts.hidden_labels().into(),
        num_of_transitions,
    )
}
