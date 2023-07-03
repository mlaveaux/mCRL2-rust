use core::fmt;

use cxx::UniquePtr;

use crate::atermpp::ATerm;

#[cxx::bridge(namespace = "mcrl2::data")]
mod ffi {

    unsafe extern "C++" {
        include!("mcrl2-rust/cpp/data/data.h");

        type data_specification;

        #[namespace = "mcrl2::data::detail"]
        type RewriterJitty;
        //#[namespace = "mcrl2::data::detail"]
        //type RewriterCompilingJitty;

        #[namespace = "atermpp"]
        type aterm = crate::atermpp::ffi::aterm;

        /// Parses the given text into a data specification.
        fn ffi_parse_data_specification(text: &str) -> Result<UniquePtr<data_specification>>;

        /// Parses the given text and typechecks it using the given data specification
        fn ffi_parse_data_expression(
            text: &str,
            data_spec: &data_specification,
        ) -> UniquePtr<aterm>;

        /// Creates an instance of the jitty rewriter.
        fn ffi_create_jitty_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>;

        /// Creates an instance of the compiling jitty rewriter.
        //fn ffi_create_jitty_compiling_rewriter(data_spec: &data_specification) -> UniquePtr<RewriterJitty>Compiling;

        /// Rewrites the given term to normal form.
        fn ffi_rewrite(rewriter: Pin<&mut RewriterJitty>, term: &aterm) -> UniquePtr<aterm>;        
        
        fn ffi_get_data_function_symbol_index(term: &aterm) -> usize;
    }
}

pub struct DataSpecification {
    pub data_spec: UniquePtr<ffi::data_specification>,
}

/// A pattern is simply an aterm of the shape f(...)
pub type DataExpression = ATerm;
pub type Variable = ATerm;

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Debug)]
pub struct DataEquation {
    pub variables: Vec<Variable>,
    pub condition: DataExpression,
    pub lhs: DataExpression,
    pub rhs: DataExpression,
}

impl DataSpecification {
    /// Parses the given text into a data specification
    pub fn new(text: &str) -> Self {
        DataSpecification {
            data_spec: ffi::ffi_parse_data_specification(text)
                .expect("failed to parse data specification"),
        }
    }

    /// Parses the given data expression as text into a term
    pub fn parse(&self, text: &str) -> ATerm {
        ATerm::from(ffi::ffi_parse_data_expression(text, &self.data_spec))
    }

    pub fn equations(&self) -> Vec<DataEquation> {
        vec![]
    }
}

pub struct JittyRewriter {
    rewriter: UniquePtr<ffi::RewriterJitty>,
}

impl JittyRewriter {
    pub fn new(spec: &DataSpecification) -> JittyRewriter {
        JittyRewriter {
            rewriter: ffi::ffi_create_jitty_rewriter(&spec.data_spec),
        }
    }

    pub fn rewrite(&mut self, term: &ATerm) -> ATerm {
        ATerm::from(ffi::ffi_rewrite(self.rewriter.pin_mut(), term.get()))
    }
}


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataVariable {
    pub(crate) term: ATerm,
}

impl DataVariable {
    pub fn name(&self) -> String {
        String::from(self.term.arg(0).get_head_symbol().name())
    }
}

impl From<ATerm> for DataVariable {
    fn from(value: ATerm) -> Self {
        assert!(value.is_data_variable(), "Term {value} is not a data variable");
        DataVariable { term: value }
    }
}

impl fmt::Display for DataVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataApplication {
    pub(crate) term: ATerm,
}

impl From<ATerm> for DataApplication {
    fn from(value: ATerm) -> Self {
        assert!(value.is_data_application(), "Term {value} is not a data application");
        DataApplication { term: value }
    }
}

impl fmt::Display for DataApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = self.term.arguments().into_iter();

        write!(f, "{}", <ATerm as Into<DataFunctionSymbol>>::into(args.next().unwrap()))?;

        let mut first = true;
        for arg in args {
            if !first {
                write!(f, ", ")?;
            } else {
                write!(f, "(")?;
            }

            write!(f, "{}", arg)?;
            first = false;
        }

        if !first {
            write!(f, ")")?;
        }

        Ok(())
    }
}

#[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataFunctionSymbol {
    pub(crate) term: ATerm,
}

impl DataFunctionSymbol
{
    pub fn name(&self) -> String {
        String::from(self.term.arg(0).get_head_symbol().name())
    }
    
    /// Returns the internal id known for every [aterm] that is a data::function_symbol.
    pub fn operation_id(&self) -> usize {
        assert!(self.term.is_data_function_symbol(), "term {} is not a data function symbol", self.term);
        ffi::ffi_get_data_function_symbol_index(&self.term.term)
    }
}

impl From<ATerm> for DataFunctionSymbol {
    fn from(value: ATerm) -> Self {
        assert!(value.is_data_function_symbol(), "Term {value} is not a data function symbol");
        DataFunctionSymbol { term: value }
    }
}

impl fmt::Display for DataFunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {        
        if !self.term.is_default() {
            write!(f, "{}", &self.name())
        } else {
            write!(f, "<default>")
        }
    }
}

/*
pub struct JittyCompilingRewriter
{
  rewriter: UniquePtr<ffi::RewriterJittyCompiling>
}
*/
