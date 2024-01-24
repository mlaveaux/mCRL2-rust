use abi_stable::{library::RootModule, package_version_strings, sabi_types::VersionStrings, StableAbi};


/// This struct is the root module, which must be converted to
/// `SabreCompiledRef` to be passed through ffi.
///
/// The `#[sabi(kind(Prefix(prefix_ref = SabreCompiledRef)))]` attribute tells
/// `StableAbi` to create an ffi-safe static reference type for `SabreCompiled`
/// called `SabreCompiledRef`.
///
/// The `#[sabi(missing_field(panic))]` attribute specifies that trying to
/// access a field that doesn't exist must panic with a message saying that the
/// field is inaccessible.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = SabreCompiledRef)))]
#[sabi(missing_field(panic))]
pub struct SabreCompiled {

    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the
    /// last field in this struct that was defined in the first compatible
    /// version of the library (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field
    /// until the library bumps its "major" version, at which point it would be
    /// moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    pub rewrite: extern "C" fn(),
}

/// The RootModule trait defines how to load the root module of a library.
impl RootModule for SabreCompiledRef {
    abi_stable::declare_root_module_statics! {SabreCompiledRef}

    const BASE_NAME: &'static str = "sabre_compiled";
    const NAME: &'static str = "sabre_compiled";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}