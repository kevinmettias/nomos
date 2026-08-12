//! The run: syntax facts per file, a rollup per directory, and what a change costs.

use crate::{Host_Variant, Resolved_Configuration};
use crate::{Corpus, Subject_Of_Path};
use crate::Parsed_Floor;
use crate::surface::{self, Surface};
use crate::{Edited, Recompute, Resolved, RunReport, SourceFile};
use nomos_analysis::{
    Context, Dependency, FactIdentity, FactKey, FactPayload, FactReader, FactStore, GuaranteeDigest,
    InputDigest, InvalidationReport, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_cap_syntax as syntax;
use nomos_capability::{Registry, Requirement, Resolution};
use nomos_contracts::{
    Applicability, BuildVariantId, CapabilityId, ConfigurationId, Digest128,
    EvidenceClass, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SnapshotId, SubjectId,
};
use nomos_lang_rust as rust;
use nomos_lang_rust_scan as scan;
use nomos_model::Digest_Of_Parts;
use nomos_workspace::{Applied, ChangeSource, Workspace, WorkspaceChangeSet};
use std::collections::BTreeSet;

/// The composed system: a workspace that says what is there, a registry that resolves
/// providers, and a store that holds what they produced.
pub struct Slice
{
    store: MemoryFactStore,
    registry: Registry,
    workspace: Workspace,
    /// What the workspace currently is, as the door last reported it.
    ///
    /// Held rather than asked for, and the difference matters at scale: `Workspace::Id` is
    /// a digest over every member, so re-deriving it per fact key re-encodes the whole tree
    /// once per file. Over the scale corpus that is 876 KB rebuilt eleven thousand times,
    /// and it cost seventy seconds of a nine-second suite when this field was not here.
    ///
    /// It is not a pin. Every value it takes comes from an [`Applied`] the workspace itself
    /// produced, and it is updated at all three places a workspace can change — ingestion,
    /// an edit, a checkout — so `Slice::Snapshot` and `Workspace::Id` never disagree.
    /// `Test_The_Fact_Context_Should_Come_From_The_Workspace` asserts they do not.
    snapshot: SnapshotId,
    /// Taken from the workspace once. Neither can change — [`Workspace`] has no door for
    /// them — so recomputing a digest per fact key would buy nothing.
    variant: BuildVariantId,
    configuration: ConfigurationId,
    generation: GenerationId,
    /// The weakest syntax answer this run will accept.
    ///
    /// A floor rather than a choice of provider. With two offers on the table the run says
    /// what it needs and the registry says who can serve it — a composition root that named
    /// a provider directly would be deciding the thing the registry exists to decide, and
    /// the two would disagree the day a third provider arrived.
    floor: Guarantee,
    /// A provider this run would rather have, if it can serve the floor.
    ///
    /// A name, not a second guarantee. Whether it was honoured is what makes
    /// [`Applicability::SupportedWithFallback`] a fact about which provider answered rather
    /// than a judgement about how good the answer was.
    preferred: Option<ProviderId>,
}

mod dispatch;
mod edit;
mod keys;
mod rollup;

impl Slice
{

    /// A slice over an empty workspace.
    ///
    /// The context is real and derived even here: the variant is what this binary was
    /// compiled as, the configuration is the composition that was just built, and the
    /// snapshot is the identity of a workspace holding nothing — which is a state, and is
    /// not the same state as one holding a file.
    #[must_use]
    pub fn Composed() -> Self
    {
        let registry = crate::Registered();
        let workspace = Workspace::Empty(Host_Variant(), Resolved_Configuration(&registry));

        return Self {
            store: MemoryFactStore::New(),
            snapshot: workspace.Id(),
            variant: workspace.Snapshot().Variant().Id(),
            configuration: workspace.Snapshot().Configuration(),
            generation: workspace.Generation(),
            workspace,
            registry,
            floor: Parsed_Floor(),
            preferred: None,
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
        slice.snapshot = applied.Snapshot();

        return slice;
    }

    /// Runs against a different floor.
    ///
    /// The whole point of two offers: a caller that will take an approximation gets an
    /// answer for files a parser refuses, and one that needs a parse does not.
    #[must_use]
    pub fn Accepting(mut self, floor: Guarantee) -> Self
    {
        self.floor = floor;
        return self;
    }

    /// Names a provider this run would rather have.
    ///
    /// A preference is not a floor. If the named provider cannot serve the floor, the
    /// registry serves one that can and reports [`Applicability::SupportedWithFallback`] —
    /// the answer still stands, and its provenance is not what was asked for.
    #[must_use]
    pub fn Preferring(mut self, provider: &str) -> Self
    {
        self.preferred = Some(ProviderId::New(provider));
        return self;
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

    /// What the workspace is, as the door last reported it. Always `Workspace().Id()`.
    #[must_use]
    pub const fn Snapshot(&self) -> SnapshotId
    {
        return self.snapshot;
    }

    /// What this run needs from a syntax provider.
    ///
    /// Held rather than hard-coded, because with two providers offering
    /// `nomos.cap.syntax.items` the floor is what decides which one answers — and the run's
    /// results are only comparable between two runs that asked for the same thing.
    #[must_use]
    pub fn Requirement(&self) -> Requirement
    {
        let need = Requirement::New(
            CapabilityId::New(syntax::CAPABILITY),
            syntax::CONTRACT_VERSION,
            self.floor,
        );

        return match &self.preferred
        {
            Some(provider) => need.Preferring(provider.clone()),
            None => need,
        };
    }

    /// Which provider answers, what it was chosen over, and how the registry describes the
    /// choice.
    ///
    /// A whole [`Selection`] rather than the winning offer, because the winner alone cannot
    /// answer the question this run has to be able to ask: whether lowering the floor
    /// bought anything. `Selection::Weaker` is what the lowered floor bought and the chosen
    /// offer is what it did *not* cost.
    ///
    /// # Panics
    ///
    /// If nothing satisfies the floor. The alternative would be a run that materialized no
    /// facts and reported a clean corpus, which is the single most repeated defect in the
    /// prototype: a check that could not run reading like a check that found nothing.
    #[must_use]
    pub fn Resolved(&self) -> Resolved
    {
        let resolution = self.registry.Resolve(&self.Requirement());

        let Resolution::Satisfied {
            selection,
            applicability,
        } = resolution
        else
        {
            panic!("no provider offers {} at this run's floor: {resolution:?}", syntax::CAPABILITY)
        };

        return Resolved {
            selection,
            applicability,
        };
    }

    /// The key a syntax fact about this file is filed under.
    ///
    /// # Why the provider is resolved rather than named
    ///
    /// It used to be `rust::PROVIDER`, written into the key by the composition root. That
    /// compiled and was right for as long as there was one provider, and it was a
    /// composition root deciding what the registry is for deciding. With two offers it
    /// would file the scanner's answer under the parser's name, and a store holding facts
    /// mislabelled with their producer is worse than one holding none.
    ///
    /// Public so that a test can compare one file's identity across two workspace states.
    /// That comparison is what closed OD-ANALYSIS-001 — the two keys used to differ and now
    /// agree — and it cannot be made from outside without this.
    #[must_use]
    pub fn Syntax_Key(&self, file: &SourceFile) -> FactKey
    {
        return self.Syntax_Key_Of(file, &self.Resolved().selection.chosen);
    }

    /// The key one named offer's answer about a file would be filed under.
    ///
    /// The same construction as [`Slice::Syntax_Key`] with the offer supplied rather than
    /// resolved, because under per-subject fallback the offer that answered is not always
    /// the offer the registry chose — and the key has to name whoever actually answered or
    /// the store is holding a fact labelled with a provider that refused it.
    #[must_use]
    pub fn Syntax_Key_Of(&self, file: &SourceFile, offer: &nomos_capability::ProviderOffer)
        -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New(syntax::CAPABILITY),
            contract_version: offer.version,
            subject: file.subject,
            semantic_inputs: Self::Syntax_Inputs(&file.source),
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: self.variant,
            configuration: self.configuration,
        };
    }

    /// The offers this run may ask, chosen first and then weakest-last.
    ///
    /// Exactly `chosen` followed by `Selection::Weaker()`, in the registry's order. The
    /// composition root does not rank and does not filter: `nomos-capability` already
    /// decided both, and a second ordering here would be a second answer to
    /// `OD-CAPABILITY-001`.
    ///
    /// Under [`Parsed_Floor`] this is one offer, because nothing weaker cleared the floor —
    /// so a run that asked for a parse gets a parse or nothing, unchanged.
    #[must_use]
    pub fn Candidates(&self) -> Vec<nomos_capability::ProviderOffer>
    {
        let selection = self.Resolved().selection;
        let mut offers = vec![selection.chosen.clone()];
        offers.extend(selection.Weaker().into_iter().cloned());

        return offers;
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
        let candidates = self.Candidates();
        for file in &corpus.files
        {
            self.Answer_For(file, &candidates, &mut report);
        }

        let groups = corpus.Groups();
        report.groups_seen = groups.len();
        for group in groups
        {
            self.Surface_For(corpus, group, &mut report);
        }

        return report;
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
        let presented = WorkspaceChangeSet::From(source).Present(path, content);
        let applied = self
            .workspace
            .Apply(&presented)
            .unwrap_or_else(|error| panic!("`{path}` could not be edited: {error}"));

        Self::Assert_The_Corpus_Holds(corpus, path, content);

        let Applied::Advanced { generation, .. } = applied
        else
        {
            return Edited::Unchanged { applied };
        };
        self.generation = generation;
        self.snapshot = applied.Snapshot();

        let invalidated = self.Invalidate_One_Subject(path);

        return Edited::Advanced {
            applied,
            invalidated,
        };
    }

    /// A checkout: several members land at once, and the store is told which of them differ.
    ///
    /// The mechanism is [`Slice::Edit`]'s — one change set, one generation. What differs is
    /// the cause. An edit is one subject changing. A checkout replaces the workspace state
    /// wholesale, and the store needs the set of members that are not the same in both.
    ///
    /// That set is not derived here. [`Workspace::Apply`] already reports an [`Effect`] per
    /// path and already knows which of them altered anything, so the differing set is read
    /// off what the workspace said rather than recomputed against it. A second computation
    /// of the same thing is a second answer waiting to disagree.
    ///
    /// # Panics
    ///
    /// If the workspace refuses the set, or if the corpus does not hold one of the paths.
    pub fn Checkout(&mut self, corpus: &mut Corpus, landing: &[(&str, &str)]) -> Edited
    {
        let from = self.snapshot;
        let applied = self.Land(landing);

        Self::Assert_The_Corpus_Holds_Each(corpus, landing);

        let Applied::Advanced { .. } = applied
        else
        {
            return Edited::Unchanged { applied };
        };
        let differing = Self::Differing_Members(&applied);
        self.generation = applied.Generation();
        self.snapshot = applied.Snapshot();

        let invalidated = self.Invalidate_The_Whole_Tree(from, applied.Snapshot(), differing);

        return Edited::Advanced {
            applied,
            invalidated,
        };
    }

    /// The four context fields, read as the one thing they are.
    ///
    /// Here rather than beside the key builders, although every caller is one. A context is
    /// not a key component — `snapshot` moves with the workspace and re-addresses nothing,
    /// which is what OD-ANALYSIS-001 settled — so `keys.rs` was the wrong home for the only
    /// method that reads all four of these fields together. Beside the fields it reads, the
    /// file shows what the type shares: the state the run advances and the context a fact is
    /// keyed against are the same four values, held once.
    pub(super) fn Context(&self) -> Context
    {
        return Context {
            snapshot: self.snapshot,
            variant: self.variant,
            configuration: self.configuration,
            generation: self.generation,
        };
    }
}
