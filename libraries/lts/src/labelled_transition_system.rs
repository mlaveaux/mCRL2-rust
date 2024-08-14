use std::fmt;

/// The index type for a label.
pub type LabelIndex = usize;

/// The index for a state.
pub type StateIndex = usize;

/// Represents a labelled transition system consisting of states with directed
/// labelled edges.
#[derive(PartialEq, Eq)]
pub struct LabelledTransitionSystem {
    states: Vec<State>,

    labels: Vec<String>,
    hidden_labels: Vec<String>,
    hidden_indices: Vec<usize>,

    initial_state: StateIndex,

    num_of_transitions: usize,
}

impl LabelledTransitionSystem {
    pub fn new(
        initial_state: StateIndex,
        states: Vec<State>,
        labels: Vec<String>,
        hidden_labels: Vec<String>,
        num_of_transitions: usize,
    ) -> LabelledTransitionSystem {
        // Check that the number of transitions has been computed correctly.
        debug_assert!(
            states
                .iter()
                .fold(0, |previous, state| previous + state.outgoing.len())
                == num_of_transitions,
            "The number of transitions is not equal to the actual number of transitions."
        );

        let mut hidden_indices: Vec<usize> = Vec::new();
        for label in &hidden_labels {
            if let Some(index) = labels.iter().position(|other| {
                other == label
            }) {
                hidden_indices.push(index);             
            }
        };
        hidden_indices.sort();

        LabelledTransitionSystem {
            initial_state,
            labels,
            hidden_labels,
            hidden_indices,
            states,
            num_of_transitions,
        }
    }

    /// Returns the index of the initial state
    pub fn initial_state_index(&self) -> StateIndex {
        self.initial_state
    }

    /// Returns a borrow of the initial state.
    pub fn initial_state(&self) -> &State {
        &self.states[self.initial_state]
    }

    /// Returns the set of outgoing transitions for the given state.
    pub fn outgoing_transitions<'a>(
        &'a self,
        state_index: usize,
    ) -> impl Iterator<Item = &(LabelIndex, StateIndex)> + 'a {
        self.state(state_index)
            .outgoing
            .iter()
    }

    /// Iterate over all (state_index, state) in the labelled transition system
    pub fn iter_states(&self) -> impl Iterator<Item = (StateIndex, &State)> + '_ {
        self.states.iter().enumerate()
    }

    /// Returns access to the given state.
    pub fn state(&self, index: StateIndex) -> &State {
        &self.states[index]
    }

    /// Returns the number of states.
    pub fn num_of_states(&self) -> StateIndex {
        self.states.len()
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
        self.hidden_indices.binary_search(&label_index).is_ok()
    }
}

/// A single state in the LTS, containing a vector of outgoing edges.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub outgoing: Vec<(LabelIndex, StateIndex)>,
}

impl fmt::Display for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print some information about the LTS.
        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Number of action labels: {}", self.labels.len())?;
        writeln!(f, "Number of transitions: {}", self.num_of_transitions)
    }
}

impl fmt::Debug for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Initial state: {}", self.initial_state)?;
        writeln!(f, "Number of transitions: {}", self.num_of_transitions)?;

        for (from, state) in self.states.iter().enumerate() {
            for (label, to) in &state.outgoing {
                let label_name = &self.labels[*label];

                writeln!(f, "{from} --[{label_name}]-> {to}")?;
            }
        }

        Ok(())
    }
}
