use std::mem::swap;

use ahash::AHashMap;
use ahash::AHashSet;
use log::debug;
use log::trace;

use crate::LabelledTransitionSystem;
use crate::State;
use crate::IndexedPartition;

/// Computes a strong bisimulation quotient using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> SigrefPartition {

    // Put all the states in the initial partition { S }.
    let mut id: AHashMap<Signature, usize> = AHashMap::new();

    // Avoids reallocations of the signature.
    let mut builder = SignatureBuilder::new();

    // Assigns the signature to each state.
    let mut partition = SigrefPartition {
        partition: vec![0; lts.num_of_states()]
    };
    let mut next_partition = SigrefPartition {
        partition: vec![0; lts.num_of_states()]
    };

    // Refine partitions until stable.
    let mut old_count = 1;
    let mut iteration = 0;

    while old_count != id.len() {
        old_count = id.len();
        debug!("Iteration {iteration}, found {old_count} blocks");
        swap(&mut partition, &mut next_partition);

        // Clear the current partition to start the next blocks.
        id.clear();

        for (state_index, state) in lts.iter_states() {

            // Compute the signature of a single state
            let signature = compute_strong_bisim_signature(state, &partition, &mut builder);
            trace!("State {state_index} signature {:?}", signature);

            // Keep track of the index for every state.
            let mut new_id = id.len();
            id.entry(signature)
                .and_modify(|n| {
                    new_id = *n;
                })
                .or_insert_with(|| {
                    new_id
                });

            next_partition.partition[state_index] = new_id;
        }

        iteration += 1;

        debug_assert!(iteration <= lts.num_of_states(), "There can never be more splits than number of states");
    }

    next_partition
}

/// Stores the partition for the signature refinement.
pub struct SigrefPartition {
    partition: Vec<usize>
}

impl IndexedPartition for SigrefPartition {
    fn block_number(&self, state_index: usize) -> usize {
        self.partition[state_index]
    }
}

/// The type of a signature for strong bisimulation. We use sorted vectors to
/// avoid the overhead of hash sets that might have unused values.
type Signature = Vec<(usize, usize)>;

/// The builder used to construct the signature.
pub type SignatureBuilder = AHashSet::<(usize, usize)>;

/// Returns the signature for strong bisimulation sig(s) = { (a, pi(t)) | s -a-> t in T }
pub fn compute_strong_bisim_signature(state: &State, partition: &impl IndexedPartition, builder: &mut SignatureBuilder) -> Signature { 

    for (label, to) in &state.outgoing {
        builder.insert((*label, partition.block_number(*to)));
    }

    // Compute the flat signature, which has Hash and is more compact.
    let mut signature_flat: Signature = builder.drain().collect();
    signature_flat.sort_unstable();

    signature_flat
}

#[cfg(test)]
mod tests {
    use log::trace;
    use test_log::test;
    
    use crate::{is_strong_bisim, random_lts};

    use super::*;

    #[test]
    fn test_random_bisim_sigref() {
        let lts = random_lts(10, 3);

        trace!("{lts:?}");
        assert!(is_strong_bisim(&lts, &strong_bisim_sigref(&lts)), "The resulting partition is not a strong bisimulation partition");
    }
}