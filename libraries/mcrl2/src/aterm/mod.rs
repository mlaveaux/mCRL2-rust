//! Rust interface of Atermpp
//! 
//! This modules provides a safe abstraction for the C++ implementation of the
//! atermpp library. For performance we have replicated the protection set
//! mechanism on the Rust side, which is used during garbage collection to mark
//! terms as being reachable.
//! 
//! Terms are first-order terms f(t0, ..., tn) where f is a function symbol of arity
//! n+1 and t0 to tn are terms. They are stored immutably and maximally shared in the
//! main memory using a hash table. These are periodically garbage collected to remove
//! unused terms.
//! 
//! Instead of `unprotected_aterm` there are [ATermRef] classes whose lifetime
//! is bound by an existing term, providing a safe abstracting for terms that
//! are implicitly protected by for example occuring as subterm of another
//! protected term. They can be upgraded to a protected term using "protect" and
//! borrowed using "borrow".
//! 
//! There are [ATerm] objects that protected a term using the thread local protection
//! set mechnanism, i.e. they are not [Send], and [ATermGlobal] which are protected
//! using a global protection scheme. Furthermore, there

pub mod aterm_builder;
pub mod aterm_container;
pub mod global_aterm_pool;
pub mod aterm_pool;
pub mod term;
pub mod busy_forbidden;
pub mod symbol;

pub use term::*;
pub use aterm_builder::*;
pub use aterm_container::*;
pub use aterm_pool::*;
pub use busy_forbidden::*;
pub use symbol::*;