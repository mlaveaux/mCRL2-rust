use cxx::UniquePtr;

#[cxx::bridge(namespace = "atermpp")]
mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/atermpp/aterm.h");

    type aterm;

    fn new_aterm() -> UniquePtr<aterm>;
  } 
}

/// Rust representation of a atermpp::aterm
struct ATerm
{
  term: UniquePtr<ffi::aterm>
}

impl ATerm
{
  pub fn new() -> ATerm
  {
    ATerm { term: ffi::new_aterm() }
  }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_make_aterm()
    {
      let term = ATerm::new();
    }
}