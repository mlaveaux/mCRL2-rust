//!
//! This crate defines general utility functions.
//!
//! This crate does not use unsafe code.

#![forbid(unsafe_code)]

pub mod fast_counter;
pub mod global_guard;
pub mod helper;
pub mod protection_set;
pub mod thread_id;

pub use fast_counter::*;
pub use global_guard::*;
pub use helper::*;
pub use protection_set::*;
pub use thread_id::*;
