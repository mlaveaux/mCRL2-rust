use std::fmt;

/// The index type for a label.
pub type LabelIndex = usize;

/// The index for a state.
pub type StateIndex = usize;

/// Represents a labelled transition system consisting of states with directed
/// labelled edges.
pub struct LabelledTransitionSystem {
    pub states: Vec<State>,

    pub labels: Vec<String>,

    pub initial_state: StateIndex,

    pub num_of_transitions: usize,
}

impl LabelledTransitionSystem {
    /// Returns a borrow of the initial state.
    pub fn initial_state(&self) -> &State {
        &self.states[self.initial_state]
    }

    /// Returns the set of outgoing transitions for the given state.
    pub fn outgoing_transitions<'a>(&'a self, state: &'a State) -> impl Iterator + 'a {
        state
            .outgoing
            .iter()
            .map(|(label_index, out_index)| (&self.labels[*label_index], &self.states[*out_index]))
    }
}

/// A single state in the LTS, containing a vector of outgoing edges.
#[derive(Default, Debug, Clone)]
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
        for (from, state) in self.states.iter().enumerate() {
            for (label, to) in &state.outgoing {
                let label_name = &self.labels[*label];

                writeln!(f, "{from} --[{label_name}]-> {to}")?;
            }
        }

        Ok(())
    }
}