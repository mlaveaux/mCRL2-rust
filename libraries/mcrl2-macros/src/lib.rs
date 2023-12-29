use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// For objects that are stored inside terms we provide the following
/// convenience macro.
/// 
/// The shape of the struct should be
/// 
/// struct BoolSortRef {}
#[proc_macro_attribute]
pub fn mcrl2_term(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    //let ast = parse_macro_input!(input as DeriveInput);

    // Every term has a borrow and protect function.
    // let name = &ast.ident;
    // let expanded = quote! {
    //     impl Into<ATermRef<'a>> for #name {            
    //         fn into(self) -> ATermRef<'a> {
    //             self.term
    //         }
    //     }
    // };

    // Hand the output tokens back to the compiler
    input
}
