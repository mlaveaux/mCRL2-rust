//!
//! This crate defines general utility functions.
//!
//! This crate does not use unsafe code.
 
#![forbid(unsafe_code)]

pub mod protection_set;
pub mod global_guard;

pub use protection_set::*;
pub use global_guard::*;