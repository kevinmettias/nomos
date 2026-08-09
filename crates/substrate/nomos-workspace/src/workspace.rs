//! The workspace, and the only thing that changes it.

use crate::change::{Change, WorkspaceChangeSet};
use crate::snapshot::WorkspaceSnapshot;
use crate::variant::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId};
use nomos_store::{Authority, Commit, DocumentKind, DocumentStore, Recorded, StoreError};

/// What one change actually did.
///
/// Reported rather than assumed, because the submitter did not know. A checkout does not
/// diff before it lands and an editor's save hook does not consult the previous
/// generation, so [`Change::Present`] is a statement about the desired end state. This is
/// the answer to what it turned out to be.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effect
{
    Added
    {
        path: String,
    },
    Modified
    {
        path: String,
    },
    Removed
    {
        path: String,
    },
    /// The change said what the workspace already said.
    ///
    /// Not an error and not silence. An editor saving an unmodified file and a checkout
    /// landing where you already were both arrive here, and a workspace that treated them
    /// as changes would advance a generation and invalidate every fact in the store to
    /// reach the answer it already had.
    Redundant
    {
        path: String,
    },
    /// A removal of something that was not there.
    ///
    /// Distinct from `Redundant` because it is worth seeing: a submitter deleting files
    /// the workspace never had is usually a submitter working from a different idea of
    /// what the workspace contains.
    AlreadyAbsent
    {
        path: String,
    },
}

impl Effect
{
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return match self
        {
            Self::Added { path }
            | Self::Modified { path }
            | Self::Removed { path }
            | Self::Redundant { path }
            | Self::AlreadyAbsent { path } => path,
        };
    }

    /// Whether this effect changed what the workspace is.
    #[must_use]
    pub const fn Altered(&self) -> bool
    {
        return matches!(self, Self::Added { .. } | Self::Modified { .. } | Self::Removed { .. });
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceError
{
    /// A change set with nothing in it.
    ///
    /// Refused rather than accepted as `Unchanged`, because they are different mistakes. A
    /// set whose changes all turned out to be redundant is a submitter who did not know;
    /// an empty set is a submitter who built one and never put anything in it, and that is
    /// almost always a loop that iterated zero times.
    Vacuous,
    /// A path that does not name a workspace-relative file.
    Unnamed
    {
        path: String,
        reason: String,
    },
    /// The same path changed twice in one set.
    ///
    /// Refused rather than last-wins, because a set that says a path is both present and
    /// absent has no correct interpretation and picking one silently would make the
    /// workspace's state depend on the order a caller happened to push changes.
    Conflicting
    {
        path: String,
    },
    Store(StoreError),
}

impl core::fmt::Display for WorkspaceError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Vacuous => write!(
                formatter,
                "a change set with no changes in it. This is refused rather than ignored \
                 because it is almost always a loop that iterated zero times"
            ),
            Self::Unnamed { path, reason } => {
                write!(formatter, "`{path}` cannot name a workspace member: {reason}")
            }
            Self::Conflicting { path } => write!(
                formatter,
                "`{path}` is changed twice in one set. There is no correct reading of a \
                 path that is both present and absent, and choosing one would make the \
                 workspace depend on the order a caller pushed changes"
            ),
            Self::Store(error) => error.fmt(formatter),
        };
    }
}

impl std::error::Error for WorkspaceError {}

impl From<StoreError> for WorkspaceError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
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
        return self.snapshot.Content_Of(&Normalize(path));
    }

    /// The one door.
    ///
    /// Every change source enters here, and one applied set produces at most one new
    /// generation. The whole set is validated before any of it is applied, so a set that
    /// is refused leaves the workspace exactly as it was — a half-applied checkout is not
    /// a state anybody should be able to ask questions about.
    pub fn Apply(&mut self, changes: &WorkspaceChangeSet) -> Result<Applied, WorkspaceError>
    {
        if changes.Is_Empty()
        {
            return Err(WorkspaceError::Vacuous);
        }

        // Validate everything first. Normalizing here rather than at each use is also what
        // makes the conflict check see `src/a.rs` and `./src/A.rs` as one path.
        let mut normalized: Vec<(String, &Change)> = Vec::new();
        for change in changes.Changes()
        {
            let path = Named(change.Path())?;

            if normalized.iter().any(|(seen, _)| return *seen == path)
            {
                return Err(WorkspaceError::Conflicting { path });
            }
            normalized.push((path, change));
        }

        let mut effects = Vec::new();
        let mut altered = false;

        for (path, change) in normalized
        {
            let effect = match change
            {
                Change::Present { content, .. } =>
                {
                    let digest = nomos_model::Content_Digest(content.as_bytes());

                    match self.snapshot.Content_Of(&path)
                    {
                        Some(held) if held == digest => Effect::Redundant { path },
                        Some(_) =>
                        {
                            self.snapshot.Put(path.clone(), digest);
                            Effect::Modified { path }
                        }
                        None =>
                        {
                            self.snapshot.Put(path.clone(), digest);
                            Effect::Added { path }
                        }
                    }
                }
                Change::Absent { .. } => match self.snapshot.Take(&path)
                {
                    Some(_) => Effect::Removed { path },
                    None => Effect::AlreadyAbsent { path },
                },
            };

            altered = altered || effect.Altered();
            effects.push(effect);
        }

        effects.sort();

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
}

/// Validates and normalizes a submitted path.
fn Named(path: &str) -> Result<String, WorkspaceError>
{
    let refuse = |reason: &str| {
        return WorkspaceError::Unnamed {
            path: path.to_owned(),
            reason: reason.to_owned(),
        };
    };

    let unified = path.trim().replace('\\', "/");

    // An absolute path is the failure portability exists to prevent. A snapshot recording
    // `F:/repos/xvpe/crates/a.rs` is a snapshot that cannot be read anywhere else, and
    // catching it at the door is the difference between a refusal and a corpus of them.
    if unified.starts_with('/')
        || unified
            .split_once(':')
            .is_some_and(|(prefix, _)| return prefix.len() == 1)
    {
        return Err(refuse("a member is workspace-relative, and this is absolute"));
    }

    let segments: Vec<&str> = unified
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect();

    if segments.is_empty()
    {
        return Err(refuse("it names nothing"));
    }

    // `..` would let a member address something outside the workspace, and two spellings
    // of one file would be two members.
    if segments.contains(&"..")
    {
        return Err(refuse("a member cannot reach outside the workspace"));
    }

    return Ok(segments.join("/").to_lowercase());
}

/// The same normalization, for lookups that have already been validated elsewhere.
fn Normalize(path: &str) -> String
{
    return Named(path).unwrap_or_else(|_| return path.to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::change::ChangeSource;

    fn Variant() -> BuildVariant
    {
        return BuildVariant::New("x86_64-unknown-linux-gnu", "dev", "1.85", ["analysis"]);
    }

    fn Fresh() -> Workspace
    {
        return Workspace::Empty(
            Variant(),
            ConfigurationId::From_Digest(Digest128::From_Bytes([0x11; 16])),
        );
    }

    fn Edit(path: &str, content: &str) -> WorkspaceChangeSet
    {
        return WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present(path, content);
    }

    #[test]
    fn Test_One_Applied_Set_Should_Produce_One_Generation()
    {
        let mut workspace = Fresh();
        assert_eq!(workspace.Generation(), GenerationId::INITIAL);

        let applied = workspace
            .Apply(
                &WorkspaceChangeSet::From(ChangeSource::GitCheckout)
                    .Present("src/a.rs", "pub fn a() {}")
                    .Present("src/b.rs", "pub fn b() {}")
                    .Present("src/c.rs", "pub fn c() {}"),
            )
            .expect("a checkout applies");

        assert_eq!(applied.Generation(), GenerationId::From_Raw(1));
        assert_eq!(
            workspace.Generation(),
            GenerationId::From_Raw(1),
            "three files in one set is one event and one generation"
        );
        assert_eq!(applied.Effects().len(), 3);
    }

    /// A save that changed nothing is not a change. Advancing here would invalidate every
    /// fact in the store to arrive back at the answer it already had.
    #[test]
    fn Test_A_Change_That_Says_What_Is_Already_True_Should_Not_Advance()
    {
        let mut workspace = Fresh();
        workspace.Apply(&Edit("src/a.rs", "pub fn a() {}")).expect("applies");
        let before = workspace.Generation();
        let identity = workspace.Id();

        let applied = workspace
            .Apply(&Edit("src/a.rs", "pub fn a() {}"))
            .expect("a redundant save is not an error");

        assert!(matches!(applied, Applied::Unchanged { .. }), "{applied:?}");
        assert_eq!(workspace.Generation(), before);
        assert_eq!(workspace.Id(), identity);
        assert_eq!(
            applied.Effects(),
            &[Effect::Redundant {
                path: "src/a.rs".to_owned()
            }]
        );
    }

    /// The positive control for the test above. If nothing ever advanced, it would pass
    /// over a workspace that cannot change at all.
    #[test]
    fn Test_A_Change_That_Says_Something_New_Should_Advance()
    {
        let mut workspace = Fresh();
        workspace.Apply(&Edit("src/a.rs", "pub fn a() {}")).expect("applies");
        let before = workspace.Generation();

        let applied = workspace
            .Apply(&Edit("src/a.rs", "pub fn changed() {}"))
            .expect("applies");

        assert!(matches!(applied, Applied::Advanced { .. }), "{applied:?}");
        assert!(workspace.Generation() > before);
        assert_eq!(
            applied.Effects(),
            &[Effect::Modified {
                path: "src/a.rs".to_owned()
            }]
        );
    }

    /// Identity is the state, not the history. This is the property that makes a snapshot
    /// worth content-addressing at all.
    #[test]
    fn Test_Editing_A_File_Back_Should_Return_To_The_Same_Snapshot()
    {
        let mut workspace = Fresh();
        workspace.Apply(&Edit("src/a.rs", "original")).expect("applies");
        let original = workspace.Id();

        workspace.Apply(&Edit("src/a.rs", "changed")).expect("applies");
        assert_ne!(workspace.Id(), original);

        workspace.Apply(&Edit("src/a.rs", "original")).expect("applies");

        assert_eq!(
            workspace.Id(),
            original,
            "the workspace is what it was, however it got back"
        );
        assert_eq!(
            workspace.Generation(),
            GenerationId::From_Raw(3),
            "and the generation counts what happened, which is three changes"
        );
    }

    #[test]
    fn Test_A_Removal_Should_Take_The_Member_And_A_Second_Should_Not()
    {
        let mut workspace = Fresh();
        workspace.Apply(&Edit("src/a.rs", "pub fn a() {}")).expect("applies");

        let removed = workspace
            .Apply(&WorkspaceChangeSet::From(ChangeSource::AgentEdit).Absent("src/a.rs"))
            .expect("applies");

        assert!(matches!(removed, Applied::Advanced { .. }));
        assert_eq!(workspace.Content_Of("src/a.rs"), None);

        let again = workspace
            .Apply(&WorkspaceChangeSet::From(ChangeSource::AgentEdit).Absent("src/a.rs"))
            .expect("removing what is not there is not an error");

        assert!(matches!(again, Applied::Unchanged { .. }));
        assert_eq!(
            again.Effects(),
            &[Effect::AlreadyAbsent {
                path: "src/a.rs".to_owned()
            }],
            "worth seeing: the submitter has a different idea of what is here"
        );
    }

    #[test]
    fn Test_An_Empty_Change_Set_Should_Be_Refused()
    {
        let mut workspace = Fresh();

        assert_eq!(
            workspace.Apply(&WorkspaceChangeSet::From(ChangeSource::Correction)),
            Err(WorkspaceError::Vacuous)
        );
    }

    /// The failure portability exists to prevent, caught at the door rather than in the
    /// bytes.
    #[test]
    fn Test_An_Absolute_Path_Should_Be_Refused()
    {
        let mut workspace = Fresh();

        for path in ["F:/repos/xvpe/a.rs", "/usr/src/a.rs", "C:\\src\\a.rs", "\\\\?\\F:\\a.rs"]
        {
            assert!(
                workspace.Apply(&Edit(path, "content")).is_err(),
                "`{path}` is not workspace-relative"
            );
        }
    }

    #[test]
    fn Test_A_Path_Reaching_Outside_The_Workspace_Should_Be_Refused()
    {
        let mut workspace = Fresh();

        for path in ["../outside.rs", "src/../../outside.rs", ".."]
        {
            assert!(workspace.Apply(&Edit(path, "content")).is_err(), "`{path}`");
        }
    }

    /// Two spellings of one path are one member, so a set naming both is a set with no
    /// correct reading.
    #[test]
    fn Test_One_Path_Changed_Twice_Should_Be_Refused()
    {
        let mut workspace = Fresh();

        assert_eq!(
            workspace.Apply(
                &WorkspaceChangeSet::From(ChangeSource::CodeGenerator)
                    .Present("src/a.rs", "one")
                    .Present("./src/A.rs", "other")
            ),
            Err(WorkspaceError::Conflicting {
                path: "src/a.rs".to_owned()
            })
        );
    }

    /// A refused set leaves the workspace exactly as it was. A half-applied checkout is
    /// not a state anybody should be able to ask questions about.
    #[test]
    fn Test_A_Refused_Set_Should_Change_Nothing()
    {
        let mut workspace = Fresh();
        workspace.Apply(&Edit("src/a.rs", "original")).expect("applies");
        let before = (workspace.Generation(), workspace.Id());

        let refused = workspace.Apply(
            &WorkspaceChangeSet::From(ChangeSource::GitCheckout)
                .Present("src/b.rs", "new")
                .Present("src/c.rs", "new")
                .Present("/absolute/d.rs", "new"),
        );

        assert!(refused.is_err());
        assert_eq!((workspace.Generation(), workspace.Id()), before);
        assert_eq!(
            workspace.Content_Of("src/b.rs"),
            None,
            "the valid changes in a refused set must not have landed"
        );
    }

    /// Provenance is a fact about the change, not about the workspace. Two workspaces
    /// holding the same files are the same workspace however the files got there.
    #[test]
    fn Test_The_Source_Should_Not_Reach_The_Workspace_Identity()
    {
        let mut identities = Vec::new();

        for source in ChangeSource::All()
        {
            let mut workspace = Fresh();
            workspace
                .Apply(&WorkspaceChangeSet::From(*source).Present("src/a.rs", "pub fn a() {}"))
                .expect("applies");
            identities.push(workspace.Id());
        }

        assert!(
            identities.windows(2).all(|pair| return pair.first() == pair.last()),
            "five sources, one workspace: {identities:?}"
        );
    }
}
