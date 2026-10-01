//! A `Platform` paired with the clock a bounded step's elapsed time is measured through.

use nomos_platform::{Clock, Environment, FileSystem, ProgramLauncher};

use super::Platform;

/// [`Platform`] paired with the [`Clock`] a step declaring
/// `nomos_contracts::Timeout::Seconds` has its elapsed time measured through.
///
/// A pair rather than a fifth field on [`Platform`] itself, and a pair rather than a fifth
/// parameter on [`super::Run_With_Clock`]. `Platform` is built by struct literal in
/// `nomos_cli::workflow` and `nomos_api::workflow`, naming every field, so a new field
/// there is a breaking change to two crates
/// `P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` may not edit; and this workspace's
/// own `parameter-count` rule caps a function at four value parameters, which
/// [`super::Run`] already sits at. Grouping is the answer `Platform` itself already gives
/// to the identical question, applied once more.
///
/// `Platform::now` is not this clock. That field is one fixed moment -- the instant a run
/// judges a waiver's expiry against -- and an elapsed time cannot be taken from one
/// instant. This is the source of successive readings, and it is optional for the same
/// reason `Platform::now` is supplied rather than read: a caller that has no clock to
/// give calls [`super::Run_Unclocked`] and is told its bounded steps went unmeasured,
/// rather than having this crate read an ambient one.
pub struct ClockedPlatform<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment, Clk: Clock>
{
    /// The real platform this run's steps dispatch through.
    pub platform: &'a Platform<'a, Launcher, Fs, Env>,
    /// Where each bounded step's start and end are read from.
    pub clock: &'a Clk,
}
