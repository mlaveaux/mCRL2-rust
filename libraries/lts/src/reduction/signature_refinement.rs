use std::mem::swap;

use bumpalo::Bump;
use fxhash::FxHashMap;
use fxhash::FxHashSet;
use log::debug;
use log::trace;

use crate::branching_bisim_signature;
use crate::branching_bisim_signature_sorted;
use crate::quotient_lts;
use crate::reorder_states;
use crate::sort_topological;
use crate::strong_bisim_signature;
use crate::tau_scc_decomposition;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;
use crate::Signature;
use crate::SignatureBuilder;

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let partition = signature_refinement(lts, |state_index, partition, _, _, builder| {
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
    let start = std::time::Instant::now();
    let scc_partition = tau_scc_decomposition(lts);

    let tau_loop_free_lts = quotient_lts(lts, &scc_partition, true);

    // Sort the states according to the topological order of the tau transitions.
    let topological_permutation = sort_topological(
        &tau_loop_free_lts,
        |label_index, _| tau_loop_free_lts.is_hidden_label(label_index),
        true,
    )
    .expect("After quotienting, the LTS should not contain cycles");

    let permuted_lts = reorder_states(&tau_loop_free_lts, |i| topological_permutation[i]);
    let mut expected_builder = SignatureBuilder::default();
    let mut visited = FxHashSet::default();
    let mut stack = Vec::new();

    let partition = signature_refinement(
        &permuted_lts,
        |state_index, partition, next_partition, block_to_signature, builder| {
            branching_bisim_signature_sorted(
                state_index,
                &permuted_lts,
                partition,
                next_partition,
                block_to_signature,
                builder,
            );            

            // Compute the expected signature, only used in debugging.
            if cfg!(debug_assertions) {
                branching_bisim_signature(
                    state_index,
                    &permuted_lts,
                    partition,
                    &mut expected_builder,
                    &mut visited,
                    &mut stack,
                );
                let expected_result = builder.clone();

                let signature = Signature::new(&builder);
                debug_assert_eq!(
                    signature.as_slice(),
                    expected_result,
                    "The sorted and expected signature should be the same"
                );
            }
        },
    );

    // Combine the SCC partition with the branching bisimulation partition.
    let mut combined_partition = IndexedPartition::new(lts.num_of_states());

    for (state_index, _) in lts.iter_states() {
        let scc_block = scc_partition.block_number(state_index);
        let reorder = topological_permutation[scc_block];
        let branching_block = partition.block_number(reorder);

        combined_partition.set_block(state_index, branching_block);
    }

    trace!("Final partition {combined_partition}");

    let mut stack: Vec<usize> = Vec::new();
    let mut visited = FxHashSet::default();
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
        "The resulting partition is not a branching bisimulation partition."
    );

    debug!(
        "Time branching_bisim_sigref: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    combined_partition
}

/// General signature refinement algorithm that accepts an arbitrary signature
///
/// The signature function is called for each state and should fill the
/// signature builder with the signature of the state. It consists of the
/// current partition, the signatures per state for the next partition.
fn signature_refinement<F>(lts: &LabelledTransitionSystem, mut signature: F) -> IndexedPartition
where
    F: FnMut(usize, &IndexedPartition, &IndexedPartition, &Vec<Signature>, &mut SignatureBuilder),
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
    let mut block_to_signature: Vec<Signature> = Vec::new();
    block_to_signature.resize_with(lts.num_of_states(), Signature::default);

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
            signature(
                state_index,
                &partition,
                &next_partition,
                &block_to_signature,
                &mut builder,
            );

            trace!("State {state_index} signature {:?}", builder);

            // Keep track of the index for every state, either use the arena to allocate space or simply borrow the value.
            let mut new_id = id.len();
            if let Some(index) = id.get(&Signature::new(&builder)) {
                new_id = *index;
            } else {
                let slice = arena.alloc_slice_copy(&builder);
                id.insert(Signature::new(slice), new_id);

                // Keep track of the signature for every block in the next partition.
                block_to_signature[new_id] = Signature::new(slice);
            }

            next_partition.set_block(state_index, new_id);
        }

        iteration += 1;

        debug_assert!(
            iteration <= lts.num_of_states().max(2),
            "There can never be more splits than number of states, but at least two iterations for stability"
        );
    }

    trace!("Refinement partition {partition}");
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
    let mut block_to_signature: Vec<Option<SignatureBuilder>> = vec![None; partition.num_of_blocks()];

    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::default();

    for (state_index, _) in lts.iter_states() {
        let block = partition.block_number(state_index);

        // Compute the flat signature, which has Hash and is more compact.
        compute_signature(state_index, &partition, &mut builder);
        let signature: Vec<(usize, usize)> = builder.clone();    

        if let Some(block_signature) = &block_to_signature[block] { 
            if signature != *block_signature {
                trace!("State {state_index} has a different signature {signature:?} then the block {block} which has signature {block_signature:?}");
                return false;
            }
        } else {
            block_to_signature[block] = Some(signature);
        };
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
