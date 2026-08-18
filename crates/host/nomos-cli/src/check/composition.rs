//! What this binary was actually compiled as.
//!
//! The capability registry composition that used to live here (`Registered`,
//! `Resolved_Configuration`) moved to `nomos-check-orchestration` — `OD-HOST-002`. This one
//! function stayed: `env!` resolves against the crate that calls it, so a build variant read
//! from inside the orchestration crate would describe that library's own compilation rather
//! than this binary's, the same reason `tests/integration/build.rs` is a deliberate twin of
//! this crate's `build.rs` rather than a shared dependency of it.

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
