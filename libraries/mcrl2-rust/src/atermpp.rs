use std::{fmt, hash::Hash, hash::Hasher, cmp::Ordering};
use cxx::UniquePtr;

#[cxx::bridge(namespace = "atermpp")]
pub mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/cpp/atermpp/aterm.h");

    type aterm;

    /// Creates a default term.
    fn new_aterm() -> UniquePtr<aterm>;

    /// Converts an aterm to a string.
    fn print_aterm(term: &aterm) -> String;

    /// Computes the hash for an aterm.
    fn hash_aterm(term: &aterm) -> usize;

    /// Returns true iff the terms are equivalent.
    fn equal_aterm(first: &aterm, second: &aterm) -> bool;

    /// Returns true iff the first term is less than the second term.
    fn less_aterm(first: &aterm, second: &aterm) -> bool;

    /// Makes a copy of the given term.
    fn copy_aterm(term: &aterm) -> UniquePtr<aterm>;
  } 
}

/// Rust representation of a atermpp::aterm
pub struct ATerm
{
  term: UniquePtr<ffi::aterm>
}

/// This is a standin for the global term pool, with the idea to eventually replace it by a proper implementation.
pub struct TermPool
{

}

/// A function symbol is simply a number
type Symbol = usize;

impl ATerm
{
  pub fn new() -> Self
  {
    ATerm { term: ffi::new_aterm() }
  }

  pub fn from(term: UniquePtr<ffi::aterm>) -> Self
  {
    ATerm { term }
  }

  /// Get access to the underlying term
  pub fn get(&self) -> &ffi::aterm
  {
    &self.term
  }

  pub fn get_head_symbol() -> Symbol
  {
    0
  }
}


impl fmt::Display for ATerm 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", ffi::print_aterm(&self.term))
  }
}

impl fmt::Debug for ATerm 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", ffi::print_aterm(&self.term))
  }
}

impl Hash for ATerm
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_usize(ffi::hash_aterm(&self.term));
  }
}

impl PartialEq for ATerm
{
  fn eq(&self, other: &Self) -> bool
  {
    ffi::equal_aterm(&self.term, &other.term)
  }
}

impl PartialOrd for ATerm
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering>
  {
    if ffi::less_aterm(&self.term, &other.term) {
      Some(Ordering::Less)
    } else if ffi::equal_aterm(&self.term, &other.term) {
      Some(Ordering::Equal)
    }
    else {
      Some(Ordering::Greater)
    }
  }
}

impl Ord for ATerm 
{
  fn cmp(&self, other: &Self) -> Ordering
  {
    if ffi::less_aterm(&self.term, &other.term) {
      Ordering::Less
    } else if ffi::equal_aterm(&self.term, &other.term) {
      Ordering::Equal
    }
    else {
      Ordering::Greater
    }
  }

}

impl Clone for ATerm
{
  fn clone(&self) -> Self 
  {
    ATerm {
      term: ffi::copy_aterm(&self.term),
    }      
  }
}

impl Eq for ATerm {}