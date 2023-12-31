mod mcrl2_term;

use mcrl2_term::mcrl2_term_impl;

/// This proc macro can be used to generate implementations for the types stored
/// in an ATerm, for example data_expressions, applications, variables. This is
/// achieved by adding the proc macro to a module that contains both the
/// declaration and implementation of such a type.
/// 
/// For every struct containing an ATerm we generate another version for the
/// ATermRef implementation, as well as `protect` and `borrow` functions to
/// convert between both types. Furthermore, all of these can be converted to
/// and from ATerms.
/// 
/// # Example
#[proc_macro_attribute]
pub fn mcrl2_term(_attributes: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    mcrl2_term_impl(proc_macro2::TokenStream::from(_attributes), proc_macro2::TokenStream::from(input)).into()
}

#[proc_macro_attribute]
pub fn term(_attributes: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}