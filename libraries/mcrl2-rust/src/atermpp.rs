use std::fmt;
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