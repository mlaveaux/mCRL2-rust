use proc_macro2::TokenStream;

use quote::{quote, ToTokens, format_ident};
use syn::{ItemMod, Item, parse_quote};

pub(crate) fn mcrl2_derive_terms_impl(_attributes: TokenStream, input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let mut ast: ItemMod = syn::parse2(input.clone()).expect("mcrl2_term can only be applied to a module");

    if let Some((_, content)) = &mut ast.content {

        // Generated code blocks are added to this list.
        let mut added = vec![];

        for item in content.iter_mut() {
            match item {
                Item::Struct(object) => {
                    // If the struct is annotated with term we process it as a term.
                    if let Some(attr) = object.attrs.iter().find(|attr| {
                        attr.meta.path().is_ident("mcrl2_term")
                    }) {
                        // The #term(assertion) annotation must contain an assertion
                        let assertion = 
                            match attr.parse_args::<syn::Ident>()
                            {
                                Ok(assertion) => {
                                    let assertion_msg = format!("{assertion}");
                                    quote!(debug_assert!(#assertion(&term), "Term {:?} does not satisfy {}", term, #assertion_msg))
                                },
                                Err(_x) => {
                                    quote!()
                                }
                            };

                        // Add the expected derive macros to the input struct.
                        object.attrs.push(parse_quote!(#[derive(Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]));

                        // ALL structs in this module must contain the term.
                        assert!(object.fields.iter().any(|field| { 
                            if let Some(name) = &field.ident {
                                name == "term"
                            } else {
                                false
                            }
                        }), "The struct {} in mod {} has no field 'term: ATerm'", object.ident, ast.ident);

                        let name = format_ident!("{}", object.ident);
                        
                        // Add a <name>Ref struct that contains the ATermRef<'a> and
                        // the implementation and both protect and borrow. Also add
                        // the conversion from and to a ATerm.
                        let name_ref = format_ident!("{}Ref", object.ident);
                        let generated: TokenStream = quote!(
                            impl #name {
                                pub fn copy<'a>(&'a self) -> #name_ref<'a> {
                                    self.term.copy().into()
                                }
                            }

                            impl From<ATerm> for #name {
                                fn from(term: ATerm) -> #name {
                                    #assertion;
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

                            impl Deref for #name {                                
                                type Target = ATerm;

                                fn deref(&self) -> &Self::Target {
                                    &self.term        
                                }
                            }

                            #[derive(Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
                            pub struct #name_ref<'a> {
                                term: ATermRef<'a>
                            }

                            impl<'a> #name_ref<'a> {
                                pub fn copy<'b>(&'b self) -> #name_ref<'b> {
                                    self.term.copy().into()
                                }

                                pub fn protect(&self) -> #name {                                
                                    self.term.protect().into()
                                }
                            }

                            impl<'a> From<ATermRef<'a>> for #name_ref<'a> {
                                fn from(term: ATermRef<'a>) -> #name_ref<'a> {
                                    #assertion;
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

                            impl<'a> Deref for #name_ref<'a> {                                
                                type Target = ATermRef<'a>;

                                fn deref(&self) -> &Self::Target {
                                    &self.term        
                                }
                            }
                        );

                        added.push(Item::Verbatim(generated));
                    }
                }
                Item::Impl(_implementation) => {
                    // Duplicate the implementation for the ATermRef struct that is generated above.
                    // let mut ref_implementation = implementation.clone();
                    // println!("{:?}", ref_implementation);

                    // if let syn::Type::Path(path) = ref_implementation.self_ty.as_ref() {
                    //     let name_ref = format_ident!("{}Ref", path.path.get_ident().unwrap());
                    //     let path = Path::from(PathSegment::from(name_ref));

                    //     ref_implementation.self_ty = Box::new(syn::Type::Path(syn::TypePath { qself: None, path }));
    
                    //     added.push(Item::Verbatim(ref_implementation.into_token_stream()));
                    // }
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

                #[mcrl2_term(test)]
                #[derive(Debug)]
                struct Test {
                    term: ATerm,
                }

                impl Test<'a> {
                    fn a_function() {

                    }
                }
            }
        ";
        
        let tokens = TokenStream::from_str(input).unwrap();
        let result = mcrl2_derive_terms_impl(TokenStream::default(), tokens);

        println!("{result}");
    }
}