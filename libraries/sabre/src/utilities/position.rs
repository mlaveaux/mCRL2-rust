//! Module for storing positions of terms
use core::fmt;
use mcrl2_rust::atermpp::ATerm;
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;

/// An ExplicitPosition stores a list of position indices. The index starts at 1.
/// The subterm of term s(s(0)) at position 1.1 is 0.
/// The empty position, aka the root term, is represented by the symbol ε.
/// Indices are stored in a SmallVec, which is configured to store 4 elements.
/// If the position contains a maximum of 4 elements it is stored on the stack.
/// If the position is longer a heap allocation is made.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExplicitPosition {
    pub indices: SmallVec<[usize; 4]>,
}

impl ExplicitPosition {
    pub fn new(indices: &[usize]) -> ExplicitPosition {
        ExplicitPosition {
            indices: SmallVec::from(indices),
        }
    }

    pub fn empty_pos() -> ExplicitPosition {
        ExplicitPosition {
            indices: smallvec![],
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.indices.len()
    }
}

impl ExplicitPosition {
    /// Converts a position to a string for pretty printing
    pub fn to_string(&self) -> String {
        if self.indices.is_empty() {
            "ε".to_string()
        } else {
            let mut s = "".to_string();
            let mut first = true;
            for p in &self.indices {
                if first {
                    s = s + &*p.to_string();
                    first = false;
                } else {
                    s = s + "." + &*p.to_string();
                }
            }
            s
        }
    }
}

fn get_position_rec<'a>(term: &'a ATerm, queue: &[usize]) -> ATerm {
    if queue.is_empty() {
        term.clone()
    } else {
        let arg = term.arg(queue[0] - 1); // Note that positions are 1 indexed.
        get_position_rec(&arg, &queue[1..]).clone()
    }
}

/// Returns the subterm at the specific position
pub fn get_position<'a>(term: &'a ATerm, position: &ExplicitPosition) -> ATerm {
    get_position_rec(term, &position.indices)
}

impl fmt::Display for ExplicitPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for ExplicitPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// An iterator over all (term, position) pairs of the given [ATerm].
pub struct PositionIterator {
    queue: VecDeque<(ATerm, ExplicitPosition)>,
}

impl PositionIterator {
    pub fn new(t: ATerm) -> PositionIterator {
        PositionIterator {
            queue: VecDeque::from([(t, ExplicitPosition::empty_pos())]),
        }
    }
}

impl Iterator for PositionIterator {
    type Item = (ATerm, ExplicitPosition);

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            None
        } else {
            // Get a subterm to inspect
            let (term, pos) = self.queue.pop_front().unwrap();

            // Put subterms in the queue
            for (i, argument) in term.arguments().iter().enumerate() {
                let mut new_position = pos.clone();
                new_position.indices.push(i + 1);
                self.queue.push_back((argument.clone(), new_position));
            }

            Some((term, pos))
        }
    }
}

#[cfg(test)]
mod tests {
    use mcrl2_rust::atermpp::TermPool;

    use super::*;

    #[test]
    fn test_get_position() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();
        let result = tp.from_string("a").unwrap();

        assert_eq!(get_position(&t, &ExplicitPosition::new(&[1, 1])), result);
    }

    #[test]
    fn test_position_iterator() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();

        for (term, pos) in PositionIterator::new(t.clone()) {
            assert_eq!(get_position(&t, &pos), term, "The resulting (subterm, position) pair doesn't match the get_position implementation");
        }

        assert_eq!(
            PositionIterator::new(t.clone()).count(),
            4,
            "The number of subterms doesn't match the expected value"
        );
    }
}
