use std::fmt;

use mcrl2_sys::{cxx::UniquePtr, lps::ffi};

/// Rust representation of a lps::linear_process_specification.
pub struct LinearProcessSpecification {
    lps: UniquePtr<ffi::specification>,
}

impl LinearProcessSpecification {
    pub fn read(filename: &str) -> LinearProcessSpecification {
        LinearProcessSpecification {
            lps: ffi::read_linear_process_specification(filename).expect("cannot read given lps."),
        }
    }
}

impl fmt::Display for LinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ffi::print_linear_process_specification(&self.lps))
    }
}
