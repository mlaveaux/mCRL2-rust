//!
//! Safe abstraction for the data library, containing rewriters and data
//! specifications. In most cases we avoid storing unnecessary data in aterms
//! and instead use structs where appropriate.
//! 
//! Since we avoid implicit conversions and Rust has no inheritance the
//! structure of the data library is slightly different. In principle for every
//! type stored in a term, e.g. data::variable, data::application etc, we
//! provide a Rust type, in this case DataVariable and DataApplication that can
//! be constructed from an ATerm (value conversion). However, when a term only
//! has to be inspected there are also DataVariableRef and DataApplicationRef
//! that mimic the unprotected term structure.
//! 
//! Every DataVariableRef can be upgraded to DataVariable with "protect" and
//! borrowed with "borrow", there are also into conversions to and from ATerms
//! that perform runtime checking for correctness.
//! 

pub mod data_specification;
pub mod data_terms;
pub mod jitty;

pub use data_specification::*;
pub use data_terms::*;
pub use jitty::*;