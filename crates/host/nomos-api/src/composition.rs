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
