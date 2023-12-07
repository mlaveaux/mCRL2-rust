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
        if true {
            write!(f, "{}", self.name())
        } else {
            write!(
                f,
                "{}:{} [{}]",
                self.name(),
                self.arity(),
                self.address() as usize,
            )
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolRef<'a> {
    function: *const ffi::_function_symbol,
    marker: PhantomData<&'a ()>
}

impl<'a> SymbolRef<'a> {
    fn new(symbol: &UniquePtr<ffi::function_symbol>) -> SymbolRef<'a> {   
        unsafe {
            SymbolRef {
                function: ffi::function_symbol_address(symbol),
                marker: PhantomData::default(),
            }
        }
    }

    pub fn protect(&self) -> Symbol {
        unsafe {
            Symbol {
                function: ffi::protect_function_symbol(self.function)
            }
        }
    }
}

impl<'a> SymbolTrait for SymbolRef<'a> {
    fn name(&self) -> &str {
        unsafe {
            ffi::get_function_symbol_name(self.function)
        }
    }

    fn arity(&self) -> usize {
        unsafe {
            ffi::get_function_symbol_arity(self.function)
        }
    }

    fn address(&self) -> *const ffi::_function_symbol {
        self.function
    }
}

impl<'a> From<*const ffi::_function_symbol> for SymbolRef<'a> {
    fn from(value: *const ffi::_function_symbol) -> Self {
        SymbolRef {
            function: value,
            marker: PhantomData::default()
        }
    }
}

pub struct Symbol {
    function: UniquePtr<ffi::function_symbol>,
}

impl<'a> Symbol {
    pub fn borrow(&'a self) -> SymbolRef<'a> {
        SymbolRef::new(&self.function)
    }
}

impl<'a> From<&SymbolRef<'a>> for Symbol {
    fn from(value: &SymbolRef) -> Self {
        value.protect()
    }
}

impl From<UniquePtr<function_symbol>> for Symbol {
    fn from(value: UniquePtr<function_symbol>) -> Self {
        Symbol {
            function: value
        }
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
        write!(f, "{}", self)
    }
}

impl<'a> fmt::Debug for SymbolRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SymbolTrait for Symbol {
    fn name(&self) -> &str {
        unsafe {
            ffi::get_function_symbol_name(ffi::function_symbol_address(&self.function))
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
        self.borrow().partial_cmp(&other.borrow())
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&other.borrow())
    }
}

impl Eq for Symbol {}