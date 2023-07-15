#[cxx::bridge(namespace = "atermpp")]
pub(crate) mod ffi {

    /// This is an abstraction of unprotected_aterm that exists on both sides.
    struct aterm_ref {
        index: usize,
    }

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/atermpp/aterm.h");

        type aterm;
        type function_symbol;

        /// Initialises the library.
        fn initialise();

        /// Trigger garbage collection.
        fn collect_garbage();

        /// Creates a default term.
        fn new_aterm() -> UniquePtr<aterm>;

        /// Creates a term from the given function and arguments.
        fn create_aterm(function: &function_symbol, arguments: &[aterm_ref]) -> UniquePtr<aterm>;

        /// Parses the given string and returns an aterm
        fn aterm_from_string(text: String) -> Result<UniquePtr<aterm>>;

        /// Returns true iff the term is an aterm_int.
        fn aterm_is_int(term: &aterm) -> bool;

        /// Returns the address of the given aterm. Should be used with care.
        fn aterm_pointer(term: &aterm) -> usize;

        /// Converts an aterm to a string.
        fn print_aterm(term: &aterm) -> String;

        /// Computes the hash for an aterm.
        fn hash_aterm(term: &aterm) -> usize;

        /// Returns true iff the terms are equivalent.
        fn equal_aterm(first: &aterm, second: &aterm) -> bool;

        /// Returns true iff the first term is less than the second term.
        fn less_aterm(first: &aterm, second: &aterm) -> bool;

        /// Makes a copy of the given term.
        fn copy_aterm(term: &aterm) -> UniquePtr<aterm>;

        /// Returns the function symbol of an aterm.
        fn get_aterm_function_symbol(term: &aterm) -> UniquePtr<function_symbol>;

        /// Returns the function symbol name
        fn get_function_symbol_name(symbol: &function_symbol) -> &str;

        /// Returns the function symbol name
        fn get_function_symbol_arity(symbol: &function_symbol) -> usize;

        /// Returns the hash for a function symbol
        fn hash_function_symbol(symbol: &function_symbol) -> usize;

        fn equal_function_symbols(first: &function_symbol, second: &function_symbol) -> bool;

        fn less_function_symbols(first: &function_symbol, second: &function_symbol) -> bool;

        /// Makes a copy of the given function symbol
        fn copy_function_symbol(symbol: &function_symbol) -> UniquePtr<function_symbol>;

        /// Returns the ith argument of this term.
        fn get_term_argument(term: &aterm, index: usize) -> UniquePtr<aterm>;

        /// Creates a function symbol with the given name and arity.
        fn create_function_symbol(name: String, arity: usize) -> UniquePtr<function_symbol>;

        fn function_symbol_address(symbol: &function_symbol) -> usize;

        /// For data::variable
        fn is_data_variable(term: &aterm) -> bool;

        fn create_data_variable(name: String) -> UniquePtr<aterm>;

        /// For data::function_symbol        
        fn is_data_function_symbol(term: &aterm) -> bool;

        fn create_data_function_symbol(name: String) -> UniquePtr<aterm>;
    }
}