use std::mem::swap;

use bumpalo::Bump;
use fxhash::FxHashMap;
use fxhash::FxHashSet;
use log::debug;
use log::trace;

use crate::branching_bisim_signature;
use crate::quotient_lts;
use crate::strong_bisim_signature;
use crate::tau_scc_decomposition;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;
use crate::Signature;
use crate::SignatureBuilder;

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let partition = signature_refinement(lts, |state_index, partition, builder| {
        strong_bisim_signature(state_index, lts, partition, builder);
    });

    debug_assert!(
        is_valid_refinement(&lts, &partition, |state_index, partition, builder| {
            strong_bisim_signature(state_index, lts, partition, builder);
        }),
        "The resulting partition is not a strong bisimulation partition for LTS {:?}",
        lts
    );

    partition
}

/// Computes a branching bisimulation partitioning using signature refinement
pub fn branching_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    // Remove tau-loops since that is a prerequisite for the branching bisimulation signature.
    let scc_partition = tau_scc_decomposition(lts);
    let tau_loop_free_lts = quotient_lts(lts, &scc_partition, true);

    let mut stack: Vec<usize> = Vec::new();
    let mut visited = FxHashSet::default();

    let partition = signature_refinement(&tau_loop_free_lts, |state_index, partition, builder| {
        branching_bisim_signature(
            state_index,
            &tau_loop_free_lts,
            partition,
            builder,
            &mut visited,
            &mut stack,
        );
    });

    // Combine the SCC partition with the branching bisimulation partition.
    let mut combined_partition = IndexedPartition::new(lts.num_of_states());

    for (state_index, _) in lts.iter_states() {
        let scc_block = scc_partition.block_number(state_index);
        let branching_block = partition.block_number(scc_block);

        combined_partition.set_block(state_index, branching_block);
    }

    debug_assert!(
        is_valid_refinement(
            &lts,
            &combined_partition,
            |state_index, partition, builder| {
                branching_bisim_signature(
                    state_index,
                    &lts,
                    partition,
                    builder,
                    &mut visited,
                    &mut stack,
                );
            }
        ),
        "The resulting partition is not a branching bisimulation partition for LTS: \n {:?}",
        lts
    );

    combined_partition
}

/// General signature refinement algorithm that accepts an arbitrary signature
fn signature_refinement<F>(lts: &LabelledTransitionSystem, mut signature: F) -> IndexedPartition
where
    F: FnMut(usize, &IndexedPartition, &mut SignatureBuilder),
{
    trace!("{:?}", lts);

    // Avoids reallocations when computing the signature.
    let mut arena = Bump::new();
    let mut builder = SignatureBuilder::default();

    // Put all the states in the initial partition { S }.
    let mut id: FxHashMap<Signature, usize> = FxHashMap::default();

    // Assigns the signature to each state.
    let mut partition = IndexedPartition::new(lts.num_of_states());
    let mut next_partition = IndexedPartition::new(lts.num_of_states());

    // Refine partitions until stable.
    let mut old_count = 1;
    let mut iteration = 0;

    while old_count != id.len() {
        old_count = id.len();
        debug!("Iteration {iteration}, found {old_count} blocks");
        swap(&mut partition, &mut next_partition);

        // Clear the current partition to start the next blocks.
        id.clear();

        // Remove the current signatures.
        arena.reset();

        for (state_index, _) in lts.iter_states() {
            // Compute the signature of a single state
            signature(state_index, &partition, &mut builder);

            // Compute the flat signature, which has Hash and is more compact.
            builder.sort_unstable();
            builder.dedup();

            trace!("State {state_index} signature {:?}", builder);

            // Keep track of the index for every state, either use the arena to allocate space or simply borrow the value.
            let mut new_id = id.len();
            if let Some(index) = id.get(&Signature::new(&builder)) {
                new_id = *index;
            } else {
                let new_signature = Signature::new(arena.alloc_slice_copy(&builder));
                id.insert(new_signature, new_id);
            }

            next_partition.set_block(state_index, new_id);
        }

        iteration += 1;

        debug_assert!(
            iteration <= lts.num_of_states().max(2),
            "There can never be more splits than number of states, but at least two iterations for stability"
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
    F: FnMut(usize, &P, &mut SignatureBuilder),
    P: Partition,
{
    // Check that the partition is indeed stable and as such is a quotient of strong bisimulation
    let mut representative: Vec<usize> = Vec::new();

    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::default();

    for (state_index, _) in lts.iter_states() {
        let block = partition.block_number(state_index);

        if block + 1 > representative.len() {
            representative.resize(block + 1, 0);
            representative[block] = state_index;
        }

        // Check that this block only contains states that are strongly bisimilar to the representative state.
        let representative_index = representative[block];
        compute_signature(state_index, &partition, &mut builder);

        // Compute the flat signature, which has Hash and is more compact.
        let mut signature: Vec<(usize, usize)> = builder.clone();
        signature.sort_unstable();
        signature.dedup();

        compute_signature(representative_index, &partition, &mut builder);

        // Compute the flat signature, which has Hash and is more compact.
        let mut representative_signature: Vec<(usize, usize)> = builder.clone();
        representative_signature.sort_unstable();
        representative_signature.dedup();

        if signature != representative_signature {
            trace!("State {state_index} has a different signature then representative state {representative_index}, but are in the same block {block}");
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

        for (state_index, _) in lts.iter_states() {
            for (other_state_index, _) in lts.iter_states() {
                if strong_partition.block_number(state_index)
                    == strong_partition.block_number(other_state_index)
                {
                    // If the states are together according to branching bisimilarity, then they should also be together according to strong bisimilarity.
                    assert_eq!(
                        branching_partition.block_number(state_index),
                        branching_partition.block_number(other_state_index),
                        "The branching partition should be a refinement of the strong partition, 
                        but states {state_index} and {other_state_index} are in different blocks"
                    );
                }
            }
        }
    }
}
