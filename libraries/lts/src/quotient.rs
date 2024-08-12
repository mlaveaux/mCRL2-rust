use crate::{compute_strong_bisim_signature, LabelledTransitionSystem, SignatureBuilder, State};

/// A trait for partition refinment algorithms that expose the block number for
/// every state. Can be used to compute the quotient labelled transition system.
pub trait IndexedPartition {

    /// Returns the block number for the given state.
    fn block_number(&self, state_index: usize) -> usize;
}


pub fn quotient_lts(lts: &LabelledTransitionSystem, partition: impl IndexedPartition) -> LabelledTransitionSystem {

    // Figure out the highest block number for the number of states.
    let mut max_block_number = 0;
    for (state_index, _state) in lts.states.iter().enumerate() {
        max_block_number = max_block_number.max(partition.block_number(state_index));
    }

    // Introduce the transitions based on the block numbers
    let num_of_transitions = 0;
    let mut states: Vec<State> = vec![State::default(); max_block_number + 1];
    for (state_index, state) in lts.states.iter().enumerate() {
        for (label, to) in &state.outgoing {
            states[partition.block_number(state_index)].outgoing.push((*label, partition.block_number(*to)));
        }
    }

    LabelledTransitionSystem {
        initial_state: 0,
        states,
        labels: lts.labels.clone(),
        num_of_transitions,
    }
}

/// Returns true iff the given partition is a strong bisimulation partition
pub fn is_strong_bisim(lts: &LabelledTransitionSystem, partition: &impl IndexedPartition) -> bool {
    // Avoids reallocations of the signature.
    let mut builder = SignatureBuilder::new();

    // Check that the partition is indeed stable and as such is a quotient of strong bisimulation
    let mut representative: Vec<usize> = Vec::new();
    for (state_index, state) in lts.states.iter().enumerate() {
        let block = partition.block_number(state_index);

        if block + 1 > representative.len() {
            representative.resize(block + 1, 0);
            representative[block] = state_index;
        }

        // Check that this block only contains states that are strongly bisimilar to the representative state.
        let representative_index = representative[block];
        let signature = compute_strong_bisim_signature(state, partition, &mut builder);
        let representative_signature = compute_strong_bisim_signature(&lts.states[representative_index], partition, &mut builder);

        debug_assert_eq!(signature, representative_signature, "State {state_index} has a different signature then representative state {representative_index}, but are in the same block {block}");
        if signature != representative_signature {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use log::trace;
    use test_log::test;

    use crate::{random_lts, strong_bisim_sigref};

    use super::*;

    #[test]
    fn test_random_quotient() {
        let lts = random_lts(10, 3);

        trace!("{lts:?}");
        quotient_lts(&lts, strong_bisim_sigref(&lts));
    }
}
