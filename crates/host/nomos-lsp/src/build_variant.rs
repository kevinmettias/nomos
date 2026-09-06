//! What this binary was actually compiled as.
//!
//! A deliberate twin of `nomos-cli::check::composition::Host_Variant`: `env!` resolves
//! against the crate that calls it, so a build variant read from inside
//! `nomos-check-orchestration` would describe that library's own compilation rather than
//! this binary's. Each real caller of `nomos_check_orchestration::Run` carries this one
//! function for itself; see this crate's own `build.rs` for why the value cannot be shared.

use nomos_workspace::BuildVariant;

/// The build variant this binary was compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program.
pub(crate) fn Host_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every component `build.rs` captures from cargo's own environment survives into the
    /// compiled program non-empty -- `target`, `profile` and `toolchain` are read straight
    /// off `env!`, which fails the build at compile time were any of them absent, so a
    /// runner that cannot supply them never reaches this test at all.
    #[test]
    fn Test_Host_Variant_Should_Read_Every_Non_Feature_Component_Baked_In_By_The_Build_Script()
    {
        let variant = Host_Variant();

        assert!(!variant.target.is_empty());
        assert!(!variant.profile.is_empty());
        assert!(!variant.toolchain.is_empty());
    }
}
