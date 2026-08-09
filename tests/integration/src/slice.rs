//! The run: syntax facts per file, a rollup per directory, and what a change costs.

use crate::context::{Host_Variant, Resolved_Configuration};
use crate::corpus::{Corpus, SourceFile, Subject_Of_Path};
use crate::surface::{self, Surface};
use nomos_analysis::{
    Context, Dependency, FactIdentity, FactKey, FactPayload, FactReader, FactStore, GuaranteeDigest,
    InputDigest, InvalidationReport, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{Registry, Requirement};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, Digest128, EvidenceClass,
    FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId, SnapshotId, SubjectId,
};
use nomos_lang_rust as rust;
use nomos_model::Digest_Of_Parts;
use nomos_workspace::{Applied, ChangeSource, Workspace, WorkspaceChangeSet};

/// What one pass over a corpus did.
///
/// Every field is a count with a named denominator somewhere in this struct. The
/// prototype reported "0 findings" for a check that had walked nothing, and the defect was
/// invisible because the report had no place to put the number that would have shown it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunReport
{
    pub files_seen: usize,
    pub syntax_materialized: usize,
    pub syntax_reused: usize,
    /// Files the provider refused, by path and reason. Named, not counted.
    pub refused: Vec<(String, String)>,
    pub groups_seen: usize,
    pub surface_materialized: usize,
    pub surface_reused: usize,
    /// Rollups that could not read one of their members, by group. A rollup over three of
    /// four files is a degraded answer, and reporting it as an answer is how a corpus with
    /// a hole in it reads as a corpus that is fine.
    pub degraded: Vec<String>,
    /// Subjects whose facts were written in this pass, in the order they were written.
    ///
    /// The list the invalidation assertions are made against. "Two facts recomputed" is
    /// satisfied by recomputing the wrong two; naming them is not.
    pub recomputed: Vec<Recompute>,
}

/// One fact written during a pass, identified by what it is about.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Recompute
{
    pub capability: String,
    pub subject: String,
}

impl RunReport
{
    /// The subjects recomputed for one capability, sorted.
    #[must_use]
    pub fn Recomputed_For(&self, capability: &str) -> Vec<String>
    {
        let mut subjects: Vec<String> = self
            .recomputed
            .iter()
            .filter(|entry| return entry.capability == capability)
            .map(|entry| return entry.subject.clone())
            .collect();
        subjects.sort();

        return subjects;
    }

    /// Everything written in this pass, sorted, as `capability of subject`.
    #[must_use]
    pub fn Recomputed(&self) -> Vec<String>
    {
        let mut all: Vec<String> = self
            .recomputed
            .iter()
            .map(|entry| return format!("{} of {}", entry.capability, entry.subject))
            .collect();
        all.sort();

        return all;
    }
}

/// What an edit did.
///
/// Two arms rather than a report and a flag, mirroring [`Applied`] for the same reason: a
/// caller has to be told which world it is in, and an empty [`InvalidationReport`] would
/// make "nothing was invalidated because nothing changed" indistinguishable from
/// "invalidation ran over a changed workspace and reached nothing", which is a bug.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edited
{
    /// The workspace is now something else, and the store was told.
    Advanced
    {
        applied: Applied,
        invalidated: InvalidationReport,
    },
    /// The edit said what the workspace already said, so nothing was invalidated.
    ///
    /// An editor saving an unmodified file arrives here. Invalidating anyway would discard
    /// every fact reachable from that file in order to recompute the answers the store
    /// already held.
    Unchanged
    {
        applied: Applied,
    },
}

impl Edited
{
    #[must_use]
    pub const fn Outcome(&self) -> &Applied
    {
        return match self
        {
            Self::Advanced { applied, .. } | Self::Unchanged { applied } => applied,
        };
    }
}

/// The composed system: a workspace that says what is there, a registry that resolves
/// providers, and a store that holds what they produced.
pub struct Slice
{
    store: MemoryFactStore,
    registry: Registry,
    workspace: Workspace,
    /// The snapshot every fact key this slice writes names.
    ///
    /// # Why it does not follow the workspace
    ///
    /// [`nomos_analysis::FactKey`] carries a [`SnapshotId`], so a fact is filed under one
    /// workspace state. Taking `workspace.Id()` at materialization time — the obvious
    /// reading of "the context comes from the workspace" — makes every key in the store
    /// change the moment any one file is edited, because a workspace snapshot is a digest
    /// over all of its members. One keystroke would recompute the corpus, and this slice
    /// measured exactly that: over the precision corpus, editing `alpha/one.rs` recomputes
    /// six facts instead of two.
    ///
    /// So the slice pins. The snapshot names the workspace state this store was opened
    /// against, edits advance the *generation*, and the store's own generation bookkeeping
    /// is what decides which facts are still current. `docs/records/OD-ANALYSIS-001`
    /// records the finding underneath: the snapshot component of a fact key is a third
    /// answer to a question `semantic_inputs` and the generation already answer between
    /// them, and being the coarsest of the three it silently defeats both.
    pinned: SnapshotId,
    /// Taken from the workspace once. Neither can change — [`Workspace`] has no door for
    /// them — so recomputing a digest per fact key would buy nothing.
    variant: BuildVariantId,
    configuration: ConfigurationId,
    generation: GenerationId,
}

impl Slice
{
    /// Declares both capabilities and registers both providers.
    ///
    /// # Panics
    ///
    /// If the registry refuses a declaration or an offer. Both are decided by this
    /// function's own constants, so a refusal here is a contradiction in the composition
    /// rather than a runtime condition — and continuing past it would produce a run whose
    /// facts nobody offered.
    #[must_use]
    pub fn Registered() -> Registry
    {
        let mut registry = Registry::New();

        registry
            .Declare(rust::Capability_Contract())
            .expect("the syntax capability is declared once");
        registry
            .Offer(rust::Provider_Offer())
            .expect("the Rust provider's offer is within its capability's ceiling");
        registry
            .Declare(surface::Capability_Contract())
            .expect("the surface capability is declared once");
        registry
            .Offer(surface::Provider_Offer())
            .expect("the rollup's offer is within its capability's ceiling");

        return registry;
    }

    /// A slice over an empty workspace.
    ///
    /// The context is real and derived even here: the variant is what this binary was
    /// compiled as, the configuration is the composition that was just built, and the
    /// snapshot is the identity of a workspace holding nothing — which is a state, and is
    /// not the same state as one holding a file.
    #[must_use]
    pub fn Composed() -> Self
    {
        let registry = Self::Registered();
        let workspace = Workspace::Empty(Host_Variant(), Resolved_Configuration(&registry));

        return Self {
            store: MemoryFactStore::New(),
            pinned: workspace.Id(),
            variant: workspace.Snapshot().Variant().Id(),
            configuration: workspace.Snapshot().Configuration(),
            generation: workspace.Generation(),
            workspace,
            registry,
        };
    }

    /// A slice over a corpus, ingested through the workspace's one door.
    ///
    /// The whole corpus arrives as a single [`ChangeSource::GitCheckout`] change set,
    /// because a checkout is one event. Applying a file at a time would produce one
    /// generation per file, and every intermediate one would describe a tree that never
    /// existed.
    ///
    /// # Panics
    ///
    /// If the workspace refuses the set. The two ways that happens are both worth a stop
    /// rather than a degraded run: an empty corpus is a walk that read nothing, and a
    /// conflict is two corpus paths that normalize to one workspace member — which would
    /// otherwise become one subject silently answering for two files.
    #[must_use]
    pub fn Over(corpus: &Corpus) -> Self
    {
        let mut slice = Self::Composed();
        let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);

        for file in &corpus.files
        {
            checkout = checkout.Present(file.path.clone(), file.source.clone());
        }

        let applied = slice.workspace.Apply(&checkout).unwrap_or_else(|error| {
            panic!(
                "the corpus at {} could not be ingested: {error}",
                corpus.root.display()
            )
        });

        slice.generation = applied.Generation();
        slice.pinned = applied.Snapshot();

        return slice;
    }

    #[must_use]
    pub const fn Generation(&self) -> GenerationId
    {
        return self.generation;
    }

    #[must_use]
    pub const fn Store(&self) -> &MemoryFactStore
    {
        return &self.store;
    }

    #[must_use]
    pub const fn Registry(&self) -> &Registry
    {
        return &self.registry;
    }

    #[must_use]
    pub const fn Workspace(&self) -> &Workspace
    {
        return &self.workspace;
    }

    /// The snapshot this slice's fact keys name. See the field's own note for why it is not
    /// `self.workspace.Id()`.
    #[must_use]
    pub const fn Pinned(&self) -> SnapshotId
    {
        return self.pinned;
    }

    fn Context(&self) -> Context
    {
        return Context {
            snapshot: self.pinned,
            variant: self.variant,
            configuration: self.configuration,
            generation: self.generation,
        };
    }

    /// The semantic inputs of a syntax fact.
    ///
    /// Recomputed here rather than obtained from `nomos-lang-rust`, deliberately. If the
    /// two disagreed, the rollup's [`FactReader::Require`] would look up a key the
    /// provider never wrote and the run would degrade — loudly, in `RunReport::degraded`.
    /// A shared helper would make the two agree by construction and this whole class of
    /// mismatch would become untestable.
    fn Syntax_Inputs(source: &str) -> InputDigest
    {
        return InputDigest::Of(&[source.as_bytes()]);
    }

    /// The key a syntax fact about this file is filed under.
    ///
    /// Public so that a test can compare one file's identity across two slices pinned to
    /// two different workspace states. That comparison is the evidence for the pinning
    /// decision recorded on [`Slice::pinned`], and it cannot be made from outside without
    /// this.
    #[must_use]
    pub fn Syntax_Key(&self, file: &SourceFile) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New(rust::CAPABILITY),
            contract_version: rust::CONTRACT_VERSION,
            subject: file.subject,
            semantic_inputs: Self::Syntax_Inputs(&file.source),
            provider: ProviderId::New(rust::PROVIDER),
            provider_version: rust::CONTRACT_VERSION,
            guarantee: GuaranteeDigest::Of(&rust::Declared_Guarantee()),
            snapshot: self.pinned,
            variant: self.variant,
            configuration: self.configuration,
        };
    }

    /// The rollup's semantic inputs: the digests of every member's inputs, in corpus order.
    ///
    /// A digest over the members rather than over the directory's name, so that editing
    /// any member changes the rollup's identity. Keying it on the name alone would leave a
    /// stale rollup matching a key that is still current, which is the failure mode a
    /// content-addressed key exists to prevent.
    fn Surface_Inputs(members: &[&SourceFile]) -> InputDigest
    {
        let parts: Vec<[u8; Digest128::BYTE_LENGTH]> = members
            .iter()
            .map(|member| return *Self::Syntax_Inputs(&member.source).Digest().Bytes())
            .collect();
        let borrowed: Vec<&[u8]> = parts.iter().map(|part| return part.as_slice()).collect();

        return InputDigest::From_Digest(Digest_Of_Parts(&borrowed));
    }

    fn Surface_Key(&self, group: SubjectId, members: &[&SourceFile]) -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New(surface::CAPABILITY),
            contract_version: surface::CONTRACT_VERSION,
            subject: group,
            semantic_inputs: Self::Surface_Inputs(members),
            provider: ProviderId::New(surface::PROVIDER),
            provider_version: surface::CONTRACT_VERSION,
            guarantee: GuaranteeDigest::Of(&surface::Declared_Guarantee()),
            snapshot: self.pinned,
            variant: self.variant,
            configuration: self.configuration,
        };
    }

    /// The rollup currently held for a group, if there is one.
    ///
    /// Looked up by recomputing the key from the members rather than remembered from the
    /// run, so a caller asking for `alpha`'s surface after an edit gets the fact matching
    /// `alpha` *as it now is* — or nothing. A cached handle would happily return the
    /// rollup for a directory that no longer exists in that shape.
    #[must_use]
    pub fn Surface_Of(&self, members: &[&SourceFile]) -> Option<MaterializedFact>
    {
        let subject = members.first()?.group_subject;
        let identity = self.Surface_Key(subject, members).At(self.generation);

        return self.store.Current(&identity, self.generation);
    }

    /// Whether the store already holds this fact at the current generation.
    fn Held(&self, key: &FactKey) -> bool
    {
        let identity = FactIdentity {
            key: key.clone(),
            generation: self.generation,
        };

        return self.store.Current(&identity, self.generation).is_some();
    }

    /// One pass over the corpus.
    ///
    /// Materializes what is missing and reuses what is not. The second call over an
    /// unchanged corpus materializes nothing, because every key it computes is already
    /// held — which is the whole claim of a content-addressed fact key, made observable.
    ///
    /// # Panics
    ///
    /// If a fact would be written behind the generation it names. The generation only ever
    /// advances through [`Slice::Touch`], so reaching this is a contradiction in the run
    /// loop rather than a runtime condition — and a run that swallowed it would carry on
    /// with a store missing the fact it believes it just wrote.
    pub fn Run(&mut self, corpus: &Corpus) -> RunReport
    {
        let mut report = RunReport {
            files_seen: corpus.files.len(),
            ..RunReport::default()
        };

        for file in &corpus.files
        {
            let key = self.Syntax_Key(file);

            if self.Held(&key)
            {
                report.syntax_reused = report.syntax_reused.saturating_add(1);
                continue;
            }

            match rust::Materialize(
                file.subject,
                &file.source,
                rust::FactContext {
                    snapshot: self.pinned,
                    variant: self.variant,
                    configuration: self.configuration,
                    generation: self.generation,
                },
            )
            {
                rust::Materialization::Materialized(fact) =>
                {
                    // A leaf: computed from the file and from nothing else, so it declares
                    // no dependencies. This is also why the corpus needs the rollup — a
                    // graph of leaves has no descendants to get wrong.
                    self.store
                        .Materialize(*fact, &[])
                        .expect("a fact is never written behind the generation it names");
                    report.syntax_materialized = report.syntax_materialized.saturating_add(1);
                    report.recomputed.push(Recompute {
                        capability: rust::CAPABILITY.to_owned(),
                        subject: file.path.clone(),
                    });
                }
                rust::Materialization::Unparseable(failure) =>
                {
                    report.refused.push((file.path.clone(), failure.to_string()));
                }
            }
        }

        let groups = corpus.Groups();
        report.groups_seen = groups.len();

        for group in groups
        {
            let members = corpus.In_Group(&group);
            let Some(subject) = members.first().map(|member| return member.group_subject)
            else
            {
                continue;
            };
            let key = self.Surface_Key(subject, &members);

            if self.Held(&key)
            {
                report.surface_reused = report.surface_reused.saturating_add(1);
                continue;
            }

            let (surface, dependencies) = self.Roll_Up(&members);

            if surface.unreachable > 0
            {
                report.degraded.push(group.clone());
            }

            self.store
                .Materialize(
                    MaterializedFact {
                        identity: key.At(self.generation),
                        // No stronger than what it derived from. A count of verified facts
                        // is derived, and promoting it to Verified would launder the
                        // rollup's own arithmetic into a measurement.
                        evidence: EvidenceClass::Derived,
                        guarantee: surface::Declared_Guarantee(),
                        payload: FactPayload::New(surface::Payload_Schema(), surface.Encode()),
                    },
                    &dependencies,
                )
                .expect("a fact is never written behind the generation it names");

            report.surface_materialized = report.surface_materialized.saturating_add(1);
            report.recomputed.push(Recompute {
                capability: surface::CAPABILITY.to_owned(),
                subject: group,
            });
        }

        return report;
    }

    /// Reads every member's syntax fact through the registry and sums what they declare.
    ///
    /// The dependency edges come from [`Reader`] observing the reads, not from this
    /// function listing them. That distinction is the point: a hand-written edge list is a
    /// claim about what was read, and this is a record of it.
    fn Roll_Up(&self, members: &[&SourceFile]) -> (Surface, Vec<Dependency>)
    {
        // The floor a rollup needs, not a preference. Syntactic resolution is enough to
        // count declarations; soundness is not, because a rollup over facts that might
        // include items the files do not contain is a number about nothing.
        let need = Requirement::New(
            CapabilityId::New(rust::CAPABILITY),
            rust::CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        );

        let mut reader = Reader::On(&self.store, &self.registry, self.Context());
        let mut surface = Surface::default();

        for member in members
        {
            let read = reader.Require(
                &CapabilityId::New(rust::CAPABILITY),
                &member.subject,
                Self::Syntax_Inputs(&member.source),
                &need,
            );

            let Ok(fact) = read
            else
            {
                // The member has no readable fact — it was refused by the provider, or
                // nothing has computed it. Counted, never skipped: a rollup that silently
                // omits a member reports a smaller surface as if it were a complete one.
                surface.unreachable = surface.unreachable.saturating_add(1);
                continue;
            };

            match crate::surface::Public_Items(&fact.payload.bytes)
            {
                Ok((items, public)) =>
                {
                    surface.files = surface.files.saturating_add(1);
                    surface.items = surface.items.saturating_add(items);
                    surface.public = surface.public.saturating_add(public);
                }
                Err(_) => surface.unreachable = surface.unreachable.saturating_add(1),
            }
        }

        return (surface, reader.Into_Dependencies());
    }

    /// One edit, through the one door.
    ///
    /// There is deliberately no second way to advance this slice's generation. The change
    /// goes to [`Workspace::Apply`], the workspace says whether it turned out to be a
    /// change, and the generation the slice materializes into is the one the workspace
    /// produced — never a counter this crate incremented on its own. A composition root
    /// that kept its own counter beside the workspace's would be two answers to which
    /// analysis state the store is in.
    ///
    /// The corpus is rewritten to match, because it is the reading of the tree the
    /// providers actually parse. Letting the two drift apart is a real scenario — a change
    /// nobody announced — and it is modelled by calling [`Corpus::Rewrite`] alone.
    ///
    /// # Panics
    ///
    /// If the workspace refuses the change, or if the corpus does not hold `path`. A silent
    /// no-op on either would make an invalidation test assert that changing nothing
    /// invalidates nothing.
    pub fn Edit(
        &mut self,
        corpus: &mut Corpus,
        source: ChangeSource,
        path: &str,
        content: &str,
    ) -> Edited
    {
        let applied = self
            .workspace
            .Apply(&WorkspaceChangeSet::From(source).Present(path, content))
            .unwrap_or_else(|error| panic!("`{path}` could not be edited: {error}"));

        assert!(
            corpus.Rewrite(path, content),
            "`{path}` is not in the corpus, so this edit changed the workspace and nothing \
             the providers read"
        );

        let Applied::Advanced { generation, .. } = applied
        else
        {
            return Edited::Unchanged { applied };
        };

        self.generation = generation;

        let invalidated = self.store.Invalidate(
            &nomos_analysis::GenerationCause::SubjectChanged {
                subject: Subject_Of_Path(path),
                // A file changed, so the cause is file-granular. The engine broadens it to
                // whatever each affected provider can actually deliver, and records having
                // done so — the rollup will be broadened to Project.
                granularity: IncrementalGranularity::File,
            },
            self.generation,
        );

        return Edited::Advanced {
            applied,
            invalidated,
        };
    }

    /// The subjects named by a set of invalidated keys, resolved back to corpus paths.
    ///
    /// Keys carry digests, and an assertion that compares digests is an assertion nobody
    /// can read when it fails. This maps them back through the corpus so a failure says
    /// `alpha/one.rs` rather than thirty-two hex characters.
    #[must_use]
    pub fn Name_Keys(corpus: &Corpus, keys: &[FactKey]) -> Vec<String>
    {
        let mut named: Vec<String> = keys
            .iter()
            .map(|key| {
                let subject = corpus
                    .files
                    .iter()
                    .find(|file| return file.subject == key.subject)
                    .map(|file| return file.path.clone())
                    .or_else(|| {
                        return corpus
                            .files
                            .iter()
                            .find(|file| return file.group_subject == key.subject)
                            .map(|file| return file.group.clone());
                    })
                    .unwrap_or_else(|| return format!("<unknown subject {}>", key.subject));

                return format!("{} of {subject}", key.contract);
            })
            .collect();
        named.sort();

        return named;
    }
}
