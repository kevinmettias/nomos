//! [`NormativeMovementResponse`], shared by [`super::spec_preview_response::SpecPreviewResponse::Previewed`]
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
