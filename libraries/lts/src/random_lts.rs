use rand::Rng;

use crate::LabelledTransitionSystem;
use crate::State;

pub fn random_lts(num_of_states: usize, num_of_labels: u32) -> LabelledTransitionSystem {

    // Introduce num_of_states states.
    let mut states: Vec<State> = vec![State::default(); num_of_states];

    // Introduce lower case letters for the labels.
    let mut labels: Vec<String> = Vec::new();
    for i in 0..num_of_labels {
        labels.push(char::from_digit(i+10, 36).unwrap().to_string());
    }

    let mut num_of_transitions = 0;

    let mut rng = rand::thread_rng();

    for state in &mut states {

        let label = rng.gen_range(0..num_of_labels);
        let to = rng.gen_range(0..num_of_states);

        state.outgoing.push((label as usize, to));
        num_of_transitions += 1;
    }

    LabelledTransitionSystem {
        initial_state: 0,
        states,
        labels,
        num_of_transitions,
    }
}