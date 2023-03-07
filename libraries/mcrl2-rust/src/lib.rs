//!
//! Defines types that directly interact with the libraries of the (mCRL2)[https://mcrl2.org/] toolset.
//! 
//! Every module mirrors the corresponding library of the mCRL2 toolset. Within it a foreign function interface (FFI) is defined using the [cxx] crate.
//! The goal of this interface is to define proper Rust types that behave as expected while making use of the existing code base.

pub mod atermpp;
pub mod lps;
pub mod data;