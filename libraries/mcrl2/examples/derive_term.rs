use mcrl2::aterm::ATerm;
use mcrl2::aterm::ATermRef;
use mcrl2::aterm::Markable;
use mcrl2::aterm::Todo;
use mcrl2_macros::mcrl2_derive_terms;
use mcrl2_macros::mcrl2_term;

fn main() {}

#[mcrl2_derive_terms]
mod term {
    use super::*;
    use std::borrow::Borrow;
    use std::ops::Deref;

    #[mcrl2_term()]
    struct BoolSort {
        term: ATerm,
    }

    impl BoolSort {
        #[allow(unused)]
        pub fn test(&self) -> bool {
            true
        }
    }
}
