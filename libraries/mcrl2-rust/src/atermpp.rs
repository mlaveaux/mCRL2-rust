use cxx::UniquePtr;

#[cxx::bridge(namespace = "atermpp")]
mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/cpp/atermpp/aterm.h");

    type aterm;

    fn new_aterm() -> UniquePtr<aterm>;
  } 
}

/// Rust representation of a atermpp::aterm
struct ATerm
{
  term: UniquePtr<ffi::aterm>
}

/// A function symbol is simply a number
type Symbol = usize;

impl ATerm
{
  pub fn new() -> ATerm
  {
    ATerm { term: ffi::new_aterm() }
  }

  pub fn get_head_symbol() -> Symbol
  {
    0
  }
}