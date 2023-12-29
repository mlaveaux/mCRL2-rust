use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;


#[proc_macro_derive(Term)]
pub fn derive_from_term(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_from_term_impl(TokenStream::from(input)).into()
}

fn derive_from_term_impl(input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let ast: ItemStruct = syn::parse2(input).unwrap();

    // Every term has a borrow and protect function.
    let name: String = ast.ident.to_string();
    let expanded = quote! {
        impl From<ATerm> for #name {            
            fn from(term: ATerm) -> Self {
                Self {
                    term
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn derive_test() {
        let input = "
            struct Test {}
        ";
        
        let tokens = TokenStream::from_str(input).unwrap();
        let result = derive_from_term_impl(tokens);

        assert_eq!(format!("{}", result), "");
    }
}