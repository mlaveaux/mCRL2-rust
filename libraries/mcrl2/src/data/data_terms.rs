use core::fmt;

use crate::aterm::{ATerm, ATermRef, ATermTrait, SymbolTrait, ATermArgs, THREAD_TERM_POOL};
use mcrl2_macros::mcrl2_derive_terms;
use mcrl2_sys::data::ffi;

pub fn is_data_variable(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_variable(term.get()) }
}

pub fn is_data_expression(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    is_data_variable(term)
        || is_data_function_symbol(term)
        || is_data_application(term)
        || is_data_abstraction(term)
        || is_data_where_clause(term)
}

pub fn is_data_function_symbol(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_function_symbol(term.get()) }
}

pub fn is_sort_expression(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_sort_expression(term.get()) }
}

pub fn is_data_where_clause(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_where_clause(term.get()) }
}

pub fn is_data_abstraction(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_abstraction(term.get()) }
}

pub fn is_data_untyped_identifier(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_untyped_identifier(term.get()) }
}

pub fn is_bool_sort(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    true
}

pub fn is_data_application(term: &ATermRef<'_>) -> bool {
    term.require_valid();

    THREAD_TERM_POOL.with_borrow_mut(|tp| {
        tp.is_data_application(term)
    })
}

// This module is only used internally to run the proc macro.
#[mcrl2_derive_terms]
mod inner {
    use super::*;
    
    use std::ops::Deref;
    
    use mcrl2_macros::mcrl2_term;
    use crate::aterm::{Markable, Todo, TermPool};

    /// A data expression can be any of:
    ///     - a variable
    ///     - a function symbol, i.e. f without arguments.
    ///     - a term applied to a number of arguments, i.e., t_0(t1, ..., tn).
    ///     - an abstraction lambda x: Sort . e, or forall and exists.
    /// 
    /// Not supported:
    ///     - a where clause "e where [x := f, ...]"
    ///     - set enumeration
    ///     - bag enumeration
    /// 
    #[mcrl2_term(is_data_expression)]
    pub struct DataExpression {
        term: ATerm,
    }
    
    impl DataExpression {    

        /// Returns the head symbol a data expression
        ///     - function symbol                  f -> f
        ///     - application       f(t_0, ..., t_n) -> f
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            if is_data_application(&self.term) {
                self.term.arg(0).upgrade(&self.term).into()
            } else if is_data_function_symbol(&self.term) {
                self.term.copy().into()
            } else {
                panic!("data_function_symbol not implemented for {}", self);
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol                  f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn data_arguments(&self) -> ATermArgs<'_> {
            if is_data_application(&self.term) {
                let mut result = self.term.arguments();
                result.next();
                result
            } else if is_data_function_symbol(&self.term) {
                Default::default()
            } else {
                panic!("data_arguments not implemented for {}", self);
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol                  f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn data_sort(&self) -> SortExpression {
            if is_data_function_symbol(&self.term) {
                DataFunctionSymbolRef::from(self.term.copy()).sort().protect()
            } else if is_data_variable(&self.term) {
                DataVariableRef::from(self.term.copy()).sort().protect()
            } else {
                panic!("data_sort not implemented for {}", self);
            }
        }
    }
 
    impl fmt::Display for DataExpression {        
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if is_data_function_symbol(&self.term) {
                write!(f, "{}", DataFunctionSymbolRef::from(self.term.copy()))
            } else if is_data_application(&self.term) {
                write!(f, "{}", DataApplicationRef::from(self.term.copy()))
            } else if is_data_variable(&self.term) {
                write!(f, "{}", DataVariableRef::from(self.term.copy()))
            } else {
                write!(f, "{}", self.term)
            }
        }
    }

    #[mcrl2_term(is_data_function_symbol)]
    pub struct DataFunctionSymbol {
        pub(crate) term: ATerm,
    }

    impl DataFunctionSymbol {
        pub fn new(tp: &mut TermPool, name: &str) -> DataFunctionSymbol {
            DataFunctionSymbol {
                term: tp.create_with(|| {
                    mcrl2_sys::data::ffi::create_data_function_symbol(name.to_string())
                })
            }
        }

        pub fn sort(&self) -> SortExpressionRef<'_> {
            self.arg(1).into()
        }

        pub fn name(&self) -> String {
            self.term.arg(0).get_head_symbol().name().to_string()
        }

        pub fn is_default(&self) -> bool {
            self.term.is_default()
        }

        /// Returns the internal operation id (a unique number) for the data::function_symbol.
        pub fn operation_id(&self) -> usize {
            debug_assert!(
                is_data_function_symbol(&self.term),
                "term {} is not a data function symbol",
                self.term
            );
            unsafe { ffi::get_data_function_symbol_index(self.term.get()) }
        }
    }

