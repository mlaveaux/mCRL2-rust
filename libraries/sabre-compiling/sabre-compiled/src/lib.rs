//!
//! The 
//! 

use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn};

use sabre_compiling::{SabreCompiled, SabreCompiledRef};

mod generated;

/// The function which exports the root module of the library.
///
/// The root module is exported inside a static of `LibHeader` type, which has
/// this extra metadata:
///
/// - The abi_stable version number used by the dynamic library.
///
/// - A constant describing the layout of the exported root module, and every
///   type it references.
///
/// - A lazily initialized reference to the root module.
///
/// - The constructor function of the root module.
///
#[export_root_module]
pub fn get_library() -> SabreCompiledRef {
    SabreCompiled {
        rewrite
    }
    .leak_into_prefix()
}


/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn rewrite() {
    generated::rewrite_term();
}