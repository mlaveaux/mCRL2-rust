use core::fmt;

use crate::aterm::{ATerm, ATermList, ATermRef, ATermTrait, SymbolTrait};
use mcrl2_sys::{atermpp, cxx::UniquePtr, data::ffi};

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
            condition: value.arg(1).protect(),
            lhs: value.arg(2).protect(),
            rhs: value.arg(3).protect(),
        }
    }
}

impl DataSpecification {
    /// Parses the given text into a data specification
    pub fn new(text: &str) -> Result<Self, Box<dyn Error>> {
        let data_spec = ffi::parse_data_specification(text)?;

        Ok(DataSpecification {
            data_spec,
        })
    }

    /// Parses the given data expression as text into a term
    pub fn parse(&self, text: &str) -> ATerm {
        ffi::parse_data_expression(text, &self.data_spec).into()
    }

    /// Returns the equations of the data specification.
    pub fn equations(&self) -> Vec<DataEquation> {
        ffi::get_data_specification_equations(&self.data_spec)
            .iter()
            .map(|x| ATerm::from(x).into())
            .collect()
    }
}

impl Clone for DataSpecification {
    fn clone(&self) -> Self {
        DataSpecification {
            data_spec: ffi::data_specification_clone(&self.data_spec),
        }
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
            atermpp::ffi::enable_automatic_garbage_collection(true);
            let result = ffi::rewrite(self.rewriter.pin_mut(), term.borrow().term).into();
            atermpp::ffi::enable_automatic_garbage_collection(false);
            result
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
        debug_assert!(
            value.is_data_variable(),
            "Term {value} is not a data variable"
        );
        DataVariable { term: value }
    }
}

impl<'a> From<ATermRef<'a>> for DataVariable {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(
            value.is_data_variable(),
            "Term {value} is not a data variable"
        );
        DataVariable {
            term: value.protect(),
        }
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
            write!(f, "{}", <ATermRef as Into<DataFunctionSymbolRef>>::into(head))?;
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

/// The data::function_symbol
#[derive(Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataFunctionSymbolRef<'a> {
    pub(crate) term: ATermRef<'a>,
}

impl<'a> DataFunctionSymbolRef<'a> {
    pub fn name(&self) -> String {
        String::from(self.term.arg(0).get_head_symbol().name())
    }

    pub fn borrow(&self) -> ATermRef<'_> {
        self.term.borrow()
    }

    pub fn protect(&self) -> DataFunctionSymbol {
        DataFunctionSymbol {
            term: self.term.protect()
        }
    }

    /// Returns the internal id known for every [aterm] that is a data::function_symbol.
    pub fn operation_id(&self) -> usize {
        debug_assert!(
            self.term.is_data_function_symbol(),
            "term {} is not a data function symbol",
            self.term
        );
        unsafe { ffi::get_data_function_symbol_index(self.term.borrow().term) }
    }
}

impl<'a> Into<ATermRef<'a>> for DataFunctionSymbolRef<'a> {
    fn into(self) -> ATermRef<'a> {
        self.term
    }
}

impl<'a> From<ATermRef<'a>> for DataFunctionSymbolRef<'a> {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(
            value.is_data_function_symbol(),
            "Term {value:?} is not a data function symbol"
        );
        DataFunctionSymbolRef { term: value }
    }
}

impl<'a> fmt::Display for DataFunctionSymbolRef<'a> {
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
            term: ffi::true_term().into(),
        }
    }

    pub fn false_term() -> BoolSort {
        BoolSort {
            term: ffi::false_term().into(),
        }
    }
}

impl From<ATerm> for BoolSort {
    fn from(value: ATerm) -> Self {
        BoolSort { term: value }
    }
}

#[derive(Clone, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataFunctionSymbol {
    pub(crate) term: ATerm,
}

impl DataFunctionSymbol {
    pub fn borrow(&self) -> DataFunctionSymbolRef<'_> {
        DataFunctionSymbolRef {
            term: self.term.borrow()
        }
    }

    pub fn operation_id(&self) -> usize {
        self.borrow().operation_id()
    }
}

impl fmt::Display for DataFunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.borrow())        
    }
}

impl<'a> From<ATerm> for DataFunctionSymbol {
    fn from(value: ATerm) -> Self {
        debug_assert!(
            value.is_data_function_symbol(),
            "Term {value:?} is not a data function symbol"
        );
        DataFunctionSymbol { term: value }
    }
}

impl Into<ATerm> for DataVariable {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for DataApplication {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for DataFunctionSymbol {
    fn into(self) -> ATerm {
        self.term        
    }
}

impl Into<ATerm> for BoolSort {
    fn into(self) -> ATerm {
        self.term        
    }
}

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

        let _data_spec = DataSpecification::new(text).unwrap();
    }
}
