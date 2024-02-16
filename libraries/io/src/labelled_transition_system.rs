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

#[derive(Default, Debug, Clone)]
pub struct State {
    pub outgoing: Vec<(LabelIndex, StateIndex)>,
}

pub enum Pair<T> {
    Both(T, T),
    One(T),
}

pub fn index_twice<T>(slc: &mut [T], a: usize, b: usize) -> Pair<&mut T> {
    if a == b {
        Pair::One(slc.get_mut(a).unwrap())
    } else {
        assert!(a <= slc.len() && b < slc.len());

        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(slc.get_unchecked_mut(a) as *mut _);
            let br = &mut *(slc.get_unchecked_mut(b) as *mut _);
            Pair::Both(ar, br)
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