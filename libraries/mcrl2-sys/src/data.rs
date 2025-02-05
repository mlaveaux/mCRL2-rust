#[cxx::bridge(namespace = "mcrl2::data")]
#[allow(clippy::missing_safety_doc)]
pub mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/data/data.h");

        type data_specification;

        #[namespace = "mcrl2::data::detail"]
        type RewriterJitty;

        #[namespace = "mcrl2::data::detail"]
        type RewriterCompilingJitty;

        #[namespace = "atermpp"]
        type aterm = crate::atermpp::ffi::aterm;

        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        /// Parses the given text into a data specification.
        fn parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;

        /// Parses the given text and typechecks it using the given data specification
        fn parse_data_expression(text: &str, data_spec: &data_specification) -> Result<UniquePtr<aterm>>;

        /// Parses the given text v: Sort as a variable and typechecks it using the given data specification
        fn parse_variable(text: &str, data_spec: &data_specification) -> Result<UniquePtr<aterm>>;

        /// Returns the data equations for the given specification.
        fn get_data_specification_equations(data_spec: &data_specification) -> UniquePtr<CxxVector<aterm>>;

        /// Returns the data constructors for the given sort.
        unsafe fn get_data_specification_constructors(
            data_spec: &data_specification,
            sort: *const _aterm,
        ) -> UniquePtr<CxxVector<aterm>>;

        /// Creates an instance of the jitty rewriter.
        fn create_jitty_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

        /// Creates an instance of the compiling jitty rewriter.
        fn create_jitty_compiling_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterCompilingJitty>;

        /// Rewrites the given term to normal form.
        unsafe fn rewrite(rewriter: Pin<&mut RewriterJitty>, term: *const _aterm) -> UniquePtr<aterm>;

        /// Clone the data specification
        fn data_specification_clone(data_spec: &data_specification) -> UniquePtr<data_specification>;

        /// Obtain the index assigned internally to every data function symbol.
        unsafe fn get_data_function_symbol_index(term: *const _aterm) -> usize;

        /// Create the data::true term
        fn true_term() -> UniquePtr<aterm>;

        /// Create the data::false term
        fn false_term() -> UniquePtr<aterm>;

        // For data::variable
        unsafe fn is_data_variable(term: *const _aterm) -> bool;

        /// Creates an unsorted data variable, must be within in a critical section.
        fn create_data_variable(name: String) -> *const _aterm;

        /// Creates an sorted data variable, must be within in a critical section.
        unsafe fn create_sorted_data_variable(name: String, sort: *const _aterm) -> *const _aterm;

        // For data::function_symbol
        unsafe fn is_data_function_symbol(term: *const _aterm) -> bool;

        /// Creates an unprotected data function symbol, must be within in a critical section.
        fn create_data_function_symbol(name: String) -> *const _aterm;

        // For data::sort_expression
        unsafe fn is_data_sort_expression(term: *const _aterm) -> bool;
        unsafe fn is_data_basic_sort(term: *const _aterm) -> bool;
        unsafe fn is_data_function_sort(term: *const _aterm) -> bool;

        // For data::data_expression
        unsafe fn is_data_where_clause(term: *const _aterm) -> bool;
        unsafe fn is_data_abstraction(term: *const _aterm) -> bool;
        unsafe fn is_data_untyped_identifier(term: *const _aterm) -> bool;
        unsafe fn is_data_machine_number(term: *const _aterm) -> bool;
    }
}
