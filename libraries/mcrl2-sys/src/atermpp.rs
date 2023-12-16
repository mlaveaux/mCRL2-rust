#[cxx::bridge(namespace = "atermpp")]
pub mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/atermpp/atermpp.h");

        type aterm;
        type function_symbol;

        /// The underlying detail::_aterm
        #[namespace = "atermpp::detail"]
        type _aterm;
        #[namespace = "atermpp::detail"]
        type _function_symbol;

        #[namespace = "mcrl2::utilities"]
        type shared_guard;
        #[namespace = "mcrl2::utilities"]
        type lock_guard;

        /// Initialises the library.
        fn initialise();

        /// Trigger garbage collection.
        fn collect_garbage();

        /// Provides shared access to the aterm library.
        fn lock_shared();
        fn unlock_shared();

        /// Provides exclusive access to the aterm library.
        fn lock_exclusive();
        fn unlock_exclusive();

        /// Prints various metrics that are being tracked for terms.
        fn print_metrics();

        /// Creates a term from the given function and arguments.
        unsafe fn create_aterm(function: *const _function_symbol, arguments: &[*const _aterm]) -> UniquePtr<aterm>;
        
        /// Parses the given string and returns an aterm
        fn aterm_from_string(text: String) -> Result<UniquePtr<aterm>>;

        /// Protect the given aterm.
        unsafe fn protect_aterm(value: *const _aterm) -> UniquePtr<aterm>;

        /// Returns true iff the term is an aterm_list.
        unsafe fn aterm_is_list(term: *const _aterm) -> bool;

        /// Returns true iff the term is the empty aterm_list.
        unsafe fn aterm_is_empty_list(term: *const _aterm) -> bool;

        /// Returns true iff the term is an aterm_int.
        unsafe fn aterm_is_int(term: *const _aterm) -> bool;

        /// Returns the address of the given aterm. Should be used with care.
        fn aterm_address(term: &aterm) -> *const _aterm;

        /// Converts an aterm to a string.
        unsafe fn print_aterm(term: *const _aterm) -> String;

        /// Convert pointer into a protected function symbol.
        unsafe fn protect_function_symbol(value: *const _function_symbol) -> UniquePtr<function_symbol>;

        /// Returns the function symbol of an aterm.
        unsafe fn get_aterm_function_symbol(term: *const _aterm) -> *const _function_symbol;

        /// Returns the function symbol name
        unsafe fn get_function_symbol_name<'a>(symbol: *const _function_symbol) -> &'a str;

        /// Returns the function symbol name
        unsafe fn get_function_symbol_arity(symbol: *const _function_symbol) -> usize;

        /// Returns the ith argument of this term.
        unsafe fn get_term_argument(term: *const _aterm, index: usize) -> *const _aterm;

        /// Creates a function symbol with the given name and arity.
        fn create_function_symbol(name: String, arity: usize) -> UniquePtr<function_symbol>;

        unsafe fn function_symbol_address(symbol: &function_symbol) -> *const _function_symbol;

        // For data::variable
        unsafe fn is_data_variable(term: *const _aterm) -> bool;

        fn create_data_variable(name: String) -> UniquePtr<aterm>;

        // For data::function_symbol        
        unsafe fn is_data_function_symbol(term: *const _aterm) -> bool;

        fn create_data_function_symbol(name: String) -> UniquePtr<aterm>;

        // For data::data_expression        
        unsafe fn is_data_where_clause(term: *const _aterm) -> bool;
        unsafe fn is_data_abstraction(term: *const _aterm) -> bool;
        unsafe fn is_data_untyped_identifier(term: *const _aterm) -> bool;

        /// This function is to generate necessary data types
        fn generate_types() -> UniquePtr<CxxVector<aterm>>;
    }
}