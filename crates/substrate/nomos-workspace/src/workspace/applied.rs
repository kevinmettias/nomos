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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The generation the advanced outcome under test is taken at, and the one its assertion
    /// reads back. Anything but `GenerationId::INITIAL`, which would make the assertion pass
    /// against an outcome that never advanced.
    const ADVANCED_GENERATION: u64 = 3;

    /// The byte the sample snapshot digest is filled with. Any byte will do; naming it keeps
    /// the snapshot the outcomes share from reading as a different one per test.
    const SAMPLE_SNAPSHOT_BYTE: u8 = 0x42;

    #[test]
    fn Test_Generation_Should_Report_The_Generation_Of_An_Advanced_Outcome()
    {
        let applied = Applied::Advanced {
            generation: GenerationId::From_Raw(ADVANCED_GENERATION),
            snapshot: Sample_Snapshot(),
            effects: Sample_Effects(),
        };

        assert_eq!(applied.Generation(), GenerationId::From_Raw(ADVANCED_GENERATION));
    }

    #[test]
    fn Test_Snapshot_Should_Report_The_Snapshot_Of_An_Unchanged_Outcome()
    {
        let snapshot = Sample_Snapshot();
        let applied = Applied::Unchanged {
            generation: GenerationId::INITIAL,
            snapshot,
            effects: Sample_Effects(),
        };

        assert_eq!(applied.Snapshot(), snapshot);
    }

    #[test]
    fn Test_Effects_Should_Report_The_Effects_Of_Either_Outcome()
    {
        let effects = Sample_Effects();
        let applied = Applied::Advanced {
            generation: GenerationId::INITIAL,
            snapshot: Sample_Snapshot(),
            effects: effects.clone(),
        };

        assert_eq!(applied.Effects(), effects.as_slice());
    }

    fn Sample_Effects() -> Vec<Effect>
    {
        return vec![Effect {
            path: "src/a.rs".to_owned(),
            kind: crate::EffectKind::Added,
        }];
    }

    fn Sample_Snapshot() -> SnapshotId
    {
        let bytes = [SAMPLE_SNAPSHOT_BYTE; nomos_contracts::Digest128::BYTE_LENGTH];
        let digest = nomos_contracts::Digest128::From_Bytes(bytes);

        return SnapshotId::From_Digest(digest);
    }
}
