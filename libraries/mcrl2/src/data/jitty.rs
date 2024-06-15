use mcrl2_sys::atermpp;
use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::data::ffi;

use crate::aterm::ATerm;
use crate::data::DataSpecification;

use super::DataExpression;

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
    pub fn rewrite(&mut self, term: DataExpression) -> DataExpression {
        unsafe {
            atermpp::ffi::enable_automatic_garbage_collection(true);
            let result: ATerm = ffi::rewrite(self.rewriter.pin_mut(), term.get()).into();
            atermpp::ffi::enable_automatic_garbage_collection(false);
            result.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::DataExpression;

    use super::*;

    #[test]
    fn test_jitty() {
        let spec = DataSpecification::new(&include_str!(
            "../../../../examples/REC/mcrl2/revelt.dataspec"
        ))
        .unwrap();
        let terms: Vec<DataExpression> =
            include_str!("../../../../examples/REC/mcrl2/revelt.expressions")
                .lines()
                .map(|text| spec.parse(text).unwrap())
                .collect();

        // let mut sa = SabreRewriter::new(tp.clone(), &spec.clone().into());
        let mut inner = JittyRewriter::new(&spec);
        let mut expected =
            include_str!("../../../sabre/tests/snapshot/result_revelt.txt").split('\n');

        for term in &terms {
            let expected_result = spec.parse(expected.next().unwrap()).unwrap();

            let result = inner.rewrite(term.clone());
            assert_eq!(
                result,
                expected_result.into(),
                "The inner rewrite result doesn't match the expected result"
            );
        }
    }
}
