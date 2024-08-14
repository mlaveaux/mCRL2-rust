use log::trace;

use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::SigrefPartition;

/// Returns true iff the given label index is a hidden label.
pub fn is_hidden_label(label_index: LabelIndex) -> bool {
    label_index == 0
}

/// Computes the tau-star partition of the given LTS.
pub fn tau_star_partition(lts: &LabelledTransitionSystem) -> SigrefPartition {
    trace!("{:?}", lts);

    let mut partition = SigrefPartition::new(lts.num_of_states());

    // Put every state into its own block.
    let mut index: usize = 0;
    for entry in partition.iter_mut() {
        *entry = index;
        index += 1;
    }

    // The inner stack for the depth first search for tau actions.
    let mut stack = Vec::new();

    // Keep track of already visited states.
    let mut visited = vec![false; lts.num_of_states()];

    // The outer depth first search used to traverse all the states.
    for (state_index, _) in lts.iter_states() {
        trace!("current partition {partition:?}");

        // Start a depth first search from the current state.
        stack.push(state_index);
        visited[state_index] = true;

        while let Some(inner_state_index) = stack.pop() {
            // We should only search up the stack, ignore current entries.
            let stack_length = stack.len();

            for (label_index, to_index) in lts.outgoing_transitions(inner_state_index) {
                if is_hidden_label(*label_index) {
                    if let Some(previous_index) = stack[0..stack_length].iter().position(|element| element == to_index) {

                        // Every state on the stack starting here is reachable in a cycle
                        for index in &stack[previous_index..stack_length] {
                            partition.set_block(*index, state_index);
                        }
                    }

                    // Explore all the states reachable with hidden actions.
                    if !visited[*to_index] {
                        visited[*to_index] = true;
                        stack.push(*to_index);
                    }
                }
            }
        }
    }

    trace!("final partition {partition:?}");
    partition
}

/// Returns true iff the labelled transition system has tau-loops.
pub fn has_tau_loop(lts: &LabelledTransitionSystem) -> bool {
    // The inner stack for the depth first search of tau actions.
    let mut stack = Vec::new();

    // Keep track of already visited states.
    let mut visited = vec![false; lts.num_of_states()];

    for (state_index, _) in lts.iter_states() {

        // Start a depth first search from the current state.
        stack.push(state_index);
        visited[state_index] = true;

        while let Some(inner_state_index) = stack.pop() {

            // We should only search up the stack, ignore current entries.
            let stack_length = stack.len();

            for (label_index, to_index) in lts.outgoing_transitions(inner_state_index) {
                if is_hidden_label(*label_index) {
                    if stack[0..stack_length].contains(&to_index) {
                        // There is state where following tau path leads back
                        // into the stack.
                        trace!("tau-loop {:?} to {to_index}", stack);
                        return true;
                    }

                    // Explore all the states reachable with hidden actions.
                    if !visited[*to_index] {
                        visited[*to_index] = true;
                        stack.push(*to_index);
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::quotient_lts;
    use crate::random_lts;
    use crate::IndexedPartition;

    use super::*;

    #[test]
    fn test_random_bisim_sigref() {
        let lts = random_lts(10, 3, 3);
        let reduction = quotient_lts(&lts, &tau_star_partition(&lts));

        assert!(!has_tau_loop(&reduction), "The tau-star quotient still contains tau loops");

        assert!(
            reduction.num_of_states() == tau_star_partition(&reduction).num_of_blocks(),
            "Applying tau star reduction should have removed all tau-loops"
        );
    }
}
