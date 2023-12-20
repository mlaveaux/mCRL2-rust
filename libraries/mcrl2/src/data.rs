use core::fmt;

use mcrl2_sys::{data::ffi, cxx::UniquePtr};
use crate::{aterm::{ATerm, ATermList, ATermRef, ATermTrait}, symbol::SymbolTrait};

pub struct DataSpecification {
    pub data_spec: UniquePtr<ffi::data_specification>,
}

/// A pattern is simply an aterm of the shape f(...)
pub type DataExpression = ATerm;

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Debug)]
pub struct DataEquation {
    pub variables: Vec<DataVariable>,
    pub condition: DataExpression,
    pub lhs: DataExpression,
    pub rhs: DataExpression,
}

impl From<ATerm> for DataEquation {
    fn from(value: ATerm) -> Self {
        let variables: ATermList<DataVariable> = value.arg(0).into();

        DataEquation { 
            variables: variables.iter().collect(), 
            condition: value.arg(1).into(), 
            lhs: value.arg(2).into(), 
            rhs: value.arg(3).into() }
    }
}

impl DataSpecification {
    /// Parses the given text into a data specification
    pub fn new(text: &str) -> Self {
        DataSpecification {
            data_spec: ffi::parse_data_specification(text)
                .expect("failed to parse data specification"),
        }
    }

    /// Parses the given data expression as text into a term
    pub fn parse(&self, text: &str) -> ATerm {
        ffi::parse_data_expression(text, &self.data_spec).into()
    }

    /// Returns the equations of the data specification.
    pub fn equations(&self) -> Vec<DataEquation> {
        ffi::get_data_specification_equations(&self.data_spec).iter().map(|x| {
            ATerm::from(x).into()
        }).collect()
    }
}

impl Clone for DataSpecification {
    fn clone(&self) -> Self {
        DataSpecification { data_spec: ffi::data_specification_clone(&self.data_spec) }
    }
}

pub struct JittyRewriter {
    rewriter: UniquePtr<ffi::RewriterJitty>,
}

impl JittyRewriter {

    /// Create a rewriter instance from the given data specification.
    pub fn new(spec: &DataSpecification) -> JittyRewriter {
        JittyRewriter {
            rewriter: ffi::create_jitty_rewriter(&spec.data_spec),
        }
    }

    /// Rewrites the term with the jitty rewriter.
    pub fn rewrite(&mut self, term: &ATerm) -> ATerm {
        unsafe {
          ffi::rewrite(self.rewriter.pin_mut(), term.borrow().term).into()
        }
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
        debug_assert!(value.is_data_variable(), "Term {value} is not a data variable");
        DataVariable { term: value }
    }
}

impl<'a> From<ATermRef<'a>> for DataVariable {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(value.is_data_variable(), "Term {value} is not a data variable");
        DataVariable { term: value.protect() }
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
        //debug_assert!(value.is_data_application(), "Term {value} is not a data application");
        DataApplication { term: value }
    }
}

impl fmt::Display for DataApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = self.term.arguments();

        let head = args.next().unwrap();
        if head.is_data_function_symbol() {
            write!(f, "{}", <ATerm as Into<DataFunctionSymbol>>::into(head))?;
        } else {
            write!(f, "{:?}", head)?;
        }


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
        debug_assert!(self.term.is_data_function_symbol(), "term {} is not a data function symbol", self.term);
        unsafe {
            ffi::get_data_function_symbol_index(self.term.borrow().term)
        }
    }
}

impl From<ATerm> for DataFunctionSymbol {
    fn from(value: ATerm) -> Self {
        debug_assert!(value.is_data_function_symbol(), "Term {value:?} is not a data function symbol");
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

pub struct BoolSort {
    pub(crate) term: ATerm,
}

impl BoolSort {
    pub fn true_term() -> BoolSort {
        BoolSort { 
            term: ffi::true_term().into()
        }
    }

    pub fn false_term() -> BoolSort {
        BoolSort { 
            term: ffi::false_term().into()
        }
    }
}

impl From<ATerm> for BoolSort {
    fn from(value: ATerm) -> Self {
        BoolSort { term: value }
    }
}

/*pub struct JittyCompilingRewriter
{
  rewriter: UniquePtr<ffi::RewriterJittyCompiling>
}*/


#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_parse_data_specification() {

        let text = "
            sort Xbool = struct
                Xfalse
            | Xtrue ;
            
            sort Bit = struct
                x0
            | x1 ;
            
            sort Octet = struct
                buildOctet (Bit, Bit, Bit, Bit, Bit, Bit, Bit, Bit) ;
            
            sort OctetSum = struct
                buildOctetSum (Bit, Octet) ;
            
            sort Half = struct
                buildHalf (Octet, Octet) ;
            
            sort HalfSum = struct
                buildHalfSum (Bit, Half) ;
            
            map
                notBool : Xbool -> Xbool ;
                andBool : Xbool # Xbool -> Xbool ;";

        let _data_spec = DataSpecification::new(text);

    }

}