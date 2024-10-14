use std::mem::swap;

use bumpalo::Bump;
use log::debug;
use log::trace;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use crate::branching_bisim_signature;
use crate::branching_bisim_signature_sorted;
use crate::combine_partition;
use crate::preprocess_branching;
use crate::strong_bisim_signature;
use crate::BlockPartition;
use crate::BlockPartitionBuilder;
use crate::IncomingTransitions;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;
use crate::Signature;
use crate::SignatureBuilder;

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let start = std::time::Instant::now();
    let partition = signature_refinement::<_, false>(lts, |state_index, partition, _, builder| {
        strong_bisim_signature(state_index, lts, partition, builder);
    });

    debug_assert_eq!(
        partition,
        strong_bisim_sigref_naive(lts),
        "The resulting partition is not a valid strong bisimulation partition."
    );

    debug!("Time strong_bisim_sigref: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    partition.into()
}

/// Computes a strong bisimulation partitioning using signature refinement
pub fn strong_bisim_sigref_naive(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let start = std::time::Instant::now();
    let partition = signature_refinement_naive(lts, |state_index, partition, _, builder| {
        strong_bisim_signature(state_index, lts, partition, builder);
    });

    debug!("Time strong_bisim_sigref_naive: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    partition
}

/// Computes a branching bisimulation partitioning using signature refinement
pub fn branching_bisim_sigref(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let (preprocessed_lts, preprocess_partition) = preprocess_branching(lts);

    let start = std::time::Instant::now();
    let mut expected_builder = SignatureBuilder::default();
    let mut visited = FxHashSet::default();
    let mut stack = Vec::new();

    let partition = signature_refinement::<_, true>(
        &preprocessed_lts,
        |state_index, partition, state_to_signature, builder| {
            branching_bisim_signature_sorted(
                state_index,
                &preprocessed_lts,
                partition,
                state_to_signature,
                builder,
            );

            // Compute the expected signature, only used in debugging.
            if cfg!(debug_assertions) {
                branching_bisim_signature(
                    state_index,
                    &preprocessed_lts,
                    partition,
                    &mut expected_builder,
                    &mut visited,
                    &mut stack,
                );
                let expected_result = builder.clone();

                let signature = Signature::new(builder);
                debug_assert_eq!(
                    signature.as_slice(),
                    expected_result,
                    "The sorted and expected signature should be the same"
                );
            }
        },
    );

    debug_assert_eq!(
        partition,
        signature_refinement_naive(&preprocessed_lts, |state_index, partition, _, builder| {
            branching_bisim_signature(
                state_index,
                &preprocessed_lts,
                partition,
                builder,
                &mut visited,
                &mut stack,
            );
        }),
        "The resulting partition is not a branching bisimulation partition."
    );

    // Combine the SCC partition with the branching bisimulation partition.
    let combined_partition = combine_partition(preprocess_partition, &partition);
    trace!("Final partition {combined_partition}");

    debug!("Time branching_bisim_sigref: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    combined_partition
}

/// Computes a branching bisimulation partitioning using signature refinement without dirty blocks.
pub fn branching_bisim_sigref_naive(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let (preprocessed_lts, preprocess_partition) = preprocess_branching(lts);

    let start = std::time::Instant::now();
    let mut expected_builder = SignatureBuilder::default();
    let mut visited = FxHashSet::default();
    let mut stack = Vec::new();

    let partition = signature_refinement_naive(
        &preprocessed_lts,
        |state_index, partition, state_to_signature, builder| {
            branching_bisim_signature_sorted(
                state_index,
                &preprocessed_lts,
                partition,
                state_to_signature,
                builder,
            );

            // Compute the expected signature, only used in debugging.
            if cfg!(debug_assertions) {
                branching_bisim_signature(
                    state_index,
                    &preprocessed_lts,
                    partition,
                    &mut expected_builder,
                    &mut visited,
                    &mut stack,
                );
                let expected_result = builder.clone();

                let signature = Signature::new(builder);
                debug_assert_eq!(
                    signature.as_slice(),
                    expected_result,
                    "The sorted and expected signature should be the same"
                );
            }
        },
    );

    // Combine the SCC partition with the branching bisimulation partition.
    let combined_partition = combine_partition(preprocess_partition, &partition);
    trace!("Final partition {combined_partition}");

    debug!("number_of_states" = combined_partition.num_of_blocks();
        "Time branching_bisim_sigref_naive: {:.3}s",
        start.elapsed().as_secs_f64()
    );
    combined_partition
}

/// General signature refinement algorithm that accepts an arbitrary signature
///
/// The signature function is called for each state and should fill the
/// signature builder with the signature of the state. It consists of the
/// current partition, the signatures per state for the next partition.
fn signature_refinement<F, const BRANCHING: bool>(
    lts: &LabelledTransitionSystem,
    mut signature: F,
) -> BlockPartition
where
    F: FnMut(usize, &BlockPartition, &Vec<Signature>, &mut SignatureBuilder),
{
    trace!("{:?}", lts);

    // Avoids reallocations when computing the signature.
    let arena = Bump::new();
    let mut builder = SignatureBuilder::default();
    let mut split_builder = BlockPartitionBuilder::default();

    // Put all the states in the initial partition { S }.
    let mut id: FxHashMap<Signature, usize> = FxHashMap::default();

    // Assigns the signature to each state.
    let mut partition = BlockPartition::new(lts.num_of_states());
    let mut state_to_signature: Vec<Signature> = Vec::new();
    state_to_signature.resize_with(lts.num_of_states(), Signature::default);

    // Refine partitions until stable.
    let mut iteration = 0usize;
    let mut num_of_blocks;
    let mut states = Vec::new();

    // Used to keep track of dirty blocks.
    let incoming = IncomingTransitions::new(lts);
    let mut worklist = vec![0];
    let mut stack = Vec::new();

    while let Some(block_index) = worklist.pop() {
        // Clear the current partition to start the next blocks.
        id.clear();

        num_of_blocks = partition.num_of_blocks();
        let block = partition.block(block_index);
        debug_assert!(
            block.has_marked(),
            "Every block in the worklist should have at least one marked state"
        );

        let mut new_id = 0;
        for new_block_index in partition.partition_marked_with(
            block_index,
            &mut split_builder,
            |state_index, partition| {
                // Compute the signature of a single state
                signature(state_index, partition, &state_to_signature, &mut builder);

                // Keep track of the index for every signature, either use the arena to allocate space or simply borrow the value.
                let index =
                    if let Some((signature, index)) = id.get_key_value(&Signature::new(&builder)) {
                        state_to_signature[state_index] = Signature::new(signature.as_slice());
                        *index
                    } else {
                        let slice = arena.alloc_slice_copy(&builder);
                        id.insert(Signature::new(slice), new_id);

                        // (branching)  Keep track of the signature for every block in the next partition.
                        state_to_signature[state_index] = Signature::new(slice);

                        let result = new_id;
                        new_id += 1;
                        result
                    };

                trace!("State {state_index} signature {:?} index {index}", builder);
                index
            },
        ) {
            if block_index != new_block_index {
                // If this is a new block, mark the incoming states as dirty
                states.clear();
                states.extend(partition.iter_block(new_block_index));

                for &state_index in &states {
                    for &(label_index, incoming_state) in incoming.incoming_transitions(state_index)
                    {
                        if !(BRANCHING
                            && lts.is_hidden_label(label_index)
                            && partition.block_number(incoming_state)
                                == partition.block_number(state_index))
                        {
                            mark_inert_tau(
                                lts,
                                &mut partition,
                                &mut worklist,
                                &mut stack,
                                &incoming,
                                incoming_state,
                            );
                        }
                    }

                    if BRANCHING {
                        // If we have a tau transition that becomes visible (to the original block) then mark the state as dirty.
                        for &(label_index, to) in lts.outgoing_transitions(state_index) {
                            if lts.is_hidden_label(label_index)
                                && partition.block_number(to) == block_index
                            {
                                mark_inert_tau(
                                    lts,
                                    &mut partition,
                                    &mut worklist,
                                    &mut stack,
                                    &incoming,
                                    state_index,
                                );
                            }
                        }
                    }
                }

            }
        }

        trace!("Iteration {iteration} partition {partition}");

        iteration += 1;
        if partition.num_of_blocks() % 1000 == 0 && num_of_blocks != partition.num_of_blocks() {
            // Only print a message when new blocks have been found.
            debug!(
                "Iteration {iteration}, found {} blocks",
                partition.num_of_blocks()
            );
        }
    }

    trace!("Refinement partition {partition}");
    debug_assert!(
        is_valid_refinement(lts, &partition, |state_index, partition, builder| signature(state_index, partition, &state_to_signature, builder)),
        "The resulting partition is not a valid partition."
    );
    partition
}

/// Marks the given state and all incoming (inert) tau transitions.
fn mark_inert_tau(
    lts: &LabelledTransitionSystem,
    partition: &mut BlockPartition,
    worklist: &mut Vec<usize>,
    stack: &mut Vec<usize>,
    incoming: &IncomingTransitions,
    state_index: usize,
) {
    stack.clear();
    stack.push(state_index);

    while let Some(state_index) = stack.pop() {
        let other_block = partition.block_number(state_index);

        if !partition.block(other_block).has_marked() {
            // If block was not already marked then add it to the worklist.
            worklist.push(other_block);
        }

        partition.mark_element(state_index);

        for &(label_index, incoming_state) in incoming.incoming_transitions(state_index) {
            if lts.is_hidden_label(label_index)
                && partition.block_number(state_index) == partition.block_number(incoming_state)
                && !partition.is_element_marked(incoming_state)
            {
                // If this is a bottom state then it must be marked recursively.
                trace!("Added state {incoming_state}");
                stack.push(incoming_state);
            }
        }
    }
}

/// General signature refinement algorithm that accepts an arbitrary signature
///
/// The signature function is called for each state and should fill the
/// signature builder with the signature of the state. It consists of the
/// current partition, the signatures per state for the next partition.
fn signature_refinement_naive<F>(
    lts: &LabelledTransitionSystem,
    mut signature: F,
) -> IndexedPartition
where
    F: FnMut(usize, &IndexedPartition, &Vec<Signature>, &mut SignatureBuilder),
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
    let mut state_to_signature: Vec<Signature> = Vec::new();
    state_to_signature.resize_with(lts.num_of_states(), Signature::default);

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
            signature(state_index, &partition, &state_to_signature, &mut builder);

            trace!("State {state_index} signature {:?}", builder);

            // Keep track of the index for every state, either use the arena to allocate space or simply borrow the value.
            let mut new_id = id.len();
            if let Some((signature, index)) = id.get_key_value(&Signature::new(&builder)) {
                state_to_signature[state_index] = Signature::new(signature.as_slice());
                new_id = *index;
            } else {
                let slice = arena.alloc_slice_copy(&builder);
                id.insert(Signature::new(slice), new_id);

                // Keep track of the signature for every block in the next partition.
                state_to_signature[state_index] = Signature::new(slice);
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
    debug_assert!(
        is_valid_refinement(lts, &partition, |state_index, partition, builder| signature(state_index, partition, &state_to_signature, builder)),
        "The resulting partition is not a valid partition."
    );
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
    let mut block_to_signature: Vec<Option<SignatureBuilder>> =
        vec![None; partition.num_of_blocks()];

    // Avoids reallocations when computing the signature.
    let mut builder = SignatureBuilder::default();

    for (state_index, _) in lts.iter_states() {
        let block = partition.block_number(state_index);

        // Compute the flat signature, which has Hash and is more compact.
        compute_signature(state_index, partition, &mut builder);
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

    // Check if there are two blocks with the same signature
    let mut signature_to_block: FxHashMap<Signature, usize> = FxHashMap::default();

    for block_index in 0..partition.num_of_blocks() {
        let signature = block_to_signature[block_index].as_ref().unwrap();

        if let Some(other_block_index) = signature_to_block.get(&Signature::new(signature)) {
            if block_index != *other_block_index {
                trace!(
                    "Block {block_index} and {other_block_index} have the same signature {signature:?}"
                );
                return false;
            }
        } else {
            signature_to_block.insert(Signature::new(signature), block_index);
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
