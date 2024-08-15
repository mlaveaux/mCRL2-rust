use log::trace;

use crate::IndexedPartition;
use crate::LabelledTransitionSystem;
use crate::Partition;

/// Computes the strongly connected tau component partitioning of the given LTS.
pub fn tau_scc_decomposition(lts: &LabelledTransitionSystem) -> IndexedPartition {
    scc_decomposition(lts, &|_, label_index, _| lts.is_hidden_label(label_index))
}

/// Computes the strongly connected component partitioning of the given LTS.
pub fn scc_decomposition<F>(lts: &LabelledTransitionSystem, filter: &F) -> IndexedPartition
where
    F: Fn(usize, usize, usize) -> bool,
{
    trace!("{:?}", lts);

    let mut partition = IndexedPartition::new(lts.num_of_states());

    // The stack for the depth first search.
    let mut stack = Vec::new();

    // Keep track of already visited states.
    let mut indices: Vec<Option<StateInfo>> = vec![None; lts.num_of_states()];

    let mut smallest_index = 0;

    // The outer depth first search used to traverse all the states.
    for (state_index, _) in lts.iter_states() {
        if indices[state_index].is_none() {
            trace!("Current partition {partition}");
            strongly_connect(
                state_index,
                lts,
                filter,
                &mut partition,
                &mut smallest_index,
                &mut stack,
                &mut indices,
            )
        }
    }

    trace!("Final partition {partition}");
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
    stack: &mut Vec<usize>,
    indices: &mut Vec<Option<StateInfo>>,
) where
    F: Fn(usize, usize, usize) -> bool,
{
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
        // NOTE: We start with a single block, but since we override all the indices anyway its safe to start with zero.
        let new_block = partition.num_of_blocks() - 1;
        trace!("Introduced new SCC {new_block}");
        partition.set_block(state_index, new_block);

        while let Some(index) = stack.pop() {
            let info = indices[index].as_mut().expect("This state was on the stack");
            info.on_stack = false;
            partition.set_block(index, new_block);
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

    use super::*;

    #[test]
    fn test_random_scc_decomposition() {
        let lts = random_lts(10, 3, 3);
        let reduction = quotient_lts(&lts, &tau_scc_decomposition(&lts));

        assert!(
            !has_tau_loop(&reduction),
            "The tau-SCC decomposition still contains tau loops"
        );

        assert!(
            reduction.num_of_states() == tau_scc_decomposition(&reduction).num_of_blocks(),
            "Applying SCC decomposition should yield the same number of SCC after second application"
        );
    }
}
