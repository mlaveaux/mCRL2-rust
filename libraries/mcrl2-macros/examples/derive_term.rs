use mcrl2_macros::mcrl2_term;

// This is boilerplate since the macro crate cannot depend on the other crate.
fn main() {

}

use std::marker::PhantomData;

#[derive(Default)]
struct ATerm {}

impl ATerm {
    pub fn borrow(&self) -> ATermRef<'_> {
        Default::default()
    }
    
}

#[derive(Default)]
struct ATermRef<'a> {
    marker: PhantomData<&'a ()>
}

impl<'a> ATermRef<'a> {
    pub fn protect(&self) -> ATerm {
        Default::default()
    }
}

#[mcrl2_term]
mod term {
    use super::*;

    struct BoolSort {
        term: ATerm
    }

    impl BoolSort {
    
        pub fn test(&self) -> bool {
            true
        }
    }
}
