use std::{io::{BufReader, Read, BufRead}, error::Error, collections::HashMap};

use streaming_iterator::StreamingIterator;
use thiserror::Error;
use regex::Regex;

type Label = usize;

#[derive(Default, Debug, Clone)]
pub struct State {
    outgoing: Vec<(Label, usize)>,
}

pub struct LTS {
    initial_state: usize,
    states: Vec<State>,
}

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Invalid .aut header {0}")]
    InvalidHeader(&'static str),

    #[error("Invalid transition line")]
    InvalidTransition(),
}

struct LineIterator<T: Read> {
    reader: BufReader<T>,
    buffer: String,
    end: bool,
}

impl<T: Read> LineIterator<T> {
    pub fn new(reader: BufReader<T>) -> LineIterator<T> {
        LineIterator {
            reader,
            buffer: String::new(),
            end: false,
        }
    }

}

impl<T: Read> StreamingIterator for LineIterator<T> {
    type Item = String;

    fn advance(&mut self) {
        match self.reader.read_line(&mut self.buffer) {
            Ok(n) if n > 0 => {
                if self.buffer.ends_with('\n') {
                    self.buffer.pop();
                    if self.buffer.ends_with('\r') {
                        self.buffer.pop();
                    }
                }
            },
            Ok(_) => self.end = true,
            Err(_) => self.end = true,
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.end {
            None
        } else {
            Some(&self.buffer)
        }
    }
}

pub fn read_aut(reader: impl Read) -> Result<LTS, Box<dyn Error>> {

    let mut lines = LineIterator::new(BufReader::new(reader));
    lines.advance();
    let header = lines.get().unwrap(); //.ok_or(IOError::InvalidHeader("The first line should be the header"))??;

    let header_regex = Regex::new(r#"des\s*\(\s*([0-9]*)\s*,\s*([0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#).unwrap();
    let transition_regex = Regex::new(r#"\s*\(\s*([0-9]*)\s*,\s*(["a-zA-Z0-9]*)\s*,\s*([0-9]*)\s*\)\s*"#).unwrap();

    let (_, [grp1, grp2, grp3]) = header_regex.captures(header).ok_or(IOError::InvalidHeader("does not match des (<init>, <num_states>, <num_transitions>)"))?
        .extract();

    let initial_state: usize = grp1.parse()?;
    let num_of_states: usize = grp2.parse()?;
    let _num_of_transitions: usize = grp3.parse()?;

    let labels: HashMap<String, usize> = HashMap::new();
    let mut states : Vec<State> = Vec::with_capacity(num_of_states);

    while let Some(line) = lines.next() {

        let (_, [grp1, grp2, grp3]) = transition_regex.captures(line).ok_or(
            IOError::InvalidTransition()
        )?.extract();

        // Parse the from and to states, with the given label.
        let from: usize = grp1.parse()?;
        let to: usize = grp3.parse()?;
        let label = **labels.get(grp2).get_or_insert(&labels.len());

        // Insert state when it does not exist, and then add the transition.
        if from >= states.len() {
            states.resize_with(from+1, Default::default);
        }

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
        let file = include_str!("../../../examples/lts/abp.aut");

        let _lts = read_aut(file.as_bytes()).unwrap();
    }

}