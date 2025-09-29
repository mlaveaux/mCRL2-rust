use mcrl2rust_lts::LabelledTransitionSystem;
use mcrl2rust_lts::CompactTransition;

/// A struct containg information related to the incoming transitions for every
/// state.
pub struct IncomingTransitions {
    incoming_transitions: Vec<CompactTransition>,
    state2incoming: Vec<TransitionIndex>,
}

// TODO: Bytepack end and silent into one u64.
#[derive(Default, Clone)]

struct TransitionIndex {
    start: usize,
    end: usize
}

impl IncomingTransitions {
    pub fn new(lts: &LabelledTransitionSystem) -> IncomingTransitions {
        let mut incoming_transitions: Vec<CompactTransition> = vec![CompactTransition::default(); lts.num_of_transitions()];
        let mut state2incoming: Vec<TransitionIndex> = vec![TransitionIndex::default(); lts.num_of_states()];
        
        // Compute the number of incoming (silent) transitions for each state.
        for state_index in lts.iter_states() {
            for trans in lts.outgoing_transitions_compact(state_index) {
                state2incoming[trans.state()].end += 1;
            }
        }

        // Fold the counts in state2incoming. Temporarily mixing up the data
        // structure such that after placing the transitions below the counts
        // will be correct.
        state2incoming.iter_mut().fold(0, |count, index| {
            let end = count + index.end;
            index.start = end;
            index.end = end;
            end
        });

        for state_index in lts.iter_states() {
        for transition in lts.outgoing_transitions_compact(state_index) {
                let index = &mut state2incoming[transition.state()];
                index.start -= 1;
                incoming_transitions[index.start] = CompactTransition::new(transition.label(), state_index);
            }
        }
        for state_index in lts.iter_states() {
            // Sort the incoming transitions such that silent transitions come first.
            let slice = &mut incoming_transitions[state2incoming[state_index].start .. state2incoming[state_index].end];
            slice.sort_unstable();
        }
        IncomingTransitions { incoming_transitions, state2incoming }
    }

    /// Returns an iterator over the incoming transitions for the given state.
    pub fn incoming_transitions(&self, state_index: usize) -> impl Iterator<Item = &CompactTransition> {
        self.incoming_transitions[self.state2incoming[state_index].start .. self.state2incoming[state_index].end].iter()
    }

    // Return an iterator over the incoming silent transitions for the given state.
    pub fn incoming_silent_transitions(&self, state_index: usize) -> impl Iterator<Item = &CompactTransition>  {
        self.incoming_transitions[self.state2incoming[state_index].start .. self.state2incoming[state_index].end].iter().take_while(|t| t.label() == 0)
    }
}

#[cfg(test)]
mod tests {

    use mcrl2rust_lts::random_lts;

    use super::*;

    #[test]
    fn test_incoming_transitions() {
        let lts = random_lts(10, 3, 3);
        let _ = IncomingTransitions::new(&lts);
    }
}
