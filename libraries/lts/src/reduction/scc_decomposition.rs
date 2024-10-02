use log::debug;
use log::trace;

use crate::quotient_lts;
use crate::reduction::sort_topological;
use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;

/// Computes the strongly connected tau component partitioning of the given LTS.
pub fn tau_scc_decomposition(lts: &LabelledTransitionSystem) -> IndexedPartition {
    let partition = scc_decomposition(lts, &|_, label_index, _| lts.is_hidden_label(label_index));
    let quotient_lts = quotient_lts(lts, &partition, true);
    debug_assert!(sort_topological(&quotient_lts, |label_index, _| lts.is_hidden_label(label_index), false).is_ok(), "The SCC decomposition contains tau-loops");
    partition
}

/// Computes the strongly connected component partitioning of the given LTS.
pub fn scc_decomposition<F>(lts: &LabelledTransitionSystem, filter: &F) -> IndexedPartition
where
    F: Fn(usize, usize, usize) -> bool,
{
    let start = std::time::Instant::now();
    trace!("{:?}", lts);

    let mut partition = IndexedPartition::new(lts.num_of_states());

    // The stack for the depth first search.
    let mut stack = Vec::new();

    // Keep track of already visited states.
    let mut indices: Vec<Option<StateInfo>> = vec![None; lts.num_of_states()];

    let mut smallest_index = 0;
    let mut next_block_number = 0;

    // The outer depth first search used to traverse all the states.
    for (state_index, _) in lts.iter_states() {
        if indices[state_index].is_none() {
            trace!("State {state_index}");
    
            strongly_connect(
                state_index,
                lts,
                filter,
                &mut partition,
                &mut smallest_index,
                &mut next_block_number,
                &mut stack,
                &mut indices,
            )
        }
    }

    trace!("Final partition {partition}");
    debug!("Found {} strongly connected components", partition.num_of_blocks());
    debug!("Time scc_decomposition: {:.3}s", start.elapsed().as_secs_f64());
    partition
}

#[derive(Clone, Debug)]
struct StateInfo {
    /// A unique index for every state.
    index: usize,

    /// Keeps track of the lowest state that can be reached on the stack.
    lowlink: usize,

    /// Keeps track of whether this state is on the stack.
    on_stack: bool,
}

