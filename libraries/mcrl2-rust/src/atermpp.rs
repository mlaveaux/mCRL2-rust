use cxx::{Exception, UniquePtr};
use std::{cmp::Ordering, fmt, hash::Hash, hash::Hasher};
use std::collections::VecDeque;

use crate::data::{DataVariable, DataApplication, DataFunctionSymbol};

#[cxx::bridge(namespace = "atermpp")]
pub mod ffi {

    /// This is an abstraction of unprotected_aterm that can only exist on the Rust side of code.
    struct aterm_ref {
        index: usize,
    }

    unsafe extern "C++" {
        include!("mcrl2-rust/cpp/atermpp/aterm.h");

        type aterm;
        type function_symbol;

        /// Initialises the library.
        fn initialise();

        /// Creates a default term.
        fn new_aterm() -> UniquePtr<aterm>;

        /// Creates a term from the given function and arguments.
        fn create_aterm(function: &function_symbol, arguments: &[aterm_ref]) -> UniquePtr<aterm>;

        /// Parses the given string and returns an aterm
        fn aterm_from_string(text: String) -> Result<UniquePtr<aterm>>;

        /// Returns true iff the term is an aterm_int.
        fn ffi_is_int(term: &aterm) -> bool;

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
        fn ffi_is_data_variable(term: &aterm) -> bool;

        fn ffi_create_data_variable(name: String) -> UniquePtr<aterm>;

        /// For data::application
        fn ffi_is_data_application(term: &aterm) -> bool;

        fn ffi_create_data_application(head: &aterm, arguments: &[aterm_ref]) -> UniquePtr<aterm>;

        /// For data::function_symbol        
        fn ffi_is_data_function_symbol(term: &aterm) -> bool;

        fn ffi_create_data_function_symbol(name: String) -> UniquePtr<aterm>;

    }
}

/// A Symbol now references to an aterm function symbol, which has a name and an arity.
pub struct Symbol {
    function: UniquePtr<ffi::function_symbol>,
}

impl Symbol {
    pub fn name(&self) -> &str {
        ffi::get_function_symbol_name(&self.function)
    }

    pub fn arity(&self) -> usize {
        ffi::get_function_symbol_arity(&self.function)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if true {
            write!(f, "{}", self.name())
        } else {
            write!(f, "{}:{} [{}]", self.name(), self.arity(), ffi::function_symbol_address(&self.function))
        }
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(ffi::hash_function_symbol(&self.function));
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        ffi::equal_function_symbols(&self.function, &other.function)
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if ffi::less_function_symbols(&self.function, &other.function) {
            Ordering::Less
        } else if self == other {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Symbol {
            function: ffi::copy_function_symbol(&self.function),
        }
    }
}

impl Eq for Symbol {}

/// Rust representation of a atermpp::aterm
pub struct ATerm {
    pub(crate) term: UniquePtr<ffi::aterm>,
}

impl ATerm {
    pub fn from(term: UniquePtr<ffi::aterm>) -> Self {
        ATerm { term }
    }

    /// Get access to the underlying term
    pub fn get(&self) -> &ffi::aterm {
        self.require_valid();
        &self.term
    }

    pub fn arg(&self, index: usize) -> ATerm {
        self.require_valid();
        assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );
        ATerm {
            term: ffi::get_term_argument(&self.term, index),
        }
    }

    pub fn arguments(&self) -> Vec<ATerm> {
        self.require_valid();
        let mut result = vec![];
        for i in 0..self.get_head_symbol().arity() {
            result.push(self.arg(i));
        }
        result
    }

    pub fn is_default(&self) -> bool {
        ffi::aterm_pointer(&self.term) == 0 
    }

    pub fn is_int(&self) -> bool {
        ffi::ffi_is_int(&self.term)
    }
    
    pub fn get_head_symbol(&self) -> Symbol {
        self.require_valid();
        Symbol {
            function: ffi::get_aterm_function_symbol(&self.term),
        }
    }

    /// Returns an iterator over all arguments of the term.
    pub fn iter(&self) -> TermIterator {
        TermIterator::new(self.clone())
    }

    /// Returns true iff the term is not default.
    fn require_valid(&self) {
        assert!(
            !self.is_default(),
            "This function can only be called on valid terms, i.e., not default terms"
        );
    }
    
    // Recognizers for the data library
    pub fn is_data_variable(&self) -> bool {
        self.require_valid();
        ffi::ffi_is_data_variable(&self.term)
    }

    pub fn is_data_application(&self) -> bool {
        self.require_valid();
        ffi::ffi_is_data_application(&self.term)
    }

    pub fn is_data_function_symbol(&self) -> bool {
        self.require_valid();
        ffi::ffi_is_data_function_symbol(&self.term)
    }
}

