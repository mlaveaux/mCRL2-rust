#[cxx::bridge(namespace = "mcrl2::lps")]
pub mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/lps/lps.h");

        #[namespace = "mcrl2::data"]
        type data_specification = crate::data::ffi::data_specification;

        type specification;

        /// Reads a .lps file and returns the resulting linear process specification.
        fn read_linear_process_specification(filename: &str) -> Result<UniquePtr<specification>>;

        /// Converts a linear process specification to a string.
        fn print_linear_process_specification(spec: &specification) -> String;

        /// Obtains the related data specification
        fn get_data_specification(spec: &specification) -> UniquePtr<data_specification>;
    }
}