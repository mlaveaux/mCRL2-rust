use std::{io::Read, error::Error, collections::HashMap};

use log::trace;
use streaming_iterator::StreamingIterator;
use thiserror::Error;
use regex::Regex;

use crate::line_iterator::LineIterator;

/// The index type for a label.
type Label = usize;

#[derive(Default, Debug, Clone)]
pub struct State {
    pub outgoing: Vec<(Label, usize)>,
}

pub struct LTS {    
    pub initial_state: usize,

    pub states: Vec<State>,
}

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Invalid .aut header {0}")]
    InvalidHeader(&'static str),

    #[error("Invalid transition line")]
    InvalidTransition(),
}

pub fn read_aut(reader: impl Read) -> Result<LTS, Box<dyn Error>> {

    let mut lines = LineIterator::new(reader);
    lines.advance();
    let header = lines.get().unwrap(); //.ok_or(IOError::InvalidHeader("The first line should be the header"))??;

    // Regex for des (<initial>: Nat, <num_of_states>: Nat, <num_of_transitions>: Nat)
    let header_regex = Regex::new(r#"des\s*\(\s*([0-9]*)\s*,\s*([0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#).unwrap();
    
    // Regex for (<from>: Nat, <label>: str, <to>: Nat)
    let transition_regex = Regex::new(r#"\s*\(\s*([0-9]*)\s*,\s*"([,\ \(\)a-zA-Z0-9]*)"\s*,\s*([0-9]*)\s*\)\s*"#).unwrap();

    let (_, [initial_txt, num_of_transitions_txt, num_of_states_txt]) = header_regex.captures(header).ok_or(IOError::InvalidHeader("does not match des (<init>, <num_states>, <num_transitions>)"))?
        .extract();

    let initial_state: usize = initial_txt.parse()?;
    let _num_of_transitions: usize = num_of_transitions_txt.parse()?;
    let num_of_states: usize = num_of_states_txt.parse()?;

    let labels: HashMap<String, usize> = HashMap::new();
    let mut states : Vec<State> = Vec::with_capacity(num_of_states);

    while let Some(line) = lines.next() {
        trace!("{}", line);

        let (_, [from_txt, label_txt, to_txt]) = transition_regex.captures(line).ok_or(
            IOError::InvalidTransition()
        )?.extract();

        // Parse the from and to states, with the given label.
        let from: usize = from_txt.parse()?;
        let to: usize = to_txt.parse()?;
        let label = **labels.get(label_txt).get_or_insert(&labels.len());

        // Insert state when it does not exist, and then add the transition.
        if from >= states.len() {
            states.resize_with(from + 1, Default::default);
        }

        if to >= states.len() {
            states.resize_with(to + 1, Default::default);
        }

        trace!("Read transition {} --[{}]-> {}", from, label_txt, to);

        states[from].outgoing.push((label, to));
    }

    // Compute back references.

    Ok(LTS {
        initial_state,
        states,
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