//!
//! Safe abstraction for the LPS library,
//!

use std::{fmt, error::Error};

use mcrl2_sys::{cxx::UniquePtr, lps::ffi};

use crate::data::DataSpecification;

/// Rust representation of a lps::linear_process_specification.
pub struct LinearProcessSpecification {
    lps: UniquePtr<ffi::specification>,
}

impl LinearProcessSpecification {

    /// Reads the linear process specification from the given path.
    pub fn read(filename: &str) -> Result<LinearProcessSpecification, Box<dyn Error>> {
        Ok(LinearProcessSpecification {
            lps: ffi::read_linear_process_specification(filename)?,
        })
    }

    /// Returns the underlying data specification.
    pub fn data_specification(&self) -> DataSpecification {
        DataSpecification {
            data_spec: ffi::get_data_specification(&self.lps)
        }
    }
}

impl fmt::Display for LinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ffi::print_linear_process_specification(&self.lps))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_linear_process_specification()
    {
        let lps = LinearProcessSpecification::read("../../examples/lps/abp.lps").unwrap();

        let _data_spec = lps.data_specification();

        println!("{}", lps);
    }
}