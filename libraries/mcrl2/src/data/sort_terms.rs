use core::fmt;

use crate::aterm::{ATerm, ATermRef, ATermTrait, SymbolTrait};
use mcrl2_macros::mcrl2_derive_terms;
use mcrl2_sys::data::ffi;

pub fn is_sort_expression(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_sort_expression(term.get()) }
}

pub fn is_bool_sort(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    true
}

pub fn is_basic_sort(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_basic_sort(term.get()) }
}

pub fn is_data_function_sort(term: &ATermRef<'_>) -> bool {
    term.require_valid();
    unsafe { ffi::is_data_function_sort(term.get()) }
}

// This module is only used internally to run the proc macro.
#[mcrl2_derive_terms]
mod inner {
    use super::*;
    
    use std::ops::Deref;
    
    use mcrl2_macros::{mcrl2_ignore, mcrl2_term};
    use crate::{aterm::{ATermList, Markable, Todo}, data::DataExpression};


    #[mcrl2_term(is_bool_sort)]
    pub struct BoolSort {
        pub(crate) term: ATerm,
    }
    
    impl BoolSort {

        /// Returns the term representing true.
        pub fn true_term() -> DataExpression {
            DataExpression::from(ATerm::from(ffi::true_term()))
        }
    
        /// Returns the term representing false.
        pub fn false_term() -> DataExpression {
            DataExpression::from(ATerm::from(ffi::false_term()))
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
            is_basic_sort(&self.term)
        }
        
        /// Returns true iff this is a function sort
        pub fn is_function_sort(&self) -> bool {
            is_data_function_sort(&self.term)
        }
    }
    
    impl fmt::Display for SortExpression {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    #[mcrl2_term(is_basic_sort)]
    pub struct BasicSort {
        term: ATerm,
    }

    impl BasicSort {
        /// Returns the name of the sort.
        pub fn name(&self) -> String {
            String::from(self.arg(0).get_head_symbol().name())
        }
    }

    /// Derived from SortExpression
    #[mcrl2_term(is_data_function_sort)]
    pub struct FunctionSort {
        term: ATerm,
    }
    
    impl FunctionSort {
        /// Returns the name of the sort.
        pub fn domain(&self) -> ATermList<SortExpression> {
            ATermList::<SortExpression>::from(self.arg(0).protect())
        }

        /// Returns the name of the sort.
        pub fn codomain(&self) -> SortExpression {
            SortExpression::from(self.arg(1).protect())
        }
    }

    #[mcrl2_ignore]
    impl From<SortExpression> for FunctionSort {
        fn from(sort: SortExpression) -> Self {
            Self {
                term: sort.term,
            }
        }
    }
    
    #[mcrl2_ignore]
    impl From<SortExpressionRef<'_>> for FunctionSortRef<'_> {
        fn from(sort: SortExpressionRef<'_>) -> Self {
            unsafe {
                std::mem::transmute(sort)
            }
        }
    }
}

pub use inner::*;