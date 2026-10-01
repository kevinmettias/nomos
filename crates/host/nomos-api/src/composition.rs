//! What this crate was actually compiled as.
//!
//! A deliberate twin of `crates/host/nomos-cli/src/gate/composition.rs` -- `env!` resolves
//! against the crate that calls it, so this has to live once per composition root regardless
//! of which other root already reads the same four values.

use nomos_workspace::BuildVariant;

/// The build variant this crate was compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program.
pub(crate) fn Host_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES")
            .split(',')
            .filter(|feature| return !feature.is_empty()),
    );
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The four `env!` components `build.rs` captures are non-empty in a real build --
    /// proving this crate was actually compiled with them set, not linked against a stale
    /// or placeholder capture.
    #[test]
    fn Test_Host_Variant_Should_Carry_A_Real_Non_Empty_Target_Profile_And_Toolchain()
    {
        let variant = Host_Variant();

        assert!(!variant.target.is_empty(), "{variant:?}");
        assert!(!variant.profile.is_empty(), "{variant:?}");
        assert!(!variant.toolchain.is_empty(), "{variant:?}");
    }
}
