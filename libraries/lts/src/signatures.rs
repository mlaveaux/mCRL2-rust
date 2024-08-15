use ahash::AHashSet;

use crate::Partition;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

/// The builder used to construct the signature.
pub type SignatureBuilder = AHashSet<(usize, usize)>;

/// The type of a signature. We use sorted vectors to avoid the overhead of hash
/// sets that might have unused values.
pub type Signature = Vec<(usize, usize)>;

/// Returns the signature for strong bisimulation sig(s, pi) = { (a, pi(t)) | s -a-> t in T }
pub fn strong_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
) -> Signature {
    for (label, to) in lts.outgoing_transitions(state_index) {
        builder.insert((*label, partition.block_number(*to)));
    }

    // Compute the flat signature, which has Hash and is more compact.
    let mut signature_flat: Signature = builder.drain().collect();
    signature_flat.sort_unstable();

    signature_flat
}

/// Returns the pre-signature for branching bisimulation sig(s, pi) = { (a, pi(t)) | s -[tau]-> s1 -> ... s_n -[a]-> t in T && pi(s) = pi(i) && (a != tau) && pi(s) != pi(t) }
pub fn branching_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
    visited: &mut AHashSet<usize>,
    stack: &mut Vec<usize>,
) -> Signature {

    // Clear the builders and the list of visited states.
    builder.clear();
    visited.clear();

    // A stack used for depth first search of tau paths.
    debug_assert!(stack.is_empty(), "The stack should be empty");
    stack.push(state_index);

    while let Some(inner_state_index) = stack.pop() {
        visited.insert(inner_state_index);

        for (label_index, to_index) in lts.outgoing_transitions(inner_state_index) {
            if lts.is_hidden_label(*label_index) {
                if partition.block_number(state_index) == partition.block_number(*to_index) {
                    // Explore the outgoing state as well.
                    if !visited.contains(to_index) {
                        visited.insert(*to_index);
                        stack.push(*to_index);
                    }
                } else {
                    builder.insert((*label_index, partition.block_number(*to_index)));                    
                }
            } else {
                // This is a visible action only reachable from tau paths with equal signatures.
                builder.insert((*label_index, partition.block_number(*to_index)));
            }
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    let mut signature_flat: Signature = builder.drain().collect();
    signature_flat.sort_unstable();

    signature_flat
}
