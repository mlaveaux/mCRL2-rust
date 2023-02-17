///
/// The FFI between the mCRL2 aterm library.
/// 

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
  term: ffi::aterm
}

impl Aterm
{
  pub fn new() -> Aterm
  {
    Aterm { term: new_aterm() }
  }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_make_aterm()
    {
      let term = atermpp::Aterm::new();
    }
}