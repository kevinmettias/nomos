//! One step of the workflow, as the reader derived it.

use super::{GateUnknown, StepGuard};

/// A step's name, the guard it declares, and the argv its `run:` line yields.
///
/// The three travel together because a local execution needs all three to give the step one
/// of `OD-GATE-033`'s values, and because the defect that record names is exactly what
/// happens when the guard is dropped on the way: `Derive_Step` answers for one named step
/// and discards every other key, so an executor built on it alone would treat a
/// Windows-only step as an ordinary one.
///
/// `argv` is a `Result` rather than an `Option` because a step with no derivable command is
/// not the same as a step with no command, and the difference is the cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedStep
{
    /// The step's name, exactly as the workflow writes it.
    pub name: String,
    /// What the step's `if:` key says about which hosts run it.
    pub guard: StepGuard,
    /// The argv the step's `run:` line yields, or why none could be derived.
    pub argv: Result<Vec<String>, GateUnknown>,
}
