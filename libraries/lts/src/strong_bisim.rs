use std::{collections::{HashMap, HashSet}, mem::swap};

use log::{debug, trace};

use crate::{LabelledTransitionSystem, State};

/// The type of a signature for strong bisimulation. We use sorted vectors to
/// avoid the overhead of hash sets that might have unused values.
type Signature = Vec<(usize, usize)>;

/// Returns the signature for strong bisimulation sig(s) = { (a, pi(t)) | s -a-> t in T }
fn compute_strong_bisim_signature(state: &State, partition: &Vec<usize>) -> Signature {    
    // Compute the signature of a single state
    let mut signature: HashSet::<(usize, usize)> = HashSet::new();

    for (label, to) in &state.outgoing {
        signature.insert((*label, partition[*to]));
    }

    // Compute the flat signature, which has Hash and is more compact.
    let mut signature_flat: Vec<(usize, usize)> = signature.drain().collect();
    signature_flat.sort_unstable();

    signature_flat

}

/// Computes a strong bisimulation quotient using signature refinement
pub fn strong_bisim_sigref(lts: &LabelledTransitionSystem) {

    // Put all the states in the initial partition { S }.
    let mut id: HashMap<Signature, usize> = HashMap::new();

    // Assigns the signature to each state.
    let mut partition: Vec<usize> = vec![0; lts.states.len()];
    let mut next_partition: Vec<usize> = vec![0; lts.states.len()];

    // Refine partitions until stable.
    let mut old_count = 1;
    let mut iteration = 0;

    while old_count != id.len() {
        old_count = id.len();
        debug!("Iteration {iteration}, found {old_count} blocks");
        swap(&mut partition, &mut next_partition);

        // Clear the current partition to start the next blocks.
        id.clear();

        for (state_index, state) in lts.states.iter().enumerate() {

            // Compute the signature of a single state
            let signature = compute_strong_bisim_signature(state, &partition);

            // Keep track of the index for every state.
            let mut new_id = id.len();
            id.entry(signature.clone())
                .and_modify(|n| {
                    next_partition[state_index] = *n;
                    new_id = *n;
                })
                .or_insert_with(|| {
                    next_partition[state_index] = new_id;
                    new_id
                });

            trace!("State {state_index} signature {new_id}:{signature:?}");
        }

        iteration += 1;

        debug_assert!(iteration <= lts.states.len(), "There can never be more splits than number of states");
    }


    // Check that the partition is indeed stable and as such is a quotient of strong bisimulation
    let mut representative: Vec<usize> = Vec::new();
    for (state_index, state) in lts.states.iter().enumerate() {
        let block = next_partition[state_index];

        if block + 1 > representative.len() {
            representative.resize(block + 1, 0);
            representative[block] = state_index;
        }

        // Check that this block only contains states that are strongly bisimilar to the representative state.
        let representative_index = representative[block];
        let signature = compute_strong_bisim_signature(state, &partition);
        let representative_signature = compute_strong_bisim_signature(&lts.states[representative_index], &partition);

        debug_assert_eq!(signature, representative_signature, "State {state_index} has a different signature then representative state {representative_index}, but are in the same block {block}");
    }
}

#[cfg(test)]
mod tests {
    use log::trace;
    use test_log::test;
    
    use crate::random_lts;

    use super::*;

    #[test]
    fn test_random_bisim() {
        let lts = random_lts(10, 3);

        trace!("{lts:?}");

        strong_bisim_sigref(&lts);
    }
}