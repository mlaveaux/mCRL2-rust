use std::fmt;
use cxx::UniquePtr;

#[cxx::bridge(namespace = "mcrl2::data")]
mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/cpp/data/data.h");

    type data_specification;

    /// Reads a .lps file and returns the resulting linear process specification.
    fn parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;
  }

}

pub struct DataSpecification
{
  data_spec: UniquePtr<ffi::data_specification>,
}

impl DataSpecification
{
  /// Parses the given text into a data specification
  pub fn from(text: &str) -> DataSpecification
  {
    DataSpecification { data_spec: ffi::parse_data_specification(text).expect("failed to parse data specification") }
  }
}