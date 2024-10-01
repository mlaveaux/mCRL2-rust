use std::error::Error;

use log::debug;

use crate::{LabelledTransitionSystem, State};

/// Returns a topological ordering of the states of the given LTS.
///
/// An error is returned if the LTS contains a cycle.
pub fn sort_topological<F>(lts: &LabelledTransitionSystem, filter: F) -> Result<Vec<usize>, Box<dyn Error>>
    where F: Fn(usize, usize) -> bool
{
    let start = std::time::Instant::now();

    // The resulting order of states.
    let mut stack = Vec::new();

    let mut visited = vec![false; lts.num_of_states()];
    let mut depth_stack = Vec::new();
    let mut marks = vec![None; lts.num_of_states()];

    for (state_index, _) in lts.iter_states() {
        if marks[state_index].is_none() {
            if !sort_topological_visit(
                lts,
                &filter,
                state_index,
                &mut depth_stack,
                &mut marks,
                &mut visited,
                &mut stack,
            ) {
                return Err("Graph contains a cycle".into());
            }
        }
    }

    stack.reverse();
    debug_assert!(is_topologically_sorted(lts, filter, |i| stack[i]));
    debug!("Time sort_topological: {:.3}s", start.elapsed().as_secs_f64());

    Ok(stack)
}

/// Reorders the states of the given LTS according to the given permutation.
pub fn reorder_states<P>(lts: &LabelledTransitionSystem, permutation: P) -> LabelledTransitionSystem
where
    P: Fn(usize) -> usize,
{
    let start = std::time::Instant::now();
    let mut states = vec![State::default(); lts.num_of_states()];

    for (state_index, state) in lts.iter_states() {
        let new_state_index = permutation(state_index);

        for (label, to_index) in &state.outgoing {
            let new_to_index = permutation(*to_index);
            states[new_state_index].outgoing.push((*label, new_to_index));
        }
    }

    debug!("Time reorder_states: {:.3}s", start.elapsed().as_secs_f64());
    LabelledTransitionSystem::new(
        permutation(lts.initial_state_index()),
        states,
        lts.labels().into(),
        lts.hidden_labels().into(),
        lts.num_of_transitions(),
    )
}

// The mark of a state in the depth first search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Temporary,
    Permanent,
}

/// Visits the given state in a depth first search.
///
/// Returns false if a cycle is detected.
fn sort_topological_visit<F>(
    lts: &LabelledTransitionSystem,
    filter: &F,
    state_index: usize,
    depth_stack: &mut Vec<usize>,
    marks: &mut Vec<Option<Mark>>,
    visited: &mut Vec<bool>,
    stack: &mut Vec<usize>,
) -> bool 
    where F: Fn(usize, usize) -> bool
{
    // Perform a depth first search.
    depth_stack.push(state_index);

    while let Some(state) = depth_stack.pop() {
        match marks[state] {
            None => {
                marks[state] = Some(Mark::Temporary);
                depth_stack.push(state); // Re-add to stack to mark as permanent later
                for (_, next_state) in lts.outgoing_transitions(state).filter(|(label, to)| filter(*label, *to)) {
                    // If it was marked temporary, then a cycle is detected.
                    if marks[*next_state] == Some(Mark::Temporary) {
                        return false;
                    }
                    if marks[*next_state].is_none() {
                        depth_stack.push(*next_state);
                    }
                }
            }
            Some(Mark::Temporary) => {
                marks[state] = Some(Mark::Permanent);
                visited[state] = true;
                stack.push(state);
            }
            Some(Mark::Permanent) => {}
        }
    }

    return true;
}

/// Returns true if the given permutation is a topological ordering of the states of the given LTS.
fn is_topologically_sorted<F, P>(lts: &LabelledTransitionSystem, filter: F, permutation: P) -> bool
where
    F: Fn(usize, usize) -> bool,
    P: Fn(usize) -> usize,
{
    debug_assert!(is_valid_permutation(&permutation, lts.num_of_states()));

    let mut visited = vec![false; lts.num_of_states()];
    for state_index in (0..lts.num_of_states()).map(permutation) {

        visited[state_index] = true;
        for (_, next_state) in lts.outgoing_transitions(state_index).filter(|(label, to)| filter(*label, *to)) {
            if visited[*next_state] {
                return false;
            }
        }
    }

    true
}

/// Returns true if the given permutation is a valid permutation.
fn is_valid_permutation<P>(permutation: &P, max: usize) -> bool 
where
    P: Fn(usize) -> usize,
{
    let mut visited = vec![false; max];

    for i in 0..max {
        // Out of bounds
        if permutation(i) >= max {
            return false;
        }

        if visited[permutation(i)] {
            return false;
        }

        visited[permutation(i)] = true;
    }

    // Check that all entries are visited.
    visited.iter().all(|&v| v)
}

#[cfg(test)]
mod tests {

    use rand::seq::SliceRandom;

    use crate::random_lts;

    use super::*;

    #[test]
    fn test_sort_topological_with_cycles() {
        let lts = random_lts(10, 15, 2);
        match sort_topological(&lts, |_, _| true) {
            Ok(order) => assert!(is_topologically_sorted(&lts, |_, _| true, |i| order[i])),
            Err(e) => assert_eq!(e.to_string(), "Graph contains a cycle"),
        }
    }

    #[test]
    fn test_reorder_states() {
        let lts = random_lts(10, 15, 2);

        // Generate a random permutation.
        let mut rng = rand::thread_rng();
        let order: Vec<usize> = {
            let mut order: Vec<usize> = (0..lts.num_of_states()).collect();
            order.shuffle(&mut rng);
            order
        };
        
        let new_lts = reorder_states(&lts, |i| order[i]);

        assert_eq!(new_lts.num_of_states(), lts.num_of_states());
        assert_eq!(new_lts.num_of_labels(), lts.num_of_labels());

        for (from, state) in lts.iter_states() {
            // Check that the states are in the correct order.
            for (label, to) in &state.outgoing {
                let new_from = order[from];
                let new_to = order[*to];
                assert!(new_lts.state(new_from).outgoing.contains(&(*label, new_to)));
            }
        }
    }

    #[test]
    fn test_is_valid_permutation() {
        let lts = random_lts(10, 15, 2);

        // Generate a valid permutation.
        let mut rng = rand::thread_rng();
        let valid_permutation: Vec<usize> = {
            let mut order: Vec<usize> = (0..lts.num_of_states()).collect();
            order.shuffle(&mut rng);
            order
        };

        assert!(is_valid_permutation(&|i| valid_permutation[i], valid_permutation.len()));

        // Generate an invalid permutation (duplicate entries).
        let invalid_permutation = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 8];
        assert!(!is_valid_permutation(&|i| invalid_permutation[i], invalid_permutation.len()));

        // Generate an invalid permutation (missing entries).
        let invalid_permutation = vec![0, 1, 3, 4, 5, 6, 7, 8];
        assert!(!is_valid_permutation(&|i| invalid_permutation[i], invalid_permutation.len()));
    }
}
