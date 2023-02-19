use std::fmt;

use cxx::UniquePtr;

use crate::atermpp::{ATerm, ffi::aterm};

#[cxx::bridge(namespace = "mcrl2::data")]
mod ffi {

  unsafe extern "C++" {    
    include!("mcrl2-rust/cpp/data/data.h");

    type data_specification;

    #[namespace = "mcrl2::data::detail"]
    type RewriterJitty;
    
    #[namespace = "atermpp"]
    type aterm = crate::atermpp::ffi::aterm;

    /// Parses the given text into a data specification.
    fn ffi_parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;

    /// Parses the given text and typechecks it using the given data specification
    fn ffi_parse_data_expression(text: &str, data_spec: &data_specification) -> UniquePtr<aterm>;

    /// Creates an instance of the jitty rewriter.
    fn ffi_create_jitty_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

    /// Rewrites the given term to normal form.
    fn ffi_rewrite(rewriter: Pin<&mut RewriterJitty>, term: &aterm) -> UniquePtr<aterm>;
  }

}

pub struct DataSpecification
{
  pub data_spec: UniquePtr<ffi::data_specification>,
}

impl DataSpecification
{
  /// Parses the given text into a data specification
  pub fn from(text: &str) -> Self
  {
    DataSpecification { data_spec: ffi::ffi_parse_data_specification(text).expect("failed to parse data specification") }
  }

  /// Parses the given data expression as text into a term
  pub fn parse(&self, text: &str) -> ATerm
  {
    ATerm::from(ffi::ffi_parse_data_expression(text, &self.data_spec))
  }
}

pub struct JittyRewriter
{
  rewriter: UniquePtr<ffi::RewriterJitty>,
}

impl JittyRewriter
{
  pub fn new(spec: &DataSpecification) -> JittyRewriter
  {
    JittyRewriter {
      rewriter: ffi::ffi_create_jitty_rewriter(&spec.data_spec),
    }
  }

  pub fn rewrite(&mut self, term: &ATerm) -> ATerm
  {
    ATerm::from(ffi::ffi_rewrite(self.rewriter.pin_mut(), term.get()))
  }
}
