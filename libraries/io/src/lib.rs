//!
//! A crate containing IO related functionality.
//! 

mod line_iterator;
mod labelled_transition_system;

pub mod io_aut;
pub mod u64_variablelength;

pub use labelled_transition_system::*;