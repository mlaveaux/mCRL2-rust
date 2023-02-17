///
/// The FFI between the mCRL2 aterm library.
/// 

#[cxx::bridge(namespace = "atermpp")]
mod atermpp {

  unsafe extern "C++" {    
    include!("mcrl2-rust/atermpp/aterm.h");

    type aterm;

    fn new_aterm() -> UniquePtr<aterm>;
  } 
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_make_aterm()
    {
      let term = atermpp::new_aterm();
    }
}