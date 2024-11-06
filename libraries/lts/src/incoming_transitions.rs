use crate::LabelIndex;
use crate::LabelledTransitionSystem;
use crate::StateIndex;

/// A struct containg information related to the incoming transitions for every
/// state.
pub struct IncomingTransitions {
    incoming_transitions: Vec<(LabelIndex, StateIndex)>,
    state2incoming: Vec<(usize,usize,usize)>, // (start, end, silent)
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions: Vec<(LabelIndex, StateIndex)> = vec![(0,0); lts.num_of_transitions()];
        let mut state2incoming: Vec<(usize,usize,usize)> = vec![ (0,0,0); lts.num_of_states()];
        
        // Compute incoming transitions for all states.
        for (_, state) in lts.iter_states() {
            for (label_index, to) in &state.outgoing {
                state2incoming[*to].1 += 1;
                if lts.is_hidden_label(*label_index) {
                    state2incoming[*to].2 += 1;
                }
            }
        }
        // Fold the counts in state2incoming (temporay mixing up the data structure).
        let mut count = 0;
        for (start, end, silent) in state2incoming.iter_mut() {
            count += *end;
            *start = count-*silent;
            *end = count;
            *silent = 0;
        }

        for (state_index, state) in lts.iter_states() {
            for (label_index, to) in &state.outgoing {
                if lts.is_hidden_label(*label_index) {
                    //Place at end of incoming transitions.
                    state2incoming[*to].2 += 1;
                    incoming_transitions[state2incoming[*to].1 - state2incoming[*to].2] = (*label_index, state_index);
                } else {
                    state2incoming[*to].0 -= 1;
                    incoming_transitions[state2incoming[*to].0] = (*label_index, state_index);
                }
            }
        }
        // for (start, end , silent) in state2incoming.iter() {
        //     print!("{} {} {}\n", start, end, silent);
        // }
        IncomingTransitions { incoming_transitions, state2incoming}
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)> {
        // Return an iterator over the incoming transitions for the given state.
        let start = self.state2incoming[state_index].0;
        let end = self.state2incoming[state_index].1;
        self.incoming_transitions[start .. end].iter()
    }

    pub fn incoming_silent_transitions(&self, state_index: usize) -> impl Iterator<Item = &(LabelIndex, StateIndex)>  {
        self.incoming_transitions[self.state2incoming[state_index].1-self.state2incoming[state_index].2 .. self.state2incoming[state_index].1].iter()
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
