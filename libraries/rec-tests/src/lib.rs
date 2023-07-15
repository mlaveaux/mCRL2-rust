//! This crate offers parsing for REC files and test cases for the rewrite engines.
//!
//! REC, short for Rewriting Engine Competition, is a format for specifying rewrite systems.
//! The parse_rec module contains functions for loading a REC file.
//!

mod parse_rec;
mod syntax;

pub use parse_rec::{from_string, load_REC_from_strings, load_REC_from_file};