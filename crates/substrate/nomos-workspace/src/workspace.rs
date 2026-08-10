//! The workspace, and the only thing that changes it.

use crate::effect::Effect;
use crate::workspace_error::WorkspaceError;
use crate::applied::Applied;
use crate::change::Change;
use crate::workspace_change_set::WorkspaceChangeSet;
use crate::snapshot::WorkspaceSnapshot;
use crate::variant::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId};
use nomos_store::{Authority, Commit, DocumentKind, DocumentStore, Recorded};

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

        let (effects, altered) = self.Apply_Each(Normalized(changes)?);
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
    fn Apply_Each(&mut self, normalized: Vec<(String, &Change)>) -> (Vec<Effect>, bool)
    {
        let mut effects = Vec::new();
        let mut altered = false;

        for (path, change) in normalized
        {
            let effect = self.Applied_One(path, change);
            altered = altered || effect.Altered();
            effects.push(effect);
        }
        effects.sort();

        return (effects, altered);
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
                Some(_) => Effect::Removed { path },
                None => Effect::AlreadyAbsent { path },
            };
        };

        let digest = nomos_model::Content_Digest(content.as_bytes());

        return self.Put(path, digest);
    }

    /// Writes one member, saying whether the write added it, changed it, or said nothing.
    fn Put(&mut self, path: String, digest: nomos_contracts::Digest128) -> Effect
    {
        let held = self.snapshot.Content_Of(&path);
        if held == Some(digest)
        {
            return Effect::Redundant { path };
        }
        self.snapshot.Put(path.clone(), digest);

        if held.is_some()
        {
            return Effect::Modified { path };
        }

        return Effect::Added { path };
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

/// Every change with its path validated and normalized, refusing the whole set on the first
/// bad or repeated path.
///
/// Validated in full before anything is applied, so a set that is refused leaves the
/// workspace exactly as it was — a half-applied checkout is not a state anybody should be
/// able to ask questions about. Normalizing here rather than at each use is also what makes
/// the conflict check see `src/a.rs` and `./src/A.rs` as one path.
fn Normalized(changes: &WorkspaceChangeSet) -> Result<Vec<(String, &Change)>, WorkspaceError>
{
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

    return Ok(normalized);
}

/// Validates and normalizes a submitted path.
fn Named(path: &str) -> Result<String, WorkspaceError>
{
    let unified = path.trim().replace('\\', "/");
    let segments = Segments_Of(&unified);

    if let Some(reason) = Unnameable(&unified, &segments)
    {
        return Err(WorkspaceError::Unnamed {
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
    }

    return Ok(segments.join("/").to_lowercase());
}

/// Why a path cannot name a member, if it cannot.
///
/// `..` would let a member address something outside the workspace, and two spellings of
/// one file would be two members.
fn Unnameable(unified: &str, segments: &[&str]) -> Option<&'static str>
{
    if Is_Absolute(unified)
    {
        return Some("a member is workspace-relative, and this is absolute");
    }

    if segments.is_empty()
    {
        return Some("it names nothing");
    }

    if segments.contains(&"..")
    {
        return Some("a member cannot reach outside the workspace");
    }

    return None;
}

/// A path's components, with the ones that name nothing dropped.
///
/// `.` and an empty segment both address the directory they sit in, so keeping either would
/// make `src/./a.rs` and `src/a.rs` two members naming one file.
fn Segments_Of(unified: &str) -> Vec<&str>
{
    return unified
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect();
}

/// A path that names a place on one machine rather than a member of a workspace.
///
/// Both spellings are refused: a leading separator and a single-letter drive prefix. A
/// snapshot recording `F:/repos/xvpe/crates/a.rs` is a snapshot that cannot be read anywhere
/// else, and catching it at the door is the difference between a refusal and a corpus of
/// them.
fn Is_Absolute(unified: &str) -> bool
{
    return unified.starts_with('/')
        || unified
            .split_once(':')
            .is_some_and(|(prefix, _)| return prefix.len() == 1);
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
    use crate::change_source::ChangeSource;

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

        let checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
            .Present("src/a.rs", "pub fn a() {}")
            .Present("src/b.rs", "pub fn b() {}")
            .Present("src/c.rs", "pub fn c() {}");

        let applied = workspace.Apply(&checkout).expect("a checkout applies");

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
        let first = Edit("src/a.rs", "pub fn a() {}");
        workspace.Apply(&first).expect("applies");
        let before = workspace.Generation();
        let identity = workspace.Id();

        let redundant = Edit("src/a.rs", "pub fn a() {}");
        let applied = workspace
            .Apply(&redundant)
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
        let first = Edit("src/a.rs", "pub fn a() {}");
        workspace.Apply(&first).expect("applies");
        let before = workspace.Generation();

        let changed = Edit("src/a.rs", "pub fn changed() {}");
        let applied = workspace.Apply(&changed).expect("applies");

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
        let first = Edit("src/a.rs", "original");
        workspace.Apply(&first).expect("applies");
        let original = workspace.Id();

        let changed = Edit("src/a.rs", "changed");
        workspace.Apply(&changed).expect("applies");
        assert_ne!(workspace.Id(), original);

        let back = Edit("src/a.rs", "original");
        workspace.Apply(&back).expect("applies");

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
        let first = Edit("src/a.rs", "pub fn a() {}");
        workspace.Apply(&first).expect("applies");

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
        let first = Edit("src/a.rs", "original");
        workspace.Apply(&first).expect("applies");
        let before = (workspace.Generation(), workspace.Id());

        let checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
            .Present("src/b.rs", "new")
            .Present("src/c.rs", "new")
            .Present("/absolute/d.rs", "new");

        let refused = workspace.Apply(&checkout);

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
            let change = WorkspaceChangeSet::From(*source).Present("src/a.rs", "pub fn a() {}");
            workspace.Apply(&change).expect("applies");
            identities.push(workspace.Id());
        }

        assert!(
            identities.windows(2).all(|pair| return pair.first() == pair.last()),
            "five sources, one workspace: {identities:?}"
        );
    }
}
