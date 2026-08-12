//! What became of a change set that was applied.

use crate::Effect;
use nomos_contracts::SnapshotId;
use nomos_contracts::GenerationId;
/// The outcome of submitting a change set.
///
/// Two arms rather than a generation and a `bool`, and no `Option`. A caller that has to
/// decide whether to invalidate must be told which world it is in, and
/// `unwrap_or(current_generation)` is how a workspace that did change gets treated as one
/// that did not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Applied
{
    /// The workspace is now something else.
    Advanced
    {
        generation: GenerationId,
        snapshot: SnapshotId,
        effects: Vec<Effect>,
    },
    /// Every change said what the workspace already said.
    Unchanged
    {
        generation: GenerationId,
        snapshot: SnapshotId,
        effects: Vec<Effect>,
    },
}

impl Applied
{
    #[must_use]
    pub const fn Generation(&self) -> GenerationId
    {
        return match self
        {
            Self::Advanced { generation, .. } | Self::Unchanged { generation, .. } => *generation,
        };
    }

    #[must_use]
    pub const fn Snapshot(&self) -> SnapshotId
    {
        return match self
        {
            Self::Advanced { snapshot, .. } | Self::Unchanged { snapshot, .. } => *snapshot,
        };
    }

    #[must_use]
    pub fn Effects(&self) -> &[Effect]
    {
        return match self
        {
            Self::Advanced { effects, .. } | Self::Unchanged { effects, .. } => effects,
        };
    }
}
