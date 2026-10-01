//! What this binary was actually compiled as.
//!
//! A deliberate twin of `check::composition::Host_Variant`, not a shared dependency of it --
//! `env!` resolves against the crate that calls it, so this has to live once per composition
//! root regardless of which other root already reads the same four values, the same
//! "additive and unwired" separation every verb group in this binary keeps from every other.

use super::BuildVariant;

/// The build variant this binary was compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program.
pub(super) fn Host_Variant() -> BuildVariant
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
