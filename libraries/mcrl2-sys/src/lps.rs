use cxx::UniquePtr;
use std::fmt;

use crate::lps_ffi::ffi::*;

/// Rust representation of a lps::linear_process_specification.
pub struct LinearProcessSpecification {
    lps: UniquePtr<specification>,
}

impl LinearProcessSpecification {
    pub fn read(filename: &str) -> LinearProcessSpecification {
        LinearProcessSpecification {
            lps: read_linear_process_specification(filename).expect("cannot read given lps."),
        }
    }
}

impl fmt::Display for LinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", print_linear_process_specification(&self.lps))
    }
}
