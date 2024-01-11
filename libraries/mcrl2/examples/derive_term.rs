use mcrl2_macros::{mcrl2_derive_terms, mcrl2_term};
use mcrl2::aterm::{ATerm, ATermRef, ATermTrait};
use mcrl2::aterm::{Markable, Todo};

// This is boilerplate since the macro crate cannot depend on the other crate.
fn main() {

}

#[mcrl2_derive_terms]
mod term {
    use super::*;
    use std::ops::Deref;

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
