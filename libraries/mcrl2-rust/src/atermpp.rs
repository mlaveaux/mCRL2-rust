use std::{fmt, hash::Hash, hash::Hasher, cmp::Ordering, collections::VecDeque};
use cxx::UniquePtr;

#[cxx::bridge(namespace = "atermpp")]
pub mod ffi {
  
  /// This is an abstraction of unprotected_aterm that can only exist on the Rust side of code.
  struct aterm_ref
  {
    index: usize
  }

  unsafe extern "C++" {    
    include!("mcrl2-rust/cpp/atermpp/aterm.h");

    type aterm;
    type function_symbol;


    /// Creates a default term.
    fn new_aterm() -> UniquePtr<aterm>;

    /// Creates a term from the given function and arguments.
    fn create_aterm(function: &function_symbol, arguments: &[aterm_ref]) -> UniquePtr<aterm>;

    /// Parses the given string and returns an aterm
    fn aterm_from_string(text: String) -> UniquePtr<aterm>;

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

    fn ffi_is_variable(term: &aterm) -> bool;

    fn ffi_create_variable(name: String) -> UniquePtr<aterm>;

    /// For data::function_symbol terms returns the internally known index.
    fn ffi_get_function_symbol_index(term: &aterm) -> usize;
  } 
}

/// A Symbol now references to an aterm function symbol, which has a name and an arity.
pub struct Symbol
{
  function: UniquePtr<ffi::function_symbol>
}

impl Symbol
{
  pub fn name(&self) -> &str
  {
    ffi::get_function_symbol_name(&self.function)
  }

  pub fn arity(&self) -> usize
  {
    ffi::get_function_symbol_arity(&self.function)
  }
}

impl fmt::Display for Symbol 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl fmt::Debug for Symbol 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl Hash for Symbol
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_usize(ffi::hash_function_symbol(&self.function));
  }
}

impl PartialEq for Symbol
{
  fn eq(&self, other: &Self) -> bool
  {
    ffi::equal_function_symbols(&self.function, &other.function)
  }
}

impl PartialOrd for Symbol
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering>
  {
    Some(self.cmp(other))
  }
}

impl Ord for Symbol 
{
  fn cmp(&self, other: &Self) -> Ordering
  {
    if ffi::less_function_symbols(&self.function, &other.function) {
      Ordering::Less
    } else if self == other {
      Ordering::Equal
    }
    else {
      Ordering::Greater
    }
  }

}

impl Clone for Symbol
{
  fn clone(&self) -> Self 
  {
    Symbol {
      function: ffi::copy_function_symbol(&self.function),
    }      
  }
}

impl Eq for Symbol {}

/// Rust representation of a atermpp::aterm
pub struct ATerm
{
  term: UniquePtr<ffi::aterm>
}

impl ATerm
{
  pub fn new() -> Self
  {
    ATerm { term: ffi::new_aterm() }
  }

  pub fn from(term: UniquePtr<ffi::aterm>) -> Self
  {
    ATerm { term }
  }

  /// Get access to the underlying term
  pub fn get(&self) -> &ffi::aterm
  {
    &self.term
  }

  pub fn arg(&self, index: usize) -> ATerm
  {
    assert!(index < self.get_head_symbol().arity(), "The given index should be a valid argument");
    ATerm { term: ffi::get_term_argument(&self.term, index) }
  }

  pub fn arguments(&self) -> Vec<ATerm>
  {
    let mut result = vec![];
    for i in 0..self.get_head_symbol().arity()
    {
      result.push(self.arg(i));
    }
    result
  }

  pub fn is_variable(&self) -> bool
  {
    ffi::ffi_is_variable(&self.term)
  }

  pub fn get_head_symbol(&self) -> Symbol
  {
    Symbol 
    { 
      function: ffi::get_aterm_function_symbol(&self.term), 
    }
  }
}

impl fmt::Display for ATerm 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", ffi::print_aterm(&self.term))
  }
}

impl fmt::Debug for ATerm 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", ffi::print_aterm(&self.term))
  }
}

impl Hash for ATerm
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_usize(ffi::hash_aterm(&self.term));
  }
}

impl PartialEq for ATerm
{
  fn eq(&self, other: &Self) -> bool
  {
    ffi::equal_aterm(&self.term, &other.term)
  }
}

impl PartialOrd for ATerm
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering>
  {
    Some(self.cmp(other))
  }
}

impl Ord for ATerm 
{
  fn cmp(&self, other: &Self) -> Ordering
  {
    if ffi::less_aterm(&self.term, &other.term) {
      Ordering::Less
    } else if self == other {
      Ordering::Equal
    }
    else {
      Ordering::Greater
    }
  }
}

impl Clone for ATerm
{
  fn clone(&self) -> Self 
  {
    ATerm {
      term: ffi::copy_aterm(&self.term),
    }      
  }
}

impl Eq for ATerm {}

/// This is a standin for the global term pool, with the idea to eventually replace it by a proper implementation.
pub struct TermPool
{

}

impl TermPool
{
  pub fn new() -> TermPool
  {
    TermPool {}
  }

  pub fn from_string(&mut self, text: &str) -> Option<ATerm>
  {
    Some(ATerm {
      term: ffi::aterm_from_string(String::from(text))
    })
  }

  /// Creates an [ATerm] with the given symbol and arguments.
  pub fn create(&mut self, symbol: &Symbol, arguments: &[ATerm]) -> ATerm
  {
    // TODO: This part of the ffi is very slow and should be improved.
    //let arguments = vec![arguments.iter().map(|x| ffi::aterm_ref { index: 0 })];

    //ATerm {
    //  term: ffi::create_aterm(&symbol.function, &arguments)
    //}
    ATerm::new()
  }

  pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol
  {
    Symbol {
      function: ffi::create_function_symbol(String::from(name), arity)
    }
  }

  pub fn create_variable(&mut self, name: &str) -> TermVariable
  {
    TermVariable {
      term: ATerm::from(ffi::ffi_create_variable(String::from(name)))
    }
  }
  
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TermVariable
{
  term: ATerm,
}

impl From<ATerm> for TermVariable {
  fn from(value: ATerm) -> Self {
    assert!(value.is_variable(), "The given term should be a variable");
    TermVariable { term: value }
  }
}

impl TermVariable
{
  /*pub fn name(&self) -> &str
  {
    self.term.arg(0).get_head_symbol().name()
  }*/

  // We do not care about it's sort.
}