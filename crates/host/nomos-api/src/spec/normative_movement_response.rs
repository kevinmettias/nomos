//! [`NormativeMovementResponse`], shared by [`super::preview_response::PreviewResponse::Previewed`]
//! and [`super::committed_preview_response::CommittedPreviewResponse`].

use nomos_spec_store::NormativeMovement;
use serde::Serialize;

use super::NormativeOutcomeResponse;

/// A serializable twin of [`nomos_spec_store::NormativeMovement`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct NormativeMovementResponse
{
    pub statement_id: String,
    pub canonical_hash: String,
    pub outcome: NormativeOutcomeResponse,
}

impl NormativeMovementResponse
{
    pub(crate) fn From(movement: NormativeMovement) -> Self
    {
        return Self {
            statement_id: movement.statement_id,
            canonical_hash: movement.canonical_hash,
            outcome: NormativeOutcomeResponse::From(movement.outcome),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::NormativeOutcome;

    /// The block the fixture's statement is held in -- read at the assertion too, so a `From`
    /// that dropped the nested outcome's own block would fail rather than pass on the tag.
    const HELD_BLOCK_ORDINAL: u32 = 3;

    #[test]
    fn Test_From_Should_Copy_Every_Field_And_Map_The_Nested_Outcome()
    {
        let movement = NormativeMovement {
            statement_id: "S-1".to_owned(),
            canonical_hash: "deadbeef".to_owned(),
            outcome: NormativeOutcome::Held { block: HELD_BLOCK_ORDINAL },
        };

        let response = NormativeMovementResponse::From(movement.clone());

        assert_eq!(response.statement_id, movement.statement_id);
        assert_eq!(response.canonical_hash, movement.canonical_hash);
        assert!(matches!(response.outcome, NormativeOutcomeResponse::Held { block: HELD_BLOCK_ORDINAL }));
    }
}
