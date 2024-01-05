use mcrl2_macros::{mcrl2_derive_terms, mcrl2_term};

// This is boilerplate since the macro crate cannot depend on the other crate.
fn main() {

}

use std::marker::PhantomData;

#[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ATerm {}

impl ATerm {
    #[allow(unused)]
    pub fn borrow(&self) -> ATermRef<'_> {
        Default::default()
    }
    
}

#[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ATermRef<'a> {
    marker: PhantomData<&'a ()>
}

impl<'a> ATermRef<'a> {

    #[allow(unused)]
    pub fn protect(&self) -> ATerm {
        Default::default()
    }
}

#[mcrl2_derive_terms]
mod term {
    use super::*;

    #[mcrl2_term()]
    struct BoolSort {
        term: ATerm
    }

    impl BoolSort {
    
        #[allow(unused)]
        pub fn test(&self) -> bool {
            true
        }
    }
}
