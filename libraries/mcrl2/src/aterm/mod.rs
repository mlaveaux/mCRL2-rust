//! Rust interface of Atermpp
//!
//! This modules provides a safe abstraction for the C++ implementation of the
//! atermpp library. For performance we have replicated the protection set
//! mechanism on the Rust side, which is used during garbage collection to mark
//! terms as being reachable.
//!
//! Instead of `unprotected_aterm` there are [ATermRef] classes whose lifetime
//! is bound by an existing term, providing a safe abstracting for terms that
//! are implicitly protected by for example occuring as subterm of another
//! protected term. They can be upgraded to a protected term using "protect" and
//! borrowed using "borrow".

pub mod aterm_builder;
pub mod aterm_container;
pub mod aterm_pool;
pub mod busy_forbidden;
pub mod global_aterm_pool;
pub mod symbol;
pub mod term;

pub use aterm_builder::*;
pub use aterm_container::*;
pub use aterm_pool::*;
pub use busy_forbidden::*;
pub use symbol::*;
pub use term::*;
