//! The run: syntax facts per file, a rollup per directory, and what a change costs.

use crate::context::{Host_Variant, Resolved_Configuration};
use crate::corpus::{Corpus, SourceFile, Subject_Of_Path};
use crate::surface::{self, Surface};
use nomos_analysis::{
    Context, Dependency, FactIdentity, FactKey, FactPayload, FactReader, FactStore, GuaranteeDigest,
    InputDigest, InvalidationReport, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_cap_syntax as syntax;
use nomos_capability::{Registry, Requirement, Resolution, Selection};
use nomos_contracts::{
    Applicability, Assurance, BuildVariantId, CapabilityId, ConfigurationId, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SnapshotId, SubjectId,
};
use nomos_lang_rust as rust;
use nomos_lang_rust_scan as scan;
use nomos_model::Digest_Of_Parts;
use nomos_workspace::{Applied, ChangeSource, Workspace, WorkspaceChangeSet};
use std::collections::BTreeSet;

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
    /// Files no admitted provider could answer for, by path and reason. Named, not counted.
    ///
    /// The reason is the *chosen* provider's, because that is the refusal a caller can act
    /// on. Under a floor that admits nobody weaker this is every refusal there is.
    pub refused: Vec<(String, String)>,
    /// How many subjects each provider answered for, by provider name.
    ///
    /// The census `OD-CAPABILITY-003` requires. Without it a run that fell back for one
    /// file in six is indistinguishable from one the parser answered whole, and the point
    /// of admitting fallback was to buy coverage visibly rather than quietly.
    pub answered_by: std::collections::BTreeMap<String, usize>,
    /// Subjects the chosen provider refused and a weaker one answered, with who answered.
    ///
    /// Named rather than counted, for the same reason `refused` is: "one file was
    /// approximated" is satisfied by approximating the wrong one.
    pub fell_back: Vec<(String, String)>,
    /// Groups whose rollup read at least one fallback answer.
    ///
    /// Distinct from `degraded`, and the distinction is the whole of what fallback buys and
    /// costs. A degraded rollup could not read a member at all; an approximated one read
    /// every member and one of them came from a weaker provider than the run asked for.
    pub approximated: Vec<String>,
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

    /// How many subjects one provider answered for in this pass.
    #[must_use]
    pub fn Answered_By(&self, provider: &str) -> usize
    {
        return self.answered_by.get(provider).copied().unwrap_or(0);
    }

    /// Whether this pass got every answer from the provider the registry chose.
    ///
    /// The question a caller asks before treating the run as exact. A run that fell back is
    /// not a clean run — it is a run that bought coverage, and it says what that cost.
    #[must_use]
    pub fn Wholly_Chosen(&self) -> bool
    {
        return self.fell_back.is_empty();
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

/// What a caller needs when it needs the file actually parsed.
///
/// Syntactic resolution is enough to count declarations, and soundness is not optional: a
/// rollup over facts that might include items the files do not contain is a number about
/// nothing. This is the floor every run used before there was anything else to resolve to,
/// and it is still the default.
#[must_use]
pub const fn Parsed_Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// What a caller needs when it would rather have a weak answer than none.
///
/// Below the parser on every axis it can be below, so both offers clear it. That is what
/// makes a *preference* meaningful: with two usable providers, which one answers is a
/// choice rather than a consequence.
#[must_use]
pub const fn Approximate_Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// Which provider the registry chose, and how far its answer can be stood behind.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
pub struct Resolved
{
    pub selection: Selection,
    pub applicability: Applicability,
}

/// What a rollup produced: the summed surface, and the reads it was derived from.
///
/// Named rather than a pair. The dependency edges are a record of what was read, not a
/// second description of the surface, and a name is what keeps the two apart at the call
/// site.
struct RolledUp
{
    surface: Surface,
    dependencies: Vec<Dependency>,
}

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
            .Declare(syntax::Capability_Contract())
            .expect("the syntax capability is declared once");
        registry
            .Offer(rust::Provider_Offer())
            .expect("the Rust provider's offer is within its capability's ceiling");
        // The second offer against the same contract. It is accepted because the ceiling
        // bounds what may be *claimed*, not how weak an offer may be — a provider that
        // promises less than the ceiling is exactly what a ceiling is for.
        registry
            .Offer(scan::Provider_Offer())
            .expect("the scanner's offer is within the same ceiling");
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


    fn Context(&self) -> Context
    {
        return Context {
            // The workspace as it is now. Not a key component — see OD-ANALYSIS-001 — so
            // it moves freely with the workspace without re-addressing a single fact.
            snapshot: self.snapshot,
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

    /// Asks whichever provider the registry resolved.
    ///
    /// # Why the composition root holds the table and not the registry
    ///
    /// A `ProviderOffer` is a claim, not a function pointer, and deliberately so:
    /// `nomos-capability` sits below every provider and must not know what any of them can
    /// be called. So somebody has to turn a resolved `ProviderId` into a call, and the only
    /// thing that legitimately knows every provider is the composition root — the same
    /// place that registered them.
    ///
    /// The `unreachable` arm is the cost of that split, and it is a real one: a provider
    /// registered and not dispatched would resolve and then fail to answer. It is written
    /// as a panic naming the provider rather than as a silent skip, because a run that
    /// quietly produced no facts for a provider it selected would report a clean corpus it
    /// never read.
    ///
    /// # Panics
    ///
    /// If the resolved provider has no dispatch here.
    fn Dispatch(&self, file: &SourceFile, offer: &nomos_capability::ProviderOffer)
        -> rust::Materialization
    {
        let provider = offer.provider.As_Str();

        if provider == rust::PROVIDER
        {
            return rust::Materialize(
                file.subject,
                &file.source,
                rust::FactContext {
                    snapshot: self.snapshot,
                    variant: self.variant,
                    configuration: self.configuration,
                    generation: self.generation,
                },
            );
        }

        if provider == scan::PROVIDER
        {
            // A scan has no refusal case, so this arm cannot produce `Unparseable`. That
            // asymmetry is the interesting half of having two providers: they do not
            // merely differ in how good the answer is, they differ in which inputs they
            // can answer for at all.
            return rust::Materialization::Materialized(Box::new(scan::Materialize(
                file.subject,
                &file.source,
                scan::FactContext {
                    snapshot: self.snapshot,
                    variant: self.variant,
                    configuration: self.configuration,
                    generation: self.generation,
                },
            )));
        }

        panic!("{provider} was resolved and this composition cannot call it");
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

        let candidates = self.Candidates();

        for file in &corpus.files
        {
            self.Answer_For(file, &candidates, &mut report);
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

            let RolledUp {
                surface,
                dependencies,
            } = self.Roll_Up(&members);

            if surface.unreachable > 0
            {
                report.degraded.push(group.clone());
            }
            if surface.approximate > 0
            {
                report.approximated.push(group.clone());
            }

            self.store
                .Materialize(
                    MaterializedFact {
                        identity: key.At(self.generation),
                        // Provenance: the tree this rollup was computed over. Not part of
                        // the key, so the workspace moving re-addresses nothing.
                        snapshot: self.snapshot,
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

    /// Walks the selection for one file until something answers.
    ///
    /// The loop `P8-SELECTION` made possible and nothing wrote. The chosen offer is asked
    /// first; if it refuses the file, the next admitted offer is asked, and so on down the
    /// ranking the registry produced. The first answer is written under *its own* provider's
    /// key, so the store never holds a fact labelled with a provider that refused it.
    ///
    /// A refusal is not cached. The parser is asked again on the next pass over the same
    /// file and refuses again, which is cheap and is the honest reading — nothing has
    /// recorded that the parser cannot answer, only that it did not.
    ///
    /// # Panics
    ///
    /// If a fact would be written behind the generation it names, which is a contradiction
    /// in the run loop rather than a runtime condition.
    fn Answer_For(
        &mut self,
        file: &SourceFile,
        candidates: &[nomos_capability::ProviderOffer],
        report: &mut RunReport,
    )
    {
        let mut refusal = None;

        for (rank, offer) in candidates.iter().enumerate()
        {
            let key = self.Syntax_Key_Of(file, offer);
            let provider = offer.provider.As_Str().to_owned();

            if self.Held(&key)
            {
                report.syntax_reused = report.syntax_reused.saturating_add(1);
                Self::Credit(report, &provider, rank, file);

                return;
            }

            match self.Dispatch(file, offer)
            {
                rust::Materialization::Materialized(fact) =>
                {
                    self.Kept(*fact, report, &provider, rank, file);

                    return;
                }
                rust::Materialization::Unparseable(failure) =>
                {
                    // The chosen provider's refusal is the one a caller can act on, so it
                    // is the one kept when nobody below can answer either.
                    if refusal.is_none()
                    {
                        refusal = Some(failure.to_string());
                    }
                }
            }
        }

        report.refused.push((
            file.path.clone(),
            refusal.unwrap_or_else(|| return "no admitted provider answered".to_owned()),
        ));
    }

    /// Records who answered for a subject, and whether that was the chosen offer.
    /// One provider's answer, stored and counted.
    ///
    /// A leaf: computed from the file and from nothing else, so it declares no
    /// dependencies. This is also why the corpus needs the rollup — a graph of leaves has
    /// no descendants to get wrong.
    fn Kept(
        &mut self,
        fact: MaterializedFact,
        report: &mut RunReport,
        provider: &str,
        rank: usize,
        file: &SourceFile,
    )
    {
        self.store
            .Materialize(fact, &[])
            .expect("a fact is never written behind the generation it names");

        report.syntax_materialized = report.syntax_materialized.saturating_add(1);
        report.recomputed.push(Recompute {
            capability: syntax::CAPABILITY.to_owned(),
            subject: file.path.clone(),
        });
        Self::Credit(report, provider, rank, file);
    }

    fn Credit(report: &mut RunReport, provider: &str, rank: usize, file: &SourceFile)
    {
        let counted = report.answered_by.entry(provider.to_owned()).or_insert(0);
        *counted = counted.saturating_add(1);

        if rank > 0
        {
            report.fell_back.push((file.path.clone(), provider.to_owned()));
        }
    }

    /// Reads every member's syntax fact through the registry and sums what they declare.
    ///
    /// The dependency edges come from [`Reader`] observing the reads, not from this
    /// function listing them. That distinction is the point: a hand-written edge list is a
    /// claim about what was read, and this is a record of it.
    fn Roll_Up(&self, members: &[&SourceFile]) -> RolledUp
    {
        // The run's own requirement, not a second one written here.
        //
        // It was a separate literal until there were two providers, and that was a latent
        // defect rather than a duplication: the rollup looks facts up by rebuilding their
        // key, and a key names the provider that answered. A rollup asking for a floor the
        // run did not ask for would resolve a different provider, rebuild a key nobody
        // wrote, and report every member unreachable — loudly, but for the wrong reason.
        let need = self.Requirement();

        let mut reader = Reader::On(&self.store, &self.registry, self.Context());
        let mut surface = Surface::default();

        for member in members
        {
            // `Require_Any` rather than `Require`, which is what makes the coverage a
            // lowered floor bought reachable by the thing that derives from it. Reading
            // only the chosen provider would leave the scanner's answer written into the
            // store and unread, and the rollup would still report the member missing —
            // the same defect one level in.
            let read = reader.Require_Any(
                &CapabilityId::New(syntax::CAPABILITY),
                &member.subject,
                Self::Syntax_Inputs(&member.source),
                &need,
            );

            let Ok((fact, applicability)) = read
            else
            {
                // The member has no readable fact — every admitted provider refused it, or
                // nothing has computed it. Counted, never skipped: a rollup that silently
                // omits a member reports a smaller surface as if it were a complete one.
                surface.unreachable = surface.unreachable.saturating_add(1);
                continue;
            };

            let approximated = applicability == Applicability::SupportedWithFallback;

            match crate::surface::Public_Items(&fact.payload.bytes)
            {
                Ok((items, public)) => Self::Summed(&mut surface, items, public, approximated),
                Err(_) => surface.unreachable = surface.unreachable.saturating_add(1),
            }
        }

        return RolledUp {
            surface,
            dependencies: reader.Into_Dependencies(),
        };
    }

    /// One readable member folded into the rollup.
    ///
    /// `approximated` is counted here rather than left to the caller because of
    /// `OD-CAPABILITY-003`'s third condition: without it the scanner's answer covers the
    /// file the parser refused, `unreachable` drops to zero, and a corpus that was visibly
    /// incomplete starts reading as complete and sound.
    fn Summed(surface: &mut Surface, items: u32, public: u32, approximated: bool)
    {
        surface.files = surface.files.saturating_add(1);
        surface.items = surface.items.saturating_add(items);
        surface.public = surface.public.saturating_add(public);

        if approximated
        {
            surface.approximate = surface.approximate.saturating_add(1);
        }
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
        self.snapshot = applied.Snapshot();

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
        let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
        for (path, content) in landing
        {
            checkout = checkout.Present(*path, *content);
        }

        let applied = self
            .workspace
            .Apply(&checkout)
            .unwrap_or_else(|error| panic!("the checkout was refused: {error}"));

        for (path, content) in landing
        {
            assert!(
                corpus.Rewrite(path, content),
                "`{path}` is not in the corpus, so this checkout changed the workspace and \
                 nothing the providers read"
            );
        }

        let Applied::Advanced { .. } = applied
        else
        {
            return Edited::Unchanged { applied };
        };

        let differing: BTreeSet<SubjectId> = applied
            .Effects()
            .iter()
            .filter(|effect| return effect.Altered())
            .map(|effect| return Subject_Of_Path(effect.Path()))
            .collect();

        self.generation = applied.Generation();
        self.snapshot = applied.Snapshot();

        let invalidated = self.store.Invalidate(
            &nomos_analysis::GenerationCause::SnapshotReplaced {
                from,
                to: applied.Snapshot(),
                differing,
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
