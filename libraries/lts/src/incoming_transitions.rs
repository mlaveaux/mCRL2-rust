use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

/// A struct containg information related to the incoming transitions for every
/// state.
pub struct IncomingTransitions {
    incoming_transitions: Vec<Vec<(LabelIndex, StateIndex)>>,
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions: Vec<Vec<(LabelIndex, StateIndex)>> = vec![Vec::default(); lts.num_of_states()];

        // Compute incoming transitions for all states.
        for (state_index, state) in lts.iter_states() {
            for (label_index, to) in &state.outgoing {
                incoming_transitions[*to].push((*label_index, state_index));
            }
        }

        IncomingTransitions { incoming_transitions }
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        self.incoming_transitions[state_index].iter()
    }
}

#[cfg(test)]
mod tests {

    use crate::random_lts;

    use super::*;

    #[test]
    fn test_incoming_transitions() {
        let lts = random_lts(10, 3, 3);
        let _ = IncomingTransitions::new(&lts);
    }
}
