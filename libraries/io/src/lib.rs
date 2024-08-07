//!
//! A crate containing IO related functionality. This includes the reading of
//! .aut (Aldebaran) lts formats, and reading encoded integers.
//!

mod line_iterator;

pub mod io_aut;
pub mod u64_variablelength;