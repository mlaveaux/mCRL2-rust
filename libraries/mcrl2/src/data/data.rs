use core::fmt;

use crate::aterm::{ATerm, ATermRef, ATermTrait, SymbolTrait, ATermArgs, THREAD_TERM_POOL};
use mcrl2_macros::{mcrl2_term, mcrl2_derive_terms};
use mcrl2_sys::data::ffi;

pub fn is_data_variable(term: ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_variable(term.get()) }
}

pub fn is_data_expression(term: ATermRef<'_>) -> bool {
    term.require_valid();
    is_data_function_symbol(term.borrow())
        || is_data_application(term.borrow())
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

pub fn is_bool_sort(term: ATermRef<'_>) -> bool {
    term.require_valid();
    true
}

pub fn is_data_application(term: ATermRef<'_>) -> bool {
    term.require_valid();

    THREAD_TERM_POOL.with_borrow_mut(|tp| {
        tp.is_data_application(term)
    })
}

// This module is only used internally to run the proc macro.
#[mcrl2_derive_terms]
mod inner {
    use super::*;
    use mcrl2_macros::mcrl2_term;

    /// A data expression:
    ///     - a function symbol, i.e. f without arguments.
    ///     - a term applied to a number of arguments, i.e., t_0(t1, ..., tn).
    #[mcrl2_term(is_data_expression)]
    pub struct DataExpression {
        term: ATerm,
    }
    
    impl DataExpression {    

        /// Returns the head symbol a data expression
        ///     - function symbol   f -> f
        ///     - application       f(t_0, ..., t_n) -> f
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            if is_data_application(self.term.borrow()) {
                self.term.arg(0).upgrade(&self.term.borrow()).into()
            } else {
                self.term.borrow().into()
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol   f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn arguments<'a>(&'a self) -> ATermArgs<'a> {
            if is_data_application(self.term.borrow()) {
                let mut result =self.term.arguments();
                result.next();
                result
            } else {
                Default::default()
            }
        }
    }


    impl fmt::Display for DataExpression {        
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if is_data_function_symbol(self.term.borrow()) {
                write!(f, "{}", DataFunctionSymbolRef::from(self.term.borrow()))
            } else {
                write!(f, "test")
            }
        }
    }
    

    #[mcrl2_term(is_data_function_symbol)]
    pub struct DataFunctionSymbol {
        pub(crate) term: ATerm,
    }

    impl DataFunctionSymbol {
        pub fn is_default(&self) -> bool {
            self.term.is_default()
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

    #[mcrl2_term(is_data_variable)]
    pub struct DataVariable {
        pub(crate) term: ATerm,
    }

    impl DataVariable {
        pub fn name(&self) -> String {
            String::from(self.term.arg(0).get_head_symbol().name())
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
        
    impl fmt::Display for DataApplication {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut args = self.term.arguments();
    
            let head = args.next().unwrap();
            if is_data_function_symbol(head.borrow()) {
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
    
                if is_data_application(arg.borrow()) {
                    write!(f, "{}", DataApplication::from(arg.protect()))?;
                } else if is_data_function_symbol(arg.borrow()) {
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
        
    #[mcrl2_term(is_bool_sort)]
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

    // TODO: This should be derived by the macro.
    impl<'b> DataExpressionRef<'b> {    

        /// Returns the head symbol a data expression
        ///     - function symbol   f -> f
        ///     - application       f(t_0, ..., t_n) -> f
        pub fn data_function_symbol(&self) -> DataFunctionSymbolRef<'_> {
            if is_data_application(self.term.borrow()) {
                self.term.arg(0).upgrade(&self.term.borrow()).into()
            } else {
                self.term.borrow().into()
            }
        }

        /// Returns the arguments of a data expression
        ///     - function symbol   f -> []
        ///     - application       f(t_0, ..., t_n) -> [t_0, ..., t_n]
        pub fn arguments<'a>(&'a self) -> ATermArgs<'a> {
            if is_data_application(self.term.borrow()) {
                let mut result =self.term.arguments();
                result.next();
                result
            } else {
                Default::default()
            }
        }
    }

    impl<'a> DataFunctionSymbolRef<'a> {
        pub fn name(&self) -> String {
            String::from(self.term.arg(0).get_head_symbol().name())
        }
    
        /// Returns the internal id known for every [aterm] that is a data::function_symbol.
        pub fn operation_id(&self) -> usize {
            debug_assert!(
                is_data_function_symbol(self.term.borrow()),
                "term {} is not a data function symbol",
                self.term
            );
            unsafe { ffi::get_data_function_symbol_index(self.term.borrow().get()) }
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
}

pub use inner::*;

#[cfg(test)]
mod tests {
    use crate::{aterm::{TermPool, ATerm, ATermTrait}, data::is_data_application};

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
        assert!(is_data_application(term.borrow()));
    }
}