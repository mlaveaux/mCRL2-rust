use core::fmt;

use crate::aterm::{ATerm, ATermRef, ATermTrait, SymbolTrait};
use mcrl2_sys::data::ffi;

pub fn is_data_variable(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_variable(term.get()) }
}

pub fn is_data_expression(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_variable(term.get()) }
}

pub fn is_data_function_symbol(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_function_symbol(term.get()) }
}

pub fn is_data_where_clause(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_where_clause(term.get()) }
}

pub fn is_data_abstraction(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_abstraction(term.get()) }
}

pub fn is_data_untyped_identifier(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_untyped_identifier(term.get()) }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataVariable {
    pub(crate) term: ATerm,
}

impl DataVariable {
    pub fn name(&self) -> String {
        String::from(self.term.arg(0).get_head_symbol().name())
    }
}

impl From<ATerm> for DataVariable {
    fn from(value: ATerm) -> Self {
        debug_assert!(
            value.is_data_variable(),
            "Term {value} is not a data variable"
        );
        DataVariable { term: value }
    }
}

impl fmt::Display for DataVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}


#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataApplication {
    pub(crate) term: ATerm,
}

impl From<ATerm> for DataApplication {
    fn from(value: ATerm) -> Self {
        //debug_assert!(value.is_data_application(), "Term {value} is not a data application");
        DataApplication { term: value }
    }
}

impl fmt::Display for DataApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = self.term.arguments();

        let head = args.next().unwrap();
        if head.is_data_function_symbol() {
            write!(f, "{}", DataFunctionSymbolRef::from(head))?;
        } else {
            write!(f, "{:?}", head)?;
        }

        let mut first = true;
        for arg in args {
            if !first {
                write!(f, ", ")?;
            } else {
                write!(f, "(")?;
            }

            if arg.is_data_application() {
                write!(f, "{}", DataApplication::from(arg.protect()))?;
            } else if arg.is_data_function_symbol() {
                write!(f, "{}", DataFunctionSymbolRef::from(arg))?;
            } else {
                write!(f, "{}", arg)?;
            }
            first = false;
        }

        if !first {
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl fmt::Debug for DataApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// The data::function_symbol
#[derive(Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataFunctionSymbolRef<'a> {
    pub(crate) term: ATermRef<'a>,
}

impl<'a> DataFunctionSymbolRef<'a> {
    pub fn name(&self) -> String {
        String::from(self.term.arg(0).get_head_symbol().name())
    }

    pub fn borrow(&self) -> ATermRef<'_> {
        self.term.borrow()
    }

    pub fn protect(&self) -> DataFunctionSymbol {
        DataFunctionSymbol {
            term: self.term.protect()
        }
    }

    /// Returns the internal id known for every [aterm] that is a data::function_symbol.
    pub fn operation_id(&self) -> usize {
        debug_assert!(
            self.term.is_data_function_symbol(),
            "term {} is not a data function symbol",
            self.term
        );
        unsafe { ffi::get_data_function_symbol_index(self.term.borrow().get()) }
    }
}

impl<'a> Into<ATermRef<'a>> for DataFunctionSymbolRef<'a> {
    fn into(self) -> ATermRef<'a> {
        self.term
    }
}

impl<'a> From<ATermRef<'a>> for DataFunctionSymbolRef<'a> {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(
            value.is_data_function_symbol(),
            "Term {value:?} is not a data function symbol"
        );
        DataFunctionSymbolRef { term: value }
    }
}

impl<'a> fmt::Display for DataFunctionSymbolRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.term.is_default() {
            write!(f, "{}", &self.name())
        } else {
            write!(f, "<default>")
        }
    }
}

pub struct BoolSort {
    pub(crate) term: ATerm,
}

impl BoolSort {
    pub fn true_term() -> BoolSort {
        BoolSort {
            term: ffi::true_term().into(),
        }
    }

    pub fn false_term() -> BoolSort {
        BoolSort {
            term: ffi::false_term().into(),
        }
    }
}

impl From<ATerm> for BoolSort {
    fn from(value: ATerm) -> Self {
        BoolSort { term: value }
    }
}

#[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataFunctionSymbol {
    pub(crate) term: ATerm,
}

impl DataFunctionSymbol {
    pub fn borrow(&self) -> DataFunctionSymbolRef<'_> {
        DataFunctionSymbolRef {
            term: self.term.borrow()
        }
    }

    pub fn operation_id(&self) -> usize {
        self.borrow().operation_id()
    }
}

impl fmt::Display for DataFunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.borrow())        
    }
}

impl<'a> From<ATerm> for DataFunctionSymbol {
    fn from(value: ATerm) -> Self {
        debug_assert!(
            value.is_data_function_symbol(),
            "Term {value:?} is not a data function symbol"
        );
        DataFunctionSymbol { term: value }
    }
}

impl Into<ATerm> for DataVariable {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for DataApplication {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for DataFunctionSymbol {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for BoolSort {
    fn into(self) -> ATerm {
        self.term        
    }
}

#[cfg(test)]
mod tests {
    use crate::aterm::{TermPool, ATerm, ATermTrait};

    #[test]
    fn test_print() {
        let mut tp = TermPool::new();

        let a = tp.create_data_function_symbol("a");
        assert_eq!("a", format!("{}", a));

        // Check printing of data applications.
        let f = tp.create_data_function_symbol("f");
        let a_term: ATerm = a.clone().into();
        let appl = tp.create_data_application(&f.borrow().into(), &[a_term]);
        assert_eq!("f(a)", format!("{}", appl));
    }

    #[test]
    fn test_recognizers() {
        let mut tp = TermPool::new();
        
        let a = tp.create_data_function_symbol("a");
        let f = tp.create_data_function_symbol("f");
        let a_term: ATerm = a.clone().into();
        let appl = tp.create_data_application(&f.borrow().into(), &[a_term]);

        let term: ATerm = appl.into();
        assert!(term.is_data_application());
       ;

    }
}