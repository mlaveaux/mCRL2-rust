//! This module contains the code to construct a set automaton.
//!
//! The code is documented with the assumption that the reader knows how set automata work.
//! See https://arxiv.org/abs/2202.08687 for a paper on the construction of set automata.

mod automaton;
mod announcement;
mod display;

pub use automaton::*;
pub use announcement::*;
pub use display::*;