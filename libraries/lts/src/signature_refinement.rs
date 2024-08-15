use std::mem::swap;

use ahash::AHashMap;
use ahash::AHashSet;
use log::debug;
use log::trace;

use crate::branching_bisim_signature;
use crate::quotient_lts;
use crate::strong_bisim_signature;
use crate::tau_scc_decomposition;
use crate::IncomingTransitions;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;
use crate::Signature;
use crate::SignatureBuilder;

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
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
pub fn branching_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    // Remove tau-loops since that is a prerequisite for the branching bisimulation signature.
    let simplified_lts = quotient_lts(lts, &tau_scc_decomposition(lts));

    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut visited = AHashSet::new();

    let partition = signature_refinement(&simplified_lts, |state_index, partition| {
        branching_bisim_signature(
            state_index,
            lts,
            partition,
            &mut builder,
            &mut visited,
            &mut stack,
        )
    });

    debug_assert!(
        is_valid_refinement(&lts, &partition, |state_index, partition| {
            branching_bisim_signature(
                state_index,
                lts,
                partition,
                &mut builder,
                &mut visited,
                &mut stack,
            )
        }),
        "The resulting partition is not a branching bisimulation partition for LTS {:?}",
        lts
    );

    partition
}

/// General signature refinement algorithm that accepts an arbitrary signature
fn signature_refinement<F>(lts: &LabelledTransitionSystem, mut signature: F) -> IndexedPartition
where
    F: FnMut(usize, &IndexedPartition) -> Signature,
{
    trace!("{:?}", lts);
    
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

    trace!("Final partition {partition}");
    partition
}


/// Returns true iff the given partition is a strong bisimulation partition
pub fn is_valid_refinement<F, P>(
    lts: &LabelledTransitionSystem,
    partition: &P,
    mut compute_signature: F,
) -> bool
where
    F: FnMut(usize, &P) -> Signature,
    P: Partition,
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

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::random_lts;

    use super::*;

    #[test]
    fn test_random_strong_bisim_sigref() {
        let lts = random_lts(10, 3, 3);
        strong_bisim_sigref(&lts);
    }

    #[test]
    fn test_random_branching_bisim_sigref() {
        let lts = random_lts(10, 3, 3);

        let strong_partition = strong_bisim_sigref(&lts);
        let branching_partition = branching_bisim_sigref(&lts);

        assert!(
            branching_partition.num_of_blocks() <= strong_partition.num_of_blocks(),
            "The branching partition should be a refinement of the strong partition"
        );
    }
}
