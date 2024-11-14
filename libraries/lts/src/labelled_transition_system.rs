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

    initial_state: StateIndex,

    num_of_transitions: usize,
}

impl LabelledTransitionSystem {
    pub fn new(
        initial_state: StateIndex,
        mut states: Vec<State>,
        mut labels: Vec<String>,
        hidden_labels: Vec<String>,
        num_of_transitions: usize,
    ) -> LabelledTransitionSystem {
        // Check that the number of transitions has been provided correctly.
        debug_assert!(
            states.iter().fold(0, |previous, state| previous + state.outgoing.len()) == num_of_transitions,
            "The number of transitions is not equal to the actual number of transitions."
        );

        // Check that the outgoing transitions are a function.
        if cfg!(debug_assertions) {
            let num_of_states = states.len();
            for (state_index, state) in states.iter().enumerate() {
                let mut outgoing_dedup = state.outgoing.clone();
                outgoing_dedup.sort_unstable();
                outgoing_dedup.dedup();

                debug_assert_eq!(
                    outgoing_dedup.len(),
                    state.outgoing.len(),
                    "State {state_index} has duplicated outgoing transitions {:?}",
                    state.outgoing
                );

                debug_assert!(
                    state
                        .outgoing
                        .iter()
                        .all(|(label, to)| *label < labels.len() && *to < num_of_states),
                    "State {state_index} has invalid outgoing transitions {:?}.",
                    state.outgoing
                );
            }
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

        for state in &mut states {
            for (label, _) in &mut state.outgoing {
                if let Ok(_) = hidden_indices.binary_search(label) {
                    // Remap all hidden actions to zero.
                    *label = 0;
                } 
                else if introduced_tau
                {
                    // Remap the zero action to the original first hidden index.
                    *label += 1;
                }
            }
        } 

        LabelledTransitionSystem {
            initial_state,
            labels,
            hidden_labels,
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
    pub fn outgoing_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        self.state(state_index).outgoing.iter()
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
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub outgoing: Vec<(LabelIndex, StateIndex)>,
}

impl State {
    /// Creates a new state with no outgoing transitions.
    pub fn new(outgoing: Vec<(LabelIndex, StateIndex)>) -> State {
        State { outgoing }
    }
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

        for (from, state) in self.states.iter().enumerate() {
            for (label, to) in &state.outgoing {
                let label_name = &self.labels[*label];

                writeln!(f, "{from} --[{label_name}]-> {to}")?;
            }
        }

        Ok(())
    }
}
