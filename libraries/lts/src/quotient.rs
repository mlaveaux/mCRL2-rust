use crate::LabelledTransitionSystem;
use crate::State;

/// A trait for partition refinment algorithms that expose the block number for
/// every state. Can be used to compute the quotient labelled transition system.
pub trait IndexedPartition {

    /// Returns the block number for the given state.
    fn block_number(&self, state_index: usize) -> usize;

    /// Returns the number of blocks in the partition.
    fn num_of_blocks(&self) -> usize;
}

/// Returns a new LTS based on the given partition.
/// 
/// All states in a single block are replaced by a single representative state.
pub fn quotient_lts(lts: &LabelledTransitionSystem, partition: &impl IndexedPartition) -> LabelledTransitionSystem {

    // Introduce the transitions based on the block numbers
    let mut num_of_transitions = 0;
    let mut states: Vec<State> = vec![State::default(); partition.num_of_blocks()];
    for (state_index, state) in lts.iter_states() {
        for (label, to) in &state.outgoing {
            debug_assert!(partition.block_number(state_index) < partition.num_of_blocks(), "Quotienting assumes that the block numbers do not exceed the number of blocks");
            states[partition.block_number(state_index)].outgoing.push((*label, partition.block_number(*to)));
            num_of_transitions += 1;
        }
    }

    LabelledTransitionSystem::new(partition.block_number(0),
        states,
        lts.labels().into(),
        num_of_transitions)
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::random_lts;
    use crate::strong_bisim_sigref;

    use super::*;

    #[test]
    fn test_random_quotient() {
        let lts = random_lts(10, 3, 3);
        quotient_lts(&lts, &strong_bisim_sigref(&lts));
    }
}