impl Default for ATerm {
    fn default() -> Self {
        ATerm {
            term: ffi::new_aterm(),
        }
    }
}

impl fmt::Display for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.require_valid();

        if self.is_data_function_symbol() {
            write!(f, "{}", <ATerm as Into<DataFunctionSymbol>>::into(self.clone()))
        } else if self.is_data_application() {
            write!(f, "{}", <ATerm as Into<DataApplication>>::into(self.clone()))
        } else if self.is_data_variable() {
            write!(f, "{}", <ATerm as Into<DataVariable>>::into(self.clone()))
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl fmt::Debug for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        if self.is_default() {
            write!(f, "<default>")?;
        } else {                
            write!(f, "{}", ffi::print_aterm(&self.term))?;
            //for term in self.iter() {   
            //   write!(f, "{:?}: [{}]", term.get_head_symbol(), ffi::aterm_pointer(&self.term))?;
            //}
        }

        Ok(())
    }
}

impl Hash for ATerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(ffi::hash_aterm(&self.term));
    }
}

impl PartialEq for ATerm {
    fn eq(&self, other: &Self) -> bool {
        ffi::equal_aterm(&self.term, &other.term)
    }
}

impl PartialOrd for ATerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        if ffi::less_aterm(&self.term, &other.term) {
            Ordering::Less
        } else if self == other {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        ATerm {
            term: ffi::copy_aterm(&self.term),
        }
    }
}

impl Eq for ATerm {}

impl From<DataVariable> for ATerm {
    fn from(value: DataVariable) -> Self {
        value.term
    }
}

impl From<DataApplication> for ATerm {
    fn from(value: DataApplication) -> Self {
        value.term
    }
}

impl From<DataFunctionSymbol> for ATerm {
    fn from(value: DataFunctionSymbol) -> Self {
        value.term
    }
}

/// This is a standin for the global term pool, with the idea to eventually replace it by a proper implementation.
pub struct TermPool {}

impl TermPool {
    pub fn new() -> TermPool {
        // Initialise the C++ aterm library.
        ffi::initialise();

        TermPool {}
    }

    pub fn from_string(&mut self, text: &str) -> Result<ATerm, Exception> {
        match ffi::aterm_from_string(String::from(text)) {
            Ok(term) => Ok(ATerm { term }),
            Err(exception) => Err(exception),
        }
    }

    /// Creates an [ATerm] with the given symbol and arguments.
    pub fn create(&mut self, symbol: &Symbol, arguments: &[ATerm]) -> ATerm {
        // TODO: This part of the ffi is very slow and should be improved.
        let arguments: Vec<ffi::aterm_ref> = arguments
            .iter()
            .map(|x| ffi::aterm_ref {
                index: ffi::aterm_pointer(x.get()),
            })
            .collect();

        ATerm {
            term: ffi::create_aterm(&symbol.function, &arguments),
        }
    }

    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        Symbol {
            function: ffi::create_function_symbol(String::from(name), arity),
        }
    }

    pub fn create_data_application(&mut self, head: &ATerm, arguments: &[ATerm]) -> DataApplication
    {
        // TODO: This part of the ffi is very slow and should be improved.
        let arguments: Vec<ffi::aterm_ref> = arguments
            .iter()
            .map(|x| ffi::aterm_ref {
                index: ffi::aterm_pointer(x.get()),
            })
            .collect();

        DataApplication { term: ATerm::from(ffi::ffi_create_data_application(head.get(), &arguments)) }
    }

    pub fn create_variable(&mut self, name: &str) -> DataVariable {
        DataVariable {
            term: ATerm::from(ffi::ffi_create_data_variable(String::from(name))),
        }
    }

    pub fn create_data_function_symbol(&mut self, name: &str) -> DataFunctionSymbol
    {
        DataFunctionSymbol { term: ATerm::from(ffi::ffi_create_data_function_symbol(String::from(name))) }
    }
}

/// An iterator over all (term, position) pairs of the given [ATerm].
pub struct TermIterator {
    queue: VecDeque<ATerm>,
}

impl TermIterator {
    pub fn new(t: ATerm) -> TermIterator {
        TermIterator {
            queue: VecDeque::from([t]),
        }
    }
}

impl Iterator for TermIterator {
    type Item = ATerm;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            None
        } else {
            // Get a subterm to inspect
            let term = self.queue.pop_front().unwrap();

            // Put subterms in the queue
            for argument in term.arguments().iter() {
                self.queue.push_back(argument.clone());
            }

            Some(term)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_iterator() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();

        let mut result = t.iter();
        assert_eq!(result.next().unwrap(), tp.from_string("f(g(a),b)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("g(a)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("b").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("a").unwrap());
    }
}
