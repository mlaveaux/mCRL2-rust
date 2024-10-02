use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

use log::debug;
use log::trace;
use regex::Regex;
use streaming_iterator::StreamingIterator;
use thiserror::Error;

use crate::line_iterator::LineIterator;
use crate::progress::Progress;
use lts::LabelIndex;
use lts::LabelledTransitionSystem;
use lts::State;

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
///     `des (<initial>: Nat, <num_of_transitions>: Nat, <num_of_states>: Nat)`
///     
/// And one line for every transition:
///     `(<from>: Nat, "<label>": Str, <to>: Nat)`
///     `(<from>: Nat, <label>: Str, <to>: Nat)`
pub fn read_aut(reader: impl Read, mut hidden_labels: Vec<String>) -> Result<LabelledTransitionSystem, Box<dyn Error>> {
    let start = Instant::now();
    debug!("Reading LTS in .aut format...");

    let mut lines = LineIterator::new(reader);
    lines.advance();
    let header = lines.get().ok_or(IOError::InvalidHeader(
        "The first line should be the header",
    ))?;

    // Regex for des (<initial>: Nat, <num_of_states>: Nat, <num_of_transitions>: Nat)
    let header_regex = Regex::new(r#"des\s*\(\s*([0-9]*)\s*,\s*([0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#)
        .expect("Regex compilation should not fail");

    // Regex for (<from>: Nat, "<label>": str, <to>: Nat) or (<from>: Nat,
    // label: str, <to>: Nat). Note that the quotes are optional, and even one
    // side can be forgotten.
    let transition_regex = Regex::new(r#"\s*\(\s*([0-9]*)\s*,\s*"?(.*)"?\s*,\s*([0-9]*)\s*\)\s*"#)
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

    // This is used to keep track of the label to index mapping.
    let mut labels_index: HashMap<String, LabelIndex> = HashMap::new();
    let mut labels: Vec<String> = Vec::new();

    let mut states: Vec<State> = Vec::with_capacity(num_of_states);
    let mut progress = Progress::new(
        |value, increment| debug!("Reading transitions {}%...", value / increment),
        num_of_transitions,
    );

    // Only used to avoid allocations when reading the transitions.
    let mut capture_locations = transition_regex.capture_locations();

    while let Some(line) = lines.next() {
        trace!("{}", line);

        // Try either of the transition regexes and otherwise return an error.transition_regex.
        // This is essentially a low level version of captures().extract() that does not allocate.
        transition_regex.captures_read(&mut capture_locations, line)
            .ok_or(IOError::InvalidTransition())?;

        let (from_txt, label_txt, to_txt) = (
            &line[capture_locations.get(1).unwrap().0..capture_locations.get(1).unwrap().1],
            &line[capture_locations.get(2).unwrap().0..capture_locations.get(2).unwrap().1],
            &line[capture_locations.get(3).unwrap().0..capture_locations.get(3).unwrap().1],
        );

        // Parse the from and to states, with the given label.
        let from: usize = from_txt.parse()?;
        let to: usize = to_txt.parse()?;

        let label_index = *labels_index
            .entry(label_txt.to_string())
            .or_insert(labels.len());

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

        progress.add(1);
    }

    // Remove duplicated outgoing transitions.
    let mut num_of_transitions = 0;
    for state in &mut states {
        state.outgoing.sort();
        state.outgoing.dedup();
        num_of_transitions += state.outgoing.len();
    }

    debug!("Finished reading LTS");

    hidden_labels.push("tau".to_string());
    debug!("Time read_aut: {:.3}s", start.elapsed().as_secs_f64());
    Ok(LabelledTransitionSystem::new(
        initial_state,
        states,
        labels,
        hidden_labels,
        num_of_transitions,
    ))
}

/// Write a labelled transition system in plain text in Aldebaran format to the given writer.
pub fn write_aut(
    writer: &mut impl Write,
    lts: &LabelledTransitionSystem,
) -> Result<(), Box<dyn Error>> {
    writeln!(
        writer,
        "des ({}, {}, {})",
        lts.initial_state_index(),
        lts.num_of_transitions(),
        lts.num_of_states()
    )?;

    for (state_index, state) in lts.iter_states() {
        for (label, to) in &state.outgoing {
            writeln!(writer, "({}, \"{}\", {})", state_index, if lts.is_hidden_label(*label) {
                "tau"
            } else {
                &lts.labels()[*label]
            }, to)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    #[test]
    fn test_reading_aut() {
        let file = include_str!("../../../examples/lts/abp.aut");

        let lts = read_aut(file.as_bytes(), vec![]).unwrap();

        assert_eq!(lts.initial_state_index(), 0);
        assert_eq!(lts.num_of_transitions(), 92);
        println!("{}", lts);
    }

    #[test]
    fn test_lts_failure() {
        let wrong_header = "
        des (0,2,                                     
            (0,\"r1(d1)\",1)
            (0,\"r1(d2)\",2)
        ";

        debug_assert!(read_aut(wrong_header.as_bytes(), vec![]).is_err());

        let wrong_transition = "
        des (0,2,3)                           
            (0,\"r1(d1),1)
            (0,\"r1(d2)\",2)
        ";

        debug_assert!(read_aut(wrong_transition.as_bytes(), vec![]).is_err());
    }

    #[test]
    fn test_traversal_lts() {
        let file = include_str!("../../../examples/lts/abp.aut");

        let lts = read_aut(file.as_bytes(), vec![]).unwrap();

        // Check the number of outgoing transitions of the initial state
        assert_eq!(lts.outgoing_transitions(lts.initial_state_index()).count(), 2);
    }

    #[test]
    fn test_writing_lts() {
        let file = include_str!("../../../examples/lts/abp.aut");
        let lts_original = read_aut(file.as_bytes(), vec![]).unwrap();

        // Check that it can be read after writing, and results in the same LTS.
        let mut buffer: Vec<u8> = Vec::new();
        write_aut(&mut buffer, &lts_original).unwrap();

        let lts = read_aut(&buffer[0..], vec![]).unwrap();

        assert_eq!(lts_original, lts, "The LTS after writing is different");
    }
}
