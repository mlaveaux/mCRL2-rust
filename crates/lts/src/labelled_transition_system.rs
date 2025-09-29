use std::fmt;

/// The index type for a label.
pub type LabelIndex = usize;

/// The index for a state.
pub type StateIndex = usize;

/// A compact representation of a transition using a single u64.
/// The high 16 bits store the label index, the low 48 bits store the target state index.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct CompactTransition(u64);

impl CompactTransition {
    #[inline]
    pub fn new(label: LabelIndex, state: StateIndex) -> Self {
        debug_assert!(label < (1 << 16), "Label index too large for compact representation");
        debug_assert!(state < (1 << 48), "State index too large for compact representation");
        Self(((label as u64) << 48) | (state as u64 & 0xFFFF_FFFF_FFFF))
    }

    #[inline]
    pub fn label(&self) -> LabelIndex {
        (self.0 >> 48) as LabelIndex
    }

    #[inline]
    pub fn state(&self) -> StateIndex {
        (self.0 & 0xFFFF_FFFF_FFFF) as StateIndex
    }

    #[inline]
    pub fn to_tuple(&self) -> (LabelIndex, StateIndex) {
        (self.label(), self.state())
    }
}

impl From<(LabelIndex, StateIndex)> for CompactTransition {
    fn from(tuple: (LabelIndex, StateIndex)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl fmt::Debug for CompactTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.label(), self.state())
    }
}

/// Represents a labelled transition system consisting of states with directed
/// labelled edges.
#[derive(PartialEq, Eq)]
pub struct LabelledTransitionSystem {
    states: Vec<State>,
    transitions: Vec<CompactTransition>,

    labels: Vec<String>,
    hidden_labels: Vec<String>,

    initial_state: StateIndex,

    num_of_transitions: usize,
}

impl LabelledTransitionSystem {

    /// Creates a new a labelled transition system with the given transitions, labels, and hidden labels.
    /// 
    /// The initial state is the state with the given index.
    /// num_of_states is the number of states in the LTS, if known. If None then deadlock states without incoming transitions are removed.
    pub fn new<I, F>(
        initial_state: StateIndex,
        num_of_states: Option<usize>,
        transition_iter: F,
        mut labels: Vec<String>,
        hidden_labels: Vec<String>,
    ) -> LabelledTransitionSystem 
    where F: Fn() -> I,
          I:Iterator<Item = (StateIndex, CompactTransition)> {

        let mut states = Vec::new();
        if let Some(num_of_states) = num_of_states {
            states.resize_with(num_of_states, Default::default);
        }

        // Count the number of transitions for every state
        let mut num_of_transitions = 0;
        for (from, trans) in transition_iter() {
            // Ensure that the states vector is large enough.
            while states.len() <= from.max(trans.state()) {
                states.push(State::default());
            }

            states[from].outgoing_end += 1;
            num_of_transitions += 1;
        }

        // Track the number of transitions before every state.
        states.iter_mut().fold(0, |count, state| {
            let result = count + state.outgoing_end;
            state.outgoing_start = count;
            state.outgoing_end = count;
            result
        });
        
        // Place the transitions, and increment the end for every state.
        let mut transitions = vec![CompactTransition::new(0, 0); num_of_transitions];
        for (from, trans) in transition_iter() {
            transitions[states[from].outgoing_end] = trans;
            states[from].outgoing_end += 1;
        }

        // Keep track of which label indexes are hidden labels.
        let mut hidden_indices: Vec<usize> = Vec::new();
        for label in &hidden_labels {
            if let Some(index) = labels.iter().position(|other| other == label) {
                hidden_indices.push(index);
            }
        }
        hidden_indices.sort();

        // Make an implicit tau label the first label.
        let introduced_tau = if hidden_indices.contains(&0) {
            labels[0] = "tau".to_string();
            false
        } else {
            labels.insert(0, "tau".to_string());
            true
        };

        // Remap all hidden actions to zero.
        for state in &mut states {
            for transition in &mut transitions[state.outgoing_start..state.outgoing_end] {
                let mut label = transition.label();
                let state = transition.state();
                if hidden_indices.binary_search(&label).is_ok() {
                    label = 0;
                } 
                else if introduced_tau {
                    // Remap the zero action to the original first hidden index.
                    label += 1;
                }
                *transition = CompactTransition::new(label, state);
            }
            transitions[state.outgoing_start..state.outgoing_end].sort_unstable();
        } 
     
        LabelledTransitionSystem {
            initial_state,
            labels,
            hidden_labels,
            states,
            num_of_transitions: transitions.len(),
            transitions,
        }
    }

    pub fn new_from_permutation<P>(
        lts: &LabelledTransitionSystem,
        permutation: P,
    ) -> Self
    where
        P: Fn(usize) -> usize + Copy,
    {
        let mut states = vec![State::default(); lts.num_of_states()];
        for state_index in lts.iter_states() {
            let new_state_index = permutation(state_index);
            let state = &lts.states[state_index];
            states[new_state_index].outgoing_start = state.outgoing_start;
            states[new_state_index].outgoing_end = state.outgoing_end;
        }


        LabelledTransitionSystem {
            initial_state: permutation(lts.initial_state),
            labels: lts.labels.clone(),
            hidden_labels: lts.hidden_labels.clone(),
            states: states,
            num_of_transitions: lts.transitions.len(),
            transitions: lts.transitions.clone(),
        }
    }

    /// Returns the index of the initial state
    pub fn initial_state_index(&self) -> StateIndex {
        self.initial_state
    }

    /// Returns the set of outgoing transitions for the given state.
    pub fn outgoing_transitions(&self, state_index: usize) -> impl Iterator<Item = (LabelIndex, StateIndex)> + '_ {
        let state = &self.states[state_index];
        self.transitions[state.outgoing_start..state.outgoing_end]
            .iter()
            .map(CompactTransition::to_tuple)
    }

    pub fn outgoing_transitions_compact(&self, state_index: usize) -> &[CompactTransition] {
        let state = &self.states[state_index];
        &self.transitions[state.outgoing_start..state.outgoing_end]
    }

    /// Iterate over all state_index in the labelled transition system
    pub fn iter_states(&self) -> impl Iterator<Item = StateIndex> {
        0..self.states.len()
    }

    /// Returns the number of states.
    pub fn num_of_states(&self) -> StateIndex {
        self.states.len()
    }

    /// Returns the number of labels.
    pub fn num_of_labels(&self) -> LabelIndex {
        self.labels.len()
    }

    /// Returns the number of transitions.
    pub fn num_of_transitions(&self) -> usize {
        self.num_of_transitions
    }

    /// Returns the list of labels.
    pub fn labels(&self) -> &[String] {
        &self.labels[0..]
    }

    /// Returns the list of hidden labels.
    pub fn hidden_labels(&self) -> &[String] {
        &self.hidden_labels[0..]
    }

    /// Returns true iff the given label index is a hidden label.
    pub fn is_hidden_label(&self, label_index: LabelIndex) -> bool {
        label_index == 0
    }
}

/// A single state in the LTS, containing a vector of outgoing edges.
#[derive(Clone, Default, PartialEq, Eq)]
struct State {
    outgoing_start: usize,
    outgoing_end: usize,
}

impl fmt::Display for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print some information about the LTS.
        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Number of action labels: {}", self.labels.len())?;
        write!(f, "Number of transitions: {}", self.num_of_transitions)
    }
}

impl fmt::Debug for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self)?;
        writeln!(f, "Initial state: {}", self.initial_state)?;
        writeln!(f, "Hidden labels: {:?}", self.hidden_labels)?;

        for state_index in self.iter_states() {
            for (label, to) in self.outgoing_transitions(state_index) {
                let label_name = &self.labels[label];

                writeln!(f, "{state_index} --[{label_name}]-> {to}")?;
            }
        }

        Ok(())
    }
}
