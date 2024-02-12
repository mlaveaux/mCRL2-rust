use std::io::{BufReader, Read, BufRead};
use streaming_iterator::StreamingIterator;

/// A lending iterator over the lines of a type implementing Read.
pub struct LineIterator<T: Read> {
    reader: BufReader<T>,
    buffer: String,
    end: bool,
}

impl<T: Read> LineIterator<T> {
    pub fn new(reader: T) -> LineIterator<T> {
        LineIterator {
            reader: BufReader::new(reader),
            buffer: String::new(),
            end: false,
        }
    }

}

impl<T: Read> StreamingIterator for LineIterator<T> {
    type Item = String;

    fn advance(&mut self) {
        
        self.buffer.clear();
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
