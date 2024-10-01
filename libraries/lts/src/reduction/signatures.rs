use std::fmt::Debug;
use std::hash::Hash;

use fxhash::FxHashSet;

use crate::LabelledTransitionSystem;
use crate::Partition;
use crate::StateIndex;

/// The builder used to construct the signature.
pub type SignatureBuilder = Vec<(usize, usize)>;

/// The type of a signature. We use sorted vectors to avoid the overhead of hash
/// sets that might have unused values.
#[derive(Eq)]
pub struct Signature(*const [(usize, usize)]);

impl Signature {
    pub fn new(slice: &[(usize, usize)]) -> Signature {
        Signature(slice)
    }

    pub fn as_slice(&self) -> &[(usize, usize)] {
        unsafe { &*self.0 }
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*self.0).hash(state) }
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

/// Returns the signature for strong bisimulation sig(s, pi) = { (a, pi(t)) | s -a-> t in T }
pub fn strong_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
) {
    builder.clear();
    
    for (label, to) in lts.outgoing_transitions(state_index) {
        builder.push((*label, partition.block_number(*to)));
    }
}

/// Returns the branching bisimulation signature for branching bisimulation
/// sig(s, pi) = { (a, pi(t)) | s -[tau]-> s1 -> ... s_n -[a]-> t in T && pi(s) = pi(s_i) && ((a != tau) || pi(s) != pi(t)) }
pub fn branching_bisim_signature(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    builder: &mut SignatureBuilder,
    visited: &mut FxHashSet<usize>,
    stack: &mut Vec<usize>,
) {

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
                    // Explore the outgoing state as well, still tau path in same block
                    if !visited.contains(to_index) {
                        visited.insert(*to_index);
                        stack.push(*to_index);
                    }
                }
                else {
                    //  pi(s) != pi(t)
                    builder.push((*label_index, partition.block_number(*to_index)));
                }
            } else {
                // (a != tau) This is a visible action only reachable from tau paths with equal signatures.
                builder.push((*label_index, partition.block_number(*to_index)));
            }
        }
    }
}
