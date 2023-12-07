#[cxx::bridge(namespace = "mcrl2::data")]
pub mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/data/data.h");

        type data_specification;

        #[namespace = "mcrl2::data::detail"]
        type RewriterJitty;
        
        //#[namespace = "mcrl2::data::detail"]
        //type RewriterCompilingJitty;

        #[namespace = "atermpp"]
        type aterm = crate::atermpp::ffi::aterm;
        
        #[namespace = "atermpp::detail"]
        type _aterm = crate::atermpp::ffi::_aterm;

        /// Parses the given text into a data specification.
        fn parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;

        /// Parses the given text and typechecks it using the given data specification
        fn parse_data_expression(
            text: &str,
            data_spec: &data_specification,
        ) -> UniquePtr<aterm>;

        /// Returns the data equations for the given specification.
        fn get_data_specification_equations(data_spec: &data_specification) -> UniquePtr<CxxVector<aterm>>;

        /// Creates an instance of the jitty rewriter.
        fn create_jitty_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

        /// Creates an instance of the compiling jitty rewriter.
        //fn create_jitty_compiling_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJittyCompiling>;

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
    }
}