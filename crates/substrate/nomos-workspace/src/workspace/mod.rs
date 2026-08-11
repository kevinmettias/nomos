//! The workspace, and the only thing that changes it.

mod naming;
#[cfg(test)]
mod tests;

use naming::{Normalize, Normalized};

use crate::effect::{Effect, EffectKind};
use crate::workspace_error::WorkspaceError;
use crate::applied::Applied;
use crate::change::Change;
use crate::workspace_change_set::WorkspaceChangeSet;
use crate::snapshot::WorkspaceSnapshot;
use crate::variant::BuildVariant;
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

        let Outcome { effects, altered } = self.Apply_Each(Normalized(changes)?);
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
            altered = altered || effect.Altered();
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