    impl fmt::Display for DataFunctionSymbol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if !self.is_default() {
                write!(f, "{}", self.name()) 
            } else {
                write!(f, "<default>")
            }       
        }
    }

    #[mcrl2_term(is_data_variable)]
    pub struct DataVariable {
        pub(crate) term: ATerm,
    }

    impl DataVariable {
        pub fn new(tp: &mut TermPool, name: &str) -> DataVariable {
            DataVariable {
                term: tp.create_with(|| {
                    mcrl2_sys::data::ffi::create_data_variable(name.to_string())
                }),
            }
        }

        pub fn name(&self) -> String {
            String::from(self.arg(0).get_head_symbol().name())
        }

        pub fn sort(&self) -> SortExpressionRef<'_> {
            self.arg(1).into()
        }
    }

    impl fmt::Display for DataVariable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    #[mcrl2_term(is_data_application)]
    pub struct DataApplication {
        pub(crate) term: ATerm,
    }

    impl DataApplication {
        pub fn new(tp: &mut TermPool, head: &ATermRef<'_>, arguments: &[impl ATermTrait]) -> DataApplication{
            DataApplication {
                term: tp.create_data_application(head, arguments)
            }
        }
        
        /// Returns the head symbol a data application
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            self.term.arg(0).upgrade(&self.term).into()
        }

        /// Returns the arguments of a data application
        pub fn data_arguments(&self) -> ATermArgs<'_> {
            let mut result = self.term.arguments();
            result.next();
            result
        }

        /// Create a new data application from the given head symbol and arguments.
        pub fn from_refs(tp: &mut TermPool, head: &ATermRef<'_>, arguments: &[DataExpressionRef<'_>]) -> DataApplication{
            DataApplication {
                term: tp.create_data_application2(head, arguments)
            }
        }
        
        /// Returns the sort of a data application.
        pub fn sort(&self) -> SortExpression {
            DataFunctionSymbolRef::from(self.arg(0)).sort().protect()
        }
    }
        
    impl fmt::Display for DataApplication {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {    
            write!(f, "{}", self.data_function_symbol())?;
    
            let mut first = true;
            for arg in self.data_arguments() {
                if !first {
                    write!(f, ", ")?;
                } else {
                    write!(f, "(")?;
                }
    
                write!(f, "{}", DataExpressionRef::from(arg.copy()))?;
                first = false;
            }
    
            if !first {
                write!(f, ")")?;
            }
    
            Ok(())
        }
    }

    #[mcrl2_term(is_bool_sort)]
    pub struct BoolSort {
        pub(crate) term: ATerm,
    }
    
    impl BoolSort {

        /// Returns the term representing true.
        pub fn true_term() -> DataExpression {
            DataExpression {
                term: ffi::true_term().into(),
            }
        }
    
        /// Returns the term representing false.
        pub fn false_term() -> DataExpression {
            DataExpression {
                term: ffi::false_term().into(),
            }
        }
    }

    #[mcrl2_term(is_sort_expression)]
    pub struct SortExpression {
        term: ATerm,
    }

    impl SortExpression {

        /// Returns the name of the sort.
        pub fn name(&self) -> String {
            String::from(self.arg(0).get_head_symbol().name())
        }

        /// Returns true iff this is a basic sort
        pub fn is_basic_sort(&self) -> bool {
            unsafe { ffi::is_data_basic_sort(self.term.get()) }
        }
        
        /// Returns true iff this is a function sort
        pub fn is_function_sort(&self) -> bool {
            unsafe { ffi::is_data_function_sort(self.term.get()) }
        }
    }
    
    impl fmt::Display for SortExpression {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }
}

pub use inner::*;

impl From<DataFunctionSymbol> for DataExpression {
    fn from(value: DataFunctionSymbol) -> Self {
        value.term.into()
    }
}

impl From<DataApplication> for DataExpression {
    fn from(value: DataApplication) -> Self {
        value.term.into()
    }
}

impl From<DataVariable> for DataExpression {
    fn from(value: DataVariable) -> Self {
        value.term.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{aterm::{TermPool, ATerm}, data::{is_data_application, DataFunctionSymbol, DataApplication}};

    #[test]
    fn test_print() {
        let mut tp = TermPool::new();

        let a = DataFunctionSymbol::new(&mut tp, "a");
        assert_eq!("a", format!("{}", a));

        // Check printing of data applications.
        let f = DataFunctionSymbol::new(&mut tp, "f");
        let a_term: ATerm = a.clone().into();
        let appl = DataApplication::new(&mut tp, &f.copy().into(), &[a_term]);
        assert_eq!("f(a)", format!("{}", appl));
    }

    #[test]
    fn test_recognizers() {
        let mut tp = TermPool::new();
        
        let a = DataFunctionSymbol::new(&mut tp, "a");
        let f = DataFunctionSymbol::new(&mut tp, "f");
        let a_term: ATerm = a.clone().into();
        let appl = DataApplication::new(&mut tp, &f.copy().into(), &[a_term]);

        let term: ATerm = appl.into();
        assert!(is_data_application(&term));
    }
}