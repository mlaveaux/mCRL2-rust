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
        state.outgoing.iter().map(|(label_index, out_index)| {
            (&self.labels[*label_index], &self.states[*out_index])
        })
    }
}

/// A single state in the LTS, containing a vector of outgoing edges.
#[derive(Default, Debug, Clone)]
pub struct State {
    pub outgoing: Vec<(LabelIndex, StateIndex)>,
}

/// An enum used to indicate an edge or a self loop.
pub enum Edge<T> {
    Regular(T, T),

    /// For a self loop we only provide a mutable reference to the single state.
    Selfloop(T),
}

/// Index two locations (from, to) of an edge, returns mutable references to it.
pub fn index_edge<T>(slice: &mut [T], a: usize, b: usize) -> Edge<&mut T> {
    if a == b {
        assert!(a <= slice.len());
        Edge::Selfloop(slice.get_mut(a).unwrap())
    } else {
        assert!(a <= slice.len() && b < slice.len());

        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(slice.get_unchecked_mut(a) as *mut _);
            let br = &mut *(slice.get_unchecked_mut(b) as *mut _);
            Edge::Regular(ar, br)
        }
    }
}

impl fmt::Display for LabelledTransitionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {        
        // Print some information about the LTS.
        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Number of action labels: {}", self.labels.len())?;
        writeln!(f, "Number of transitions: {}", self.num_of_transitions)
    }
}

#[cfg(test)]
mod tests {
    use crate::io_aut::read_aut;

    #[test]
    fn test_traversal_lts() {
        env_logger::init();

        let file = include_str!("../../../examples/lts/abp.aut");

        let lts = read_aut(file.as_bytes()).unwrap();

        // Check the number of outgoing transitions of the initial state
        assert_eq!(lts.outgoing_transitions(lts.initial_state()).count(), 2);
    }
}
