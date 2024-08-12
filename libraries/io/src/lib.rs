//!
//! A crate containing IO related functionality. This includes the reading of
//! .aut (Aldebaran) lts formats, and reading encoded integers.
//!
//! This crate does not use unsafe code.

#![forbid(unsafe_code)]

mod progress;
mod line_iterator;

pub mod io_aut;
pub mod u64_variablelength;