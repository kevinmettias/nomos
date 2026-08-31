//! The workspace, and the only thing that changes it.

// A change set over a workspace, and the refusals a workspace raises.
#[path = "workspace/change_set.rs"]
mod change_set;
#[path = "workspace/error.rs"]
mod error;

pub use change_set::ChangeSet as WorkspaceChangeSet;
pub use error::Error as WorkspaceError;

mod naming;
#[cfg(test)]
mod tests;

use naming::{Normalize_Path, Normalized_Changes};

use crate::Effect;
use crate::EffectKind;
use crate::Applied;
use crate::Change;
use crate::WorkspaceSnapshot;
use crate::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId};
use nomos_store::{Authority, Commit, DocumentKind, DocumentStore, Recorded};

/// What applying a set of changes came to.
///
/// Named rather than a pair, so that the caller reading `altered` is reading the question
/// it is asking — whether the generation must advance — and not a position.
struct Outcome
{
    effects: Vec<Effect>,
    altered: bool,
}

/// What the workspace currently is.
///
/// The only `&mut self` method that changes anything is [`Workspace::Apply`]. That is the
/// design, not an accident of what has been needed so far: every other way to change a
/// workspace would be a second answer to what it currently is.
pub struct Workspace
{
    snapshot: WorkspaceSnapshot,
    generation: GenerationId,
}

impl Workspace
{
    /// An empty workspace at the initial generation.
    #[must_use]
    pub fn Empty(variant: BuildVariant, configuration: ConfigurationId) -> Self
    {
        return Self {
            snapshot: WorkspaceSnapshot::Of(variant, configuration),
            generation: GenerationId::INITIAL,
        };
    }

    #[must_use]
    pub const fn Generation(&self) -> GenerationId
    {
        return self.generation;
    }

    #[must_use]
    pub const fn Snapshot(&self) -> &WorkspaceSnapshot
    {
        return &self.snapshot;
    }

    #[must_use]
    pub fn Id(&self) -> SnapshotId
    {
        return self.snapshot.Id();
    }

    /// The content address of one member, or `None` if the workspace does not have it.
    #[must_use]
    pub fn Content_Of(&self, path: &str) -> Option<Digest128>
    {
        return self.snapshot.Content_Of(&Normalize_Path(path));
    }

    /// The one door.
    ///
    /// Every change source enters here, and one applied set produces at most one new
    /// generation. The whole set is validated before any of it is applied, so a set that
    /// is refused leaves the workspace exactly as it was — a half-applied checkout is not
    /// a state anybody should be able to ask questions about.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError::Vacuous`] if `changes` is empty, [`WorkspaceError::Unnamed`]
    /// if a path does not name a workspace-relative file, or [`WorkspaceError::Conflicting`]
    /// if one path is changed twice in the same set.
    pub fn Apply(&mut self, changes: &WorkspaceChangeSet) -> Result<Applied, WorkspaceError>
    {
        if changes.Is_Empty()
        {
            return Err(WorkspaceError::Vacuous);
        }

        let Outcome { effects, altered } = self.Apply_Each(Normalized_Changes(changes)?);
        if !altered
        {
            return Ok(Applied::Unchanged {
                generation: self.generation,
                snapshot: self.snapshot.Id(),
                effects,
            });
        }

        self.generation = self.generation.Next();

        return Ok(Applied::Advanced {
            generation: self.generation,
            snapshot: self.snapshot.Id(),
            effects,
        });
    }

    /// Applies every change, and says whether any of them moved the snapshot.
    ///
    /// A set of changes that all turn out to be redundant is applied and alters nothing,
    /// which is what keeps a re-run of one checkout from advancing the generation and
    /// invalidating a corpus of facts that are still true.
    fn Apply_Each(&mut self, normalized: Vec<(String, &Change)>) -> Outcome
    {
        let mut effects = Vec::new();
        let mut altered = false;

        for (path, change) in normalized
        {
            let effect = self.Applied_One(path, change);
            altered = altered || effect.Is_Altered();
            effects.push(effect);
        }
        effects.sort();

        return Outcome { effects, altered };
    }

    /// One change against the snapshot, and what it did to it.
    ///
    /// A write of content the snapshot already holds is `Redundant` rather than `Modified`.
    /// The distinction is what keeps a checkout that changed nothing from advancing the
    /// generation and invalidating a corpus of facts that are still true.
    fn Applied_One(&mut self, path: String, change: &Change) -> Effect
    {
        let Change::Present { content, .. } = change
        else
        {
            return match self.snapshot.Take(&path)
            {
                Some(_) => Effect {
                    path,
                    kind: EffectKind::Removed,
                },
                None => Effect {
                    path,
                    kind: EffectKind::AlreadyAbsent,
                },
            };
        };

        let digest = nomos_model::Content_Digest(content.as_bytes());

        return self.Put(path, digest);
    }

    /// Records this state in a document store.
    ///
    /// The workspace's own bytes are the only thing written, and they carry no reference
    /// to a tree. A store that has them can answer every question this workspace answers
    /// without the tree ever having existed for it.
    ///
    /// # Why the record is a `Fact` and not a kind of its own
    ///
    /// Settled by P8-KIND and recorded in `docs/records/OD-STORE-001`. A [`DocumentKind`]
    /// earns its place when the store must *behave* differently for documents of that kind
    /// — it decides which authority may hold them, and [`nomos_store::Index`] decodes every
    /// [`DocumentKind::Commit`] as a commit manifest. Neither is true of a workspace state:
    /// it is observed, like a fact, and the store makes no structural promise about it.
    ///
    /// A kind added for a label rather than for behaviour is the first step to one kind per
    /// schema, at which point `DocumentKind` says nothing `SchemaId` did not already say.
    ///
    /// What was wrong was the *name*. `DocumentKind::Snapshot` never meant a snapshot; it
    /// meant a commit, and the collision with a workspace state was the confusion this
    /// comment used to describe as unavoidable. It is now `DocumentKind::Commit`, and the
    /// two concepts no longer share a word.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError::Store`] if `store` refuses the commit.
    pub fn Record(&self, store: &mut DocumentStore) -> Result<(), WorkspaceError>
    {
        let recorded = Recorded::New(
            DocumentKind::Fact,
            SchemaId::New(crate::snapshot::SNAPSHOT_SCHEMA),
            self.snapshot.Encode(),
        );

        let commit = Commit::Under(
            self.Id(),
            self.snapshot.Variant().Id(),
            self.snapshot.Configuration(),
            self.generation,
        )
        .Recording(recorded);

        store.Commit(&commit)?;

        return Ok(());
    }

    /// The authority a workspace snapshot belongs to.
    ///
    /// Observed, and stated here so that a caller opening a store for a workspace cannot
    /// guess. A workspace snapshot is a measurement of a tree, not something anybody
    /// authored, and putting it in an authored store would make one authority's writes the
    /// other's evidence.
    #[must_use]
    pub const fn Authority() -> Authority
    {
        return Authority::Observed;
    }

    /// Writes one member, saying whether the write added it, changed it, or said nothing.
    fn Put(&mut self, path: String, digest: nomos_contracts::Digest128) -> Effect
    {
        let held = self.snapshot.Content_Of(&path);
        if held == Some(digest)
        {
            return Effect {
                path,
                kind: EffectKind::Redundant,
            };
        }
        self.snapshot.Put(path.clone(), digest);

        if held.is_some()
        {
            return Effect {
                path,
                kind: EffectKind::Modified,
            };
        }

        return Effect {
            path,
            kind: EffectKind::Added,
        };
    }
}

#[cfg(test)]
mod local_tests
{
    use super::*;
    use crate::ChangeSource;

    #[test]
    fn Test_Empty_Should_Have_No_Members_And_A_Fresh_Counter()
    {
        let workspace = Fresh_Workspace();

        assert_eq!(workspace.Generation(), GenerationId::INITIAL);
        assert!(workspace.Snapshot().Is_Empty());
    }

    #[test]
    fn Test_Generation_Should_Advance_By_Exactly_One_Per_Applied_Set()
    {
        let mut workspace = Fresh_Workspace();
        let changes = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("a.rs", "fn a() {}");

        workspace.Apply(&changes).expect("applies");

        assert_eq!(workspace.Generation(), GenerationId::From_Raw(1));
    }

    #[test]
    fn Test_Snapshot_Should_Expose_The_Workspaces_Current_State()
    {
        let mut workspace = Fresh_Workspace();
        let changes = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("a.rs", "fn a() {}");

        workspace.Apply(&changes).expect("applies");

        assert_eq!(workspace.Snapshot().Length(), 1);
        assert!(!workspace.Snapshot().Is_Empty());
    }

    #[test]
    fn Test_Id_Should_Match_The_Snapshots_Own_Identity()
    {
        let workspace = Fresh_Workspace();

        assert_eq!(workspace.Id(), workspace.Snapshot().Id());
    }

    #[test]
    fn Test_Content_Of_Should_Find_What_Was_Written_At_A_Path()
    {
        let mut workspace = Fresh_Workspace();
        let changes = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("src/a.rs", "fn a() {}");
        workspace.Apply(&changes).expect("applies");

        let expected = nomos_model::Content_Digest("fn a() {}".as_bytes());

        assert_eq!(
            workspace.Content_Of("SRC/A.RS"),
            Some(expected),
            "lookup normalizes the path"
        );
        assert_eq!(workspace.Content_Of("does/not/exist.rs"), None);
    }

    #[test]
    fn Test_Apply_Should_Refuse_A_Vacuous_Change_Set()
    {
        let mut workspace = Fresh_Workspace();

        assert_eq!(
            workspace.Apply(&WorkspaceChangeSet::From(ChangeSource::Correction)),
            Err(WorkspaceError::Vacuous)
        );
    }

    #[test]
    fn Test_Record_Should_Let_The_Store_Read_The_State_Back()
    {
        let workspace = Fresh_Workspace();
        let mut store = DocumentStore::For(Workspace::Authority());

        workspace.Record(&mut store).expect("an observed store admits it");

        let recorded: Vec<_> = store
            .Documents()
            .values()
            .filter(|document| return document.kind == DocumentKind::Fact)
            .collect();
        assert_eq!(recorded.len(), 1, "one state was recorded");

        let decoded =
            WorkspaceSnapshot::Decode(&recorded.first().expect("the assertion above found exactly one recorded document").bytes)
                .expect("it decodes");
        assert_eq!(decoded.Id(), workspace.Id());
    }

    #[test]
    fn Test_Authority_Should_Be_Observed_Not_Authored()
    {
        assert_eq!(Workspace::Authority(), Authority::Observed);
    }

    fn Sample_Variant() -> BuildVariant
    {
        return BuildVariant::New("x86_64-pc-windows-msvc", "dev", "1.85", ["analysis"]);
    }

    fn Sample_Configuration() -> ConfigurationId
    {
        return ConfigurationId::From_Digest(Digest128::From_Bytes([0x24; 16]));
    }

    fn Fresh_Workspace() -> Workspace
    {
        return Workspace::Empty(Sample_Variant(), Sample_Configuration());
    }
}
