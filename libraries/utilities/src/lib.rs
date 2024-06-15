//!
//! This crate defines general utility functions.
//!
//! This crate does not use unsafe code.

#![forbid(unsafe_code)]

pub mod global_guard;
pub mod helper;
pub mod protection_set;

pub use global_guard::*;
pub use helper::*;
pub use protection_set::*;
