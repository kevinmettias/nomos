//! What one step of a local gate execution produced.

use super::GateUnknown;

/// The three values `OD-GATE-033` decided a local execution gives every step.
///
/// The forbidden fourth value is a subset reported as clean, which is why there is no
/// `Skipped`: a step this host did not run is [`Self::Unavailable`], naming the host, and a
/// step the reader could not understand is [`Self::Refused`], carrying the cause. Neither
/// is a pass, and neither is silent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepExecution
{
    /// The guard admitted this host and the step's argv derived, so it ran here and this
    /// is what it exited with.
    Executed
    {
        /// The exit code the step really exited with.
        code: i32,
    },
    /// The guard does not admit this host, so this execution produced no evidence about
    /// the step. Reported as unavailable, never as a pass.
    Unavailable
    {
        /// The host this execution ran on, which the step's guard excluded.
        host: String,
    },
    /// The guard or the `run:` body is outside the subset this workspace implements, so no
    /// answer could be derived without guessing.
    Refused(GateUnknown),
}
