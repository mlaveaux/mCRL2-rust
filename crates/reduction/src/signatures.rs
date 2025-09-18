use std::fmt::Debug;
use std::hash::Hash;

use rustc_hash::FxHashSet;

use mcrl2rust_lts::LabelledTransitionSystem;
use mcrl2rust_lts::StateIndex;

use crate::Partition;
use crate::quotient_lts;
use crate::reorder_partition;
use crate::reorder_states;
use crate::sort_topological;
use crate::tau_scc_decomposition;
use crate::BlockPartition;
use crate::IndexedPartition;

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CompactSignaturePair(u64);

impl Debug for CompactSignaturePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.label(), self.state())
    }
}

impl CompactSignaturePair {
    #[inline]
    pub fn new(label: usize, state: usize) -> Self {
        Self(((label as u64) << 48) | (state as u64 & 0xFFFF_FFFF_FFFF))
    }

    #[inline]
    pub fn label(&self) -> usize {
        (self.0 >> 48) as usize
    }

    #[inline]
    pub fn state(&self) -> usize {
        (self.0 & 0xFFFF_FFFF_FFFF) as usize
    }
}
/// The builder used to construct the signature.
pub type SignatureBuilder = Vec<CompactSignaturePair>;

/// The type of a signature. We use sorted vectors to avoid the overhead of hash
/// sets that might have unused values.
#[derive(Eq)]
pub struct Signature(*const [CompactSignaturePair]);

impl Signature {
    pub fn new(slice: &[CompactSignaturePair]) -> Signature {
        Signature(slice)
    }

    pub fn as_slice(&self) -> &[CompactSignaturePair] {
        unsafe { &*self.0 }
    }
}

impl Hash for CompactSignaturePair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl Signature {
    // Check if target is a subset of self, excluding a specific element
    pub fn is_subset_of(&self, other: &[CompactSignaturePair], exclude: CompactSignaturePair) -> bool {
        let mut self_iter = self.as_slice().iter();
        let mut other_iter = other.iter().filter(|&&x| x != exclude);

        let mut self_item = self_iter.next();
        let mut other_item = other_iter.next();

        while let Some(&o) = other_item {
            match self_item {
                Some(&s) if s == o => {
                    // Match found, move both iterators forward
                    self_item = self_iter.next();
                    other_item = other_iter.next();
                }
                Some(&s) if s < o => {
                    // Move only self iterator forward
                    self_item = self_iter.next();
                }
                _ => {
                    // No match found in self for o
                    return false;
                }
            }
        }
        // If we finished self_iter without returning false, self is a subset
        true
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature(&[])
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

    for transition in lts.outgoing_transitions_compact(state_index) {
        builder.push(CompactSignaturePair::new(transition.label(), partition.block_number(transition.state())));
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
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

        for transition in lts.outgoing_transitions_compact(inner_state_index) {
            if lts.is_hidden_label(transition.label()) {
                if partition.block_number(state_index) == partition.block_number(transition.state()) {
                    // Explore the outgoing state as well, still tau path in same block
                    if !visited.contains(&transition.state()) {
                        visited.insert(transition.state());
                        stack.push(transition.state());
                    }
                } else {
                    //  pi(s) != pi(t)
                    builder.push(CompactSignaturePair::new(transition.label(), partition.block_number(transition.state())));
                }
            } else {
                // (a != tau) This is a visible action only reachable from tau paths with equal signatures.
                builder.push(CompactSignaturePair::new(transition.label(), partition.block_number(transition.state())));
            }
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// The input lts must contain no tau-cycles.
pub fn branching_bisim_signature_sorted(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &impl Partition,
    state_to_signature: &[Signature],
    builder: &mut SignatureBuilder,
) {
    builder.clear();

    for transition in lts.outgoing_transitions_compact(state_index) {
        let to_block = partition.block_number(transition.state());

        if partition.block_number(state_index) == to_block {
            if lts.is_hidden_label(transition.label()) {
                // Inert tau transition, take signature from the outgoing tau-transition.
                builder.extend(state_to_signature[transition.state()].as_slice());
            } else {
                builder.push(CompactSignaturePair::new(transition.label(), to_block));
            }
        } else {
            // Visible action, add to the signature.
            builder.push(CompactSignaturePair::new(transition.label(), to_block));
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// The input lts must contain no tau-cycles.
pub fn branching_bisim_signature_inductive(
    state_index: StateIndex,
    lts: &LabelledTransitionSystem,
    partition: &BlockPartition,
    state_to_key: &[usize],
    builder: &mut SignatureBuilder,
) {
    builder.clear();

    let num_act: usize = lts.num_of_labels(); //this label index does not occur.
    for (label_index, to) in lts.outgoing_transitions(state_index) {
        let to_block = partition.block_number(to);

        if partition.block_number(state_index) == to_block {
            if lts.is_hidden_label(label_index) && partition.is_element_marked(to) {
                // Inert tau transition, take signature from the outgoing tau-transition.
                builder.push(CompactSignaturePair::new(num_act, state_to_key[to]));
            } else {
                builder.push(CompactSignaturePair::new(label_index, to_block));
            }
        } else {
            // Visible action, add to the signature.
            builder.push(CompactSignaturePair::new(label_index, to_block));
        }
    }

    // Compute the flat signature, which has Hash and is more compact.
    builder.sort_unstable();
    builder.dedup();
}

/// Perform the preprocessing necessary for branching bisimulation with the
/// sorted signature see `branching_bisim_signature_sorted`.
pub fn preprocess_branching(lts: &LabelledTransitionSystem) -> (LabelledTransitionSystem, IndexedPartition) {
    let scc_partition = tau_scc_decomposition(lts);
    let tau_loop_free_lts = quotient_lts(lts, &scc_partition, true);

    // Sort the states according to the topological order of the tau transitions.
    let topological_permutation = sort_topological(
        &tau_loop_free_lts,
        |label_index, _| tau_loop_free_lts.is_hidden_label(label_index),
        true,
    )
    .expect("After quotienting, the LTS should not contain cycles");

    (
        reorder_states(&tau_loop_free_lts, |i| topological_permutation[i]),
        reorder_partition(scc_partition, |i| topological_permutation[i]),
    )
}
