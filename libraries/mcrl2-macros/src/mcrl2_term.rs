use proc_macro2::TokenStream;

use quote::{quote, ToTokens, format_ident};
use syn::{ItemMod, Item};

pub(crate) fn mcrl2_term_impl(_attributes: TokenStream, input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let mut ast: ItemMod = syn::parse2(input.clone()).expect("mcrl2_term can only be applied to a module");

    if let Some((_, content)) = &mut ast.content {

        // Added code blocks are added to this list.
        let mut added = vec![];

        // We keep track of term structs since their implementation blocks must be adapted.
        let mut objects = vec![];

        for item in content.iter() {
            match item {
                Item::Struct(object) => {
                    // If the struct is annotated with term we process it as a term.
                    if object.attrs.iter().find(|attr| {
                        attr.meta.path().is_ident("term")
                    }).is_some() {
                        // The #term(assertion) annotation must contain an assertion
                        //let args: Path = term.parse_args().expect("Required");

                        // TODO: Use the term assertion, and derive visibility from the struct.

                        // ALL structs in this module must contain the term.
                        assert!(object.fields.iter().any(|field| { 
                            if let Some(name) = &field.ident {
                                name == "term"
                            } else {
                                false
                            }
                        }), "The struct {} in mod {} has no field 'term: ATerm'", object.ident, ast.ident);

                        let name = format_ident!("{}", object.ident);
                        objects.push(name.clone());

                        // Add a <name>Ref struct that contains the ATermRef<'a> and
                        // the implementation and both protect and borrow. Also add
                        // the conversion from and to a ATerm.
                        let name_ref = format_ident!("{}Ref", object.ident);
                        let generated: TokenStream = quote!(
                            impl #name {
                                pub fn borrow(&self) -> #name_ref<'_> {
                                    self.term.borrow().into()
                                }
                            }

                            impl From<ATerm> for #name {
                                fn from(term: ATerm) -> #name {
                                    // Add assertion
                                    #name {
                                        term
                                    }
                                }
                            }   

                            impl Into<ATerm> for #name {
                                fn into(self) -> ATerm {
                                    self.term
                                }
                            }

                            #[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
                            pub struct #name_ref<'a> {
                                term: ATermRef<'a>
                            }

                            impl<'a> #name_ref<'a> {
                                pub fn protect(&self) -> #name {                                
                                    self.term.protect().into()
                                }
                            }

                            impl<'a> From<ATermRef<'a>> for #name_ref<'a> {
                                fn from(term: ATermRef<'a>) -> #name_ref<'a> {
                                    // Add assertion
                                    #name_ref {
                                        term
                                    }
                                }
                            }

                            impl<'a> Into<ATermRef<'a>> for #name_ref<'a> {
                                fn into(self) -> ATermRef<'a> {
                                    self.term
                                }
                            }
                        );

                        added.push(Item::Verbatim(generated));
                    }
                }
                Item::Impl(implementation) => {


                }
                _ => {
                    // Ignore the rest.
                }
            }
        }


        content.append(&mut added);
    }

    // Hand the output tokens back to the compiler
    ast.into_token_stream()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_macro() {
        let input = "
            mod anything {

                #[term(test)]
                struct Test {
                    term: ATerm,
                }
            }
        ";
        
        let tokens = TokenStream::from_str(input).unwrap();
        let result = mcrl2_term_impl(TokenStream::default(), tokens);

        //assert_eq!(format!("{}", result), "");
    }
}