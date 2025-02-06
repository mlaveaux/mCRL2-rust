//!
//! A crate containing labelled transition systems related functionality.
//!
//! This crate does not use unsafe code.

#![forbid(unsafe_code)]

mod labelled_transition_system;
mod random_lts;

pub use labelled_transition_system::*;
pub use random_lts::*;
