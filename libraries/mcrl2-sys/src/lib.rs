//!
//! This crate demonstrates a Rust interface to interact with the
//! [mCRL2](https://mcrl2.org/) toolset. Other crates demonstrate various
//! prototypes making use of this interface.
//! 
//! Every module mirrors the corresponding library of the mCRL2 toolset. Within
//! it a foreign function interface (FFI) is defined using the [cxx] crate. The
//! goal of this interface is to define proper Rust types that behave as
//! expected while making use of the existing code base.

mod atermpp_ffi;
mod data_ffi;
mod lps_ffi;

pub mod atermpp;
pub mod lps;
pub mod data;