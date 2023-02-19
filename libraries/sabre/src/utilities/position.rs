//! Module for storing positions in terms
use smallvec::{SmallVec,smallvec};
use core::fmt;

type PositionIndex = usize;

/// An ExplicitPosition stores a list of position indices. The index starts at 1.
/// The subterm of term s(s(0)) at position 1.1 is 0.
/// The empty position, aka the root term, is represented by the symbol ε.
/// Indices are stored in a SmallVec, which is configured to store 4 elements.
/// If the position contains a maximum of 4 elements it is stored on the stack.
/// If the position is longer a heap allocation is made.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExplicitPosition 
{
    pub indices: SmallVec<[PositionIndex;4]>
}

impl ExplicitPosition 
{
    pub fn empty_pos() -> ExplicitPosition 
    {
        ExplicitPosition { indices: smallvec![] }
    }

    #[inline(always)]
    pub fn len(&self) -> usize 
    {
        self.indices.len()
    }
}

impl ExplicitPosition 
{
    /// Converts a position to a string for pretty printing
    pub fn to_string(&self) -> String 
    {
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

impl fmt::Display for ExplicitPosition 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for ExplicitPosition 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
