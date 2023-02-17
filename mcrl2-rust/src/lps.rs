use std::fmt;
use cxx::UniquePtr;

#[cxx::bridge(namespace = "mcrl2::lps")]
mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/lps/lps.h");

    type specification;

    /// Reads a .lps file and returns the resulting linear process specification.
    fn read_linear_process_specification(filename: &str) -> Result<UniquePtr<specification>>;

    /// Converts a linear process specification to a string.
    fn print_linear_process_specification(spec: &specification) -> String;
  }

}

/// Rust representation of a lps::linear_process_specification.
pub struct LinearProcessSpecification
{
  a: UniquePtr<ffi::specification>
}

impl LinearProcessSpecification
{
  pub fn read(filename: &str) -> LinearProcessSpecification
  {
    return LinearProcessSpecification {
      a: ffi::read_linear_process_specification(filename).expect("cannot read given lps.")
    }
  }
}

impl fmt::Display for LinearProcessSpecification {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{}", ffi::print_linear_process_specification(&self.a))
  }
}
