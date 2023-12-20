#[cxx::bridge(namespace = "atermpp")]
pub mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-sys/cpp/atermpp/atermpp.h");

        type aterm;
        type function_symbol;
        type callback_container;
        type term_mark_stack;

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

        /// Enable automated garbage collection.
        /// 
        /// # Warning
        /// This will deadlock when any Rust terms are created due to the interaction with the protection sets.
        /// Instead, call collect_garbage periodically to trigger garbage collection when needed.
        fn enable_automatic_garbage_collection(enabled: bool);

        /// Trigger garbage collection.
        fn collect_garbage();

        /// Provides shared access to the aterm library.
        fn lock_shared();
        fn unlock_shared();

        /// Provides exclusive access to the aterm library.
        fn lock_exclusive();
        fn unlock_exclusive();

        /// Register a function to be called during marking of the garbage collection
        fn register_mark_callback(callback_mark: fn(Pin<&mut term_mark_stack>) -> (), callback_size: fn() -> usize) -> UniquePtr<callback_container>;

        /// Prints various metrics that are being tracked for terms.
        fn print_metrics();

        /// Creates a term from the given function and arguments, must be protected
        unsafe fn create_aterm(function: *const _function_symbol, arguments: &[*const _aterm]) -> *const _aterm;
        
        /// Parses the given string and returns an aterm
        fn aterm_from_string(text: String) -> Result<UniquePtr<aterm>>;

        /// Returns the pointer underlying the given term.
        unsafe fn aterm_address(term: &aterm) -> *const _aterm;

        /// Marks the aterm to prevent garbage collection.
        unsafe fn aterm_mark_address(term: *const _aterm, todo: Pin<&mut term_mark_stack>);

        /// Returns true iff the term is an aterm_list.
        unsafe fn aterm_is_list(term: *const _aterm) -> bool;

        /// Returns true iff the term is the empty aterm_list.
        unsafe fn aterm_is_empty_list(term: *const _aterm) -> bool;

        /// Returns true iff the term is an aterm_int.
        unsafe fn aterm_is_int(term: *const _aterm) -> bool;

        /// Converts an aterm to a string.
        unsafe fn print_aterm(term: *const _aterm) -> String;

        /// Returns the function symbol of an aterm.
        unsafe fn get_aterm_function_symbol(term: *const _aterm) -> *const _function_symbol;

        /// Returns the function symbol name
        unsafe fn get_function_symbol_name<'a>(symbol: *const _function_symbol) -> &'a str;

        /// Returns the function symbol name
        unsafe fn get_function_symbol_arity(symbol: *const _function_symbol) -> usize;

        /// Returns the ith argument of this term.
        unsafe fn get_term_argument(term: *const _aterm, index: usize) -> *const _aterm;

        /// Creates a function symbol with the given name and arity, increases the reference counter by one.
        fn create_function_symbol(name: String, arity: usize) -> *const _function_symbol;

        /// Protects the given function symbol by incrementing the reference counter.
        unsafe fn protect_function_symbol(symbol: *const _function_symbol);

        /// Decreases the reference counter of the function symbol by one.
        unsafe fn drop_function_symbol(symbol: *const _function_symbol);

        /// Obtain the address of the given function symbol.
        unsafe fn function_symbol_address(symbol: &function_symbol) -> *const _function_symbol;

        // For data::variable
        unsafe fn is_data_variable(term: *const _aterm) -> bool;

        /// Creates an unprotected data variable, must be within in a critical section.
        fn create_data_variable(name: String) -> *const _aterm;

        // For data::function_symbol        
        unsafe fn is_data_function_symbol(term: *const _aterm) -> bool;

        /// Creates an unprotected data function symbol, must be within in a critical section.
        fn create_data_function_symbol(name: String) -> *const _aterm;

        // For data::data_expression        
        unsafe fn is_data_where_clause(term: *const _aterm) -> bool;
        unsafe fn is_data_abstraction(term: *const _aterm) -> bool;
        unsafe fn is_data_untyped_identifier(term: *const _aterm) -> bool;

        /// This function is to generate necessary data types
        fn generate_types() -> UniquePtr<CxxVector<aterm>>;
    }
}