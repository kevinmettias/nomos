//! [`NormativeOutcomeResponse`], carried only by [`super::normative_movement_response::
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The block the held statement stays in. Distinct from the moved and gone ordinals
    /// below, so a `From` that crossed two variants' blocks would not still match.
    const HELD_BLOCK_ORDINAL: u32 = 3;
    /// Where the moved statement lands. Its origin is `1`, which stays literal as the
    /// identity end of the move rather than a chosen position.
    const MOVED_TO_BLOCK_ORDINAL: u32 = 2;
    /// The block the gone statement was recorded against.
    const GONE_FROM_BLOCK_ORDINAL: u32 = 4;

    #[test]
    fn Test_From_Should_Map_Every_Domain_Variant_To_Its_Own_Response_Variant()
    {
        assert!(matches!(
            NormativeOutcomeResponse::From(NormativeOutcome::Held { block: HELD_BLOCK_ORDINAL }),
            NormativeOutcomeResponse::Held { block: HELD_BLOCK_ORDINAL }
        ));
        assert!(matches!(
            NormativeOutcomeResponse::From(NormativeOutcome::Moved { from: 1, to: MOVED_TO_BLOCK_ORDINAL }),
            NormativeOutcomeResponse::Moved { from: 1, to: MOVED_TO_BLOCK_ORDINAL }
        ));
        assert!(matches!(
            NormativeOutcomeResponse::From(NormativeOutcome::Gone { from: GONE_FROM_BLOCK_ORDINAL }),
            NormativeOutcomeResponse::Gone { from: GONE_FROM_BLOCK_ORDINAL }
        ));
        assert!(matches!(
            NormativeOutcomeResponse::From(NormativeOutcome::Unlocatable),
            NormativeOutcomeResponse::Unlocatable
        ));
    }
}
