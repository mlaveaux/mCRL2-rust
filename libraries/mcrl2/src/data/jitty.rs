use mcrl2_sys::{cxx::UniquePtr, data::ffi, atermpp};

use crate::aterm::ATerm;
use crate::data::DataSpecification;

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
            let result = ffi::rewrite(self.rewriter.pin_mut(), term.get()).into();
            atermpp::ffi::enable_automatic_garbage_collection(false);
            result
        }
    }
}