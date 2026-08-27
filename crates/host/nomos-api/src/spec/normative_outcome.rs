//! [`NormativeOutcomeResponse`], carried only by [`super::normative_movement::
//! NormativeMovementResponse`].

use nomos_spec_store::NormativeOutcome;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::NormativeOutcome`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NormativeOutcomeResponse
{
    /// Still in the block it was in.
    Held
    {
        block: u32
    },
    /// Still present, in a different block.
    Moved
    {
        from: u32, to: u32
    },
    /// Was in the record and is not in the staged text.
    Gone
    {
        from: u32
    },
    /// Recorded against the record and not found in it before the edit either.
    Unlocatable,
}

impl NormativeOutcomeResponse
{
    pub(crate) fn From(outcome: NormativeOutcome) -> Self
    {
        return match outcome
        {
            NormativeOutcome::Held { block } => Self::Held { block },
            NormativeOutcome::Moved { from, to } => Self::Moved { from, to },
            NormativeOutcome::Gone { from } => Self::Gone { from },
            NormativeOutcome::Unlocatable => Self::Unlocatable,
        };
    }
}