/// Tarjan's strongly connected components algorithm.
///
/// The `filter` can be used to determine which (from, label, to) edges should
/// to be connected.
///
/// The `smallest_index`, `stack` and `indices` are updated in each recursive
/// call to keep track of the current SCC.
fn strongly_connect<F>(
    state_index: usize,
    lts: &LabelledTransitionSystem,
    filter: &F,
    partition: &mut IndexedPartition,
    smallest_index: &mut usize,
    next_block_number: &mut usize,
    stack: &mut Vec<usize>,
    indices: &mut Vec<Option<StateInfo>>,
) where
    F: Fn(usize, usize, usize) -> bool,
{    
    trace!("Visiting state {state_index}");

    indices[state_index] = Some(StateInfo {
        index: *smallest_index,
        lowlink: *smallest_index,
        on_stack: true,
    });

    *smallest_index += 1;

    // Start a depth first search from the current state.
    stack.push(state_index);

    // Consider successors of the current state.
    for (label_index, to_index) in lts.outgoing_transitions(state_index) {
        if filter(state_index, *label_index, *to_index) {
            if let Some(meta) = &mut indices[*to_index] {
                if meta.on_stack { 
                    // Successor w is in stack S and hence in the current SCC
                    // If w is not on stack, then (v, w) is an edge pointing to an SCC already found and must be ignored
                    // v.lowlink := min(v.lowlink, w.lowlink);
                    let w_lowlink = indices[*to_index]
                        .as_ref()
                        .expect("The state must be visited in the recursive call")
                        .index;
                    let info = indices[state_index].as_mut().expect("This state was added before");
                    info.lowlink = info.lowlink.min(w_lowlink);
                }
            } else {
                // Successor w has not yet been visited; recurse on it
                strongly_connect(
                    *to_index,
                    lts,
                    filter,
                    partition,
                    smallest_index,
                    next_block_number,
                    stack,
                    indices,
                );

                // v.lowlink := min(v.lowlink, w.lowlink);
                let w_lowlink = indices[*to_index]
                    .as_ref()
                    .expect("The state must be visited in the recursive call")
                    .index;
                let info = indices[state_index].as_mut().expect("This state was added before");
                info.lowlink = info.lowlink.min(w_lowlink);
            }
        }
    }

    let info = indices[state_index].as_ref().expect("This state was added before");
    if info.lowlink == info.index {
        // Start a new strongly connected component.
        while let Some(index) = stack.pop() {
            let info = indices[index].as_mut().expect("This state was on the stack");
            info.on_stack = false;

            trace!("Added state {index} to block {}", next_block_number);
            partition.set_block(index, *next_block_number);

            if index == state_index || stack.is_empty() {
                *next_block_number += 1;
                break;
            }
        }
    }
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
            // We should only search up the stack, ignoring added entries.
            let stack_length = stack.len();

            for (label_index, to_index) in lts.outgoing_transitions(inner_state_index) {
                if lts.is_hidden_label(*label_index) {
                    if state_index == *to_index {
                        // There is a tau selfloop;
                        trace!("tau self-loop for {to_index}");
                        return true;
                    }

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
    use crate::Partition;
    use crate::State;

    use super::*;

    /// Returns the reachable states from the given state index.
    fn reachable_states(
        lts: &LabelledTransitionSystem,
        state_index: usize,
        filter: &impl Fn(usize, usize, usize) -> bool,
    ) -> Vec<usize> {
        let mut stack = vec![state_index];
        let mut visited = vec![false; lts.num_of_states()];

        // Depth first search to find all reachable states.
        while let Some(inner_state_index) = stack.pop() {
            for (_, to_index) in lts.outgoing_transitions(inner_state_index) {
                if filter(inner_state_index, 0, *to_index) && !visited[*to_index] {
                    visited[*to_index] = true;
                    stack.push(*to_index);
                }
            }
        }

        // All the states that were visited are reachable.
        visited
            .into_iter()
            .enumerate()
            .filter_map(|(index, visited)| if visited { Some(index) } else { None })
            .collect()
    }

    #[test]
    fn test_random_tau_scc_decomposition() {
        let lts = random_lts(10, 3, 3);
        let partitioning = tau_scc_decomposition(&lts);
        let reduction = quotient_lts(&lts, &partitioning, true);
        trace!("{:?}", reduction);

        assert!(
            !has_tau_loop(&reduction),
            "The tau-SCC decomposition still contains tau loops"
        );

        // Check that states in a strongly connected component are reachable from each other.
        for (state_index, _) in lts.iter_states() {
            let reachable = reachable_states(&reduction, state_index, &|_, label, _| lts.is_hidden_label(label));

            // All other states in the same block should be reachable.
            let block = partitioning.block_number(state_index);

            for (other_state_index, _) in lts.iter_states().filter(|(index, _)| state_index != *index && partitioning.block_number(*index) == block) {
                assert!(
                    reachable.contains(&other_state_index),
                    "State {state_index} and {other_state_index} should be reachable"
                );
            }
        }   

        assert!(
            reduction.num_of_states() == tau_scc_decomposition(&reduction).num_of_blocks(),
            "Applying SCC decomposition again should yield the same number of SCC after second application"
        );
    }

    #[test]
    fn test_cycles() {
        let states = vec![
            State::new(vec![(0, 2), (0, 4)]),
            State::new(vec![(0, 0)]),
            State::new(vec![(0, 1),(1, 0)]),
            State::new(vec![]),
            State::new(vec![]),
        ];

        let lts = LabelledTransitionSystem::new(0, states, vec!["tau".into(), "a".into()], vec!["tau".into()], 5);

        let _ = tau_scc_decomposition(&lts);
    }
}
