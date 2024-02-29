use std::{collections::HashMap, error::Error, io::Read};

use log::trace;
use regex::Regex;
use streaming_iterator::StreamingIterator;
use thiserror::Error;

use crate::{
    labelled_transition_system::{LabelIndex, LabelledTransitionSystem, State},
    line_iterator::LineIterator,
};

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Invalid .aut header {0}")]
    InvalidHeader(&'static str),

    #[error("Invalid transition line")]
    InvalidTransition(),
}

/// Loads a labelled transition system in the Aldebaran format from the given reader.
///
/// The Aldebaran format consists of a header:
///     `des (<initial>: Nat, <num_of_states>: Nat, <num_of_transitions>: Nat)`
///     
/// And one line for every transition:
///     `(<from>: Nat, "<label>": Str, <to>: Nat)`
///     `(<from>: Nat, <label>: Str, <to>: Nat)`
pub fn read_aut(reader: impl Read) -> Result<LabelledTransitionSystem, Box<dyn Error>> {
    let mut lines = LineIterator::new(reader);
    lines.advance();
    let header = lines.get().ok_or(IOError::InvalidHeader(
        "The first line should be the header",
    ))?;

    // Regex for des (<initial>: Nat, <num_of_states>: Nat, <num_of_transitions>: Nat)
    let header_regex = Regex::new(r#"des\s*\(\s*([0-9]*)\s*,\s*([0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#)
        .expect("Regex compilation should not fail");

    // Regex for (<from>: Nat, "<label>": str, <to>: Nat)
    let transition_regex = Regex::new(r#"\s*\(\s*([0-9]*)\s*,\s*"(.*)"\s*,\s*([0-9]*)\s*\)\s*"#)
        .expect("Regex compilation should not fail");

    // Regex for (<from>: Nat, label: str, <to>: Nat), used in the VLTS benchmarks
    let unquoted_transition_regex =
        Regex::new(r#"\s*\(\s*([0-9]*)\s*,\s*(.*)\s*,\s*([0-9]*)\s*\)\s*"#)
            .expect("Regex compilation should not fail");

    let (_, [initial_txt, num_of_transitions_txt, num_of_states_txt]) = header_regex
        .captures(header)
        .ok_or(IOError::InvalidHeader(
            "does not match des (<init>, <num_states>, <num_transitions>)",
        ))?
        .extract();

    let initial_state: usize = initial_txt.parse()?;
    let num_of_transitions: usize = num_of_transitions_txt.parse()?;
    let num_of_states: usize = num_of_states_txt.parse()?;

    let labels_index: HashMap<String, LabelIndex> = HashMap::new();
    let mut labels: Vec<String> = Vec::new();

    let mut states: Vec<State> = Vec::with_capacity(num_of_states);

    while let Some(line) = lines.next() {
        trace!("{}", line);

        // Try either of the transition regexes and otherwise return an error.
        let (_, [from_txt, label_txt, to_txt]) = transition_regex
            .captures(line)
            .or(unquoted_transition_regex.captures(line))
            .ok_or(IOError::InvalidTransition())?
            .extract();

        // Parse the from and to states, with the given label.
        let from: usize = from_txt.parse()?;
        let to: usize = to_txt.parse()?;
        let label_index = **labels_index
            .get(label_txt)
            .get_or_insert(&labels_index.len());

        // Insert state when it does not exist, and then add the transition.
        if from >= states.len() {
            states.resize_with(from + 1, Default::default);
        }

        if to >= states.len() {
            states.resize_with(to + 1, Default::default);
        }

        if label_index >= labels.len() {
            labels.resize_with(label_index + 1, Default::default);
        }

        trace!("Read transition {} --[{}]-> {}", from, label_txt, to);

        states[from].outgoing.push((label_index, to));

        if labels[label_index].is_empty() {
            labels[label_index] = label_txt.to_string();
        }
    }

    Ok(LabelledTransitionSystem {
        initial_state,
        states,
        labels,
        num_of_transitions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_aut() {
        env_logger::init();

        let file = include_str!("../../../examples/lts/abp.aut");

        let _lts = read_aut(file.as_bytes()).unwrap();
    }
}
