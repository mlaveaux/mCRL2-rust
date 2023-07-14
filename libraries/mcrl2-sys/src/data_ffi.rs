#[cxx::bridge(namespace = "mcrl2::data")]
pub(crate) mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/data/data.h");

        type data_specification;

        #[namespace = "mcrl2::data::detail"]
        type RewriterJitty;
        
        //#[namespace = "mcrl2::data::detail"]
        //type RewriterCompilingJitty;

        #[namespace = "atermpp"]
        type aterm = crate::atermpp_ffi::ffi::aterm;

        /// Parses the given text into a data specification.
        fn parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;

        /// Parses the given text and typechecks it using the given data specification
        fn parse_data_expression(
            text: &str,
            data_spec: &data_specification,
        ) -> UniquePtr<aterm>;

        /// Creates an instance of the jitty rewriter.
        fn create_jitty_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

        /// Creates an instance of the compiling jitty rewriter.
        //fn create_jitty_compiling_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJittyCompiling>;

        /// Rewrites the given term to normal form.
        fn rewrite(rewriter: Pin<&mut RewriterJitty>, term: &aterm) -> UniquePtr<aterm>;        
        
        /// Obtain
        fn get_data_function_symbol_index(term: &aterm) -> usize;
    }
}