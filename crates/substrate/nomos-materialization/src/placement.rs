//! What a completed run did to one target, and under which two classifications.

use crate::ownership_class::OwnershipClass;
use crate::placement_outcome::PlacementOutcome;
use crate::publication_scope::PublicationScope;

/// One target a run placed.
///
/// Carries both classifications, not only the one that decided the write. The publication
/// scope decided nothing here and is reported anyway, because `OD-PACKAGE-005` makes it the
/// input to a *later* decision — whether the placed asset may enter repository-distributed
/// state — and a mechanism that dropped it on the floor would force whoever asks that
/// question to re-derive it from a manifest this report's reader may not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement
{
    /// The surface label the intent carried, unread and passed through.
    pub surface: String,
    /// The target, as the intent spelled it.
    pub target: String,
    /// The class that decided what the write did.
    pub ownership_class: OwnershipClass,
    /// The scope that decided nothing here.
    pub publication_scope: PublicationScope,
    /// What the write did.
    pub outcome: PlacementOutcome,
}
