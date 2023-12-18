use std::marker::PhantomData;

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt;

use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::atermpp::ffi::{self, _function_symbol, function_symbol};

/// A Symbol references to an aterm function symbol, which has a name and an arity.
pub trait SymbolTrait {
    /// Obtain the symbol's name
    fn name(&self) -> &str;

    /// Obtain the symbol's arity
    fn arity(&self) -> usize;

    /// Returns the index of the function symbol
    fn address(&self) -> *const ffi::_function_symbol;
}

// TODO: How to use this for all symbol impls
impl fmt::Display for dyn SymbolTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for dyn SymbolTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} [{}]",
            self.name(),
            self.arity(),
            self.address() as usize,
        )
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRef<'a> {
    symbol: *const ffi::_function_symbol,
    marker: PhantomData<&'a ()>
}

impl<'a> SymbolRef<'a> {
    fn new(symbol: *const ffi::_function_symbol,) -> SymbolRef<'a> {   
        unsafe {
            SymbolRef {
                symbol,
                marker: PhantomData,
            }
        }
    }

    pub fn protect(&self) -> Symbol {
        Symbol::new(self.symbol)
    }
}

impl<'a> SymbolTrait for SymbolRef<'a> {
    fn name(&self) -> &str {
        unsafe {
            ffi::get_function_symbol_name(self.symbol)
        }
    }

    fn arity(&self) -> usize {
        unsafe {
            ffi::get_function_symbol_arity(self.symbol)
        }
    }

    fn address(&self) -> *const ffi::_function_symbol {
        self.symbol
    }
}

impl<'a> From<*const ffi::_function_symbol> for SymbolRef<'a> {
    fn from(symbol: *const ffi::_function_symbol) -> Self {
        SymbolRef {
            symbol,
            marker: PhantomData
        }
    }
}

pub struct Symbol {
    symbol: *const ffi::_function_symbol,
}

impl Symbol {
    /// Takes ownership of the given pointer without changing the reference counter.
    pub(crate) fn take(symbol: *const ffi::_function_symbol) -> Symbol {  
        Symbol {
            symbol
        }
    }

    /// Protects the given pointer.
    pub(crate) fn new(symbol: *const ffi::_function_symbol) -> Symbol {     
        unsafe { ffi::protect_function_symbol(symbol) };   
        Symbol {
            symbol
        }
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe { ffi::drop_function_symbol(self.symbol) };
    }
}

impl<'a> Symbol {
    pub fn borrow<'b: 'a>(&'a self) -> SymbolRef<'b> {
        SymbolRef::new(self.symbol)
    }
}

impl<'a> From<&SymbolRef<'a>> for Symbol {
    fn from(value: &SymbolRef) -> Self {
        value.protect()
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        self.borrow().protect()
    }
}

/// TODO: These might be derivable?
impl<'a> fmt::Display for SymbolRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> fmt::Debug for SymbolRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl SymbolTrait for Symbol {
    fn name(&self) -> &str {
        unsafe {
            ffi::get_function_symbol_name(self.symbol)
        }
    }

    fn arity(&self) -> usize {
        self.borrow().arity()
    }

    fn address(&self) -> *const _function_symbol {
        self.borrow().address()
    }
}


impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().eq(&other.borrow())
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.borrow().cmp(&other.borrow()))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&other.borrow())
    }
}

impl Eq for Symbol {}