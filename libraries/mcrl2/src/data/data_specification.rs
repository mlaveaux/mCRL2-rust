use std::error::Error;

use mcrl2_sys::{cxx::{UniquePtr, self}, data::ffi};
use utilities::lock_global;

use crate::aterm::{ATerm, ATermList, ATermRef};

use super::{DataVariable, DataExpression, DataFunctionSymbol, SortExpressionRef};

/// A safe abstraction for the mCRL2 data specification.
pub struct DataSpecification {
    pub(crate) data_spec: UniquePtr<ffi::data_specification>,
}

impl DataSpecification {
    /// Parses the given text into a data specification
    pub fn new(text: &str) -> Result<Self, cxx::Exception> {
        let _guard = lock_global();
        let data_spec = ffi::parse_data_specification(text)?;

        Ok(DataSpecification {
            data_spec,
        })
    }

    /// Parses the given text as a data expression for the spec.
    pub fn parse(&self, text: &str) -> Result<DataExpression, Box<dyn Error>> {
        let _guard = lock_global();
        let term: ATerm = ffi::parse_data_expression(text, &self.data_spec)?.into();

        Ok(term.into())
    }

    /// Parses the given text as a data variable for the spec.
    pub fn parse_variable(&self, text: &str) -> Result<DataVariable, Box<dyn Error>> {
        let _guard = lock_global();
        let term: ATerm = ffi::parse_variable(text, &self.data_spec)?.into();
        Ok(term.into())
    }

    /// Returns the equations of the data specification.
    pub fn equations(&self) -> Vec<DataEquation> {
        ffi::get_data_specification_equations(&self.data_spec)
            .iter()
            .map(|x| ATerm::from(x).into())
            .collect()
    }

    /// Returns the constructors for the given sort expression.
    pub fn constructors(&self, sort: &SortExpressionRef<'_>) -> Vec<DataFunctionSymbol> {
        let t: ATermRef<'_> = sort.copy().into();
        unsafe {
            ffi::get_data_specification_constructors(&self.data_spec, t.get())
            .iter()
            .map(|x| ATerm::from(x).into())
            .collect()
        }
    }
}

impl Clone for DataSpecification {
    fn clone(&self) -> Self {
        DataSpecification {
            data_spec: ffi::data_specification_clone(&self.data_spec),
        }
    }
}

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
            condition: value.arg(1).protect().into(),
            lhs: value.arg(2).protect().into(),
            rhs: value.arg(3).protect().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

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
