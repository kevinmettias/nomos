//! Walking a capability's admitted offers until one of them answers.

// What a read depended on: the ordered trail, and one entry of it.
#[path = "reading/dependency.rs"] mod dependency;
#[path = "reading/trail.rs"] mod trail;

pub use dependency::Dependency;
pub(crate) use trail::Trail;

use nomos_capability::Requirement;
use nomos_capability::Selection;
use nomos_contracts::GenerationId;
use nomos_contracts::Applicability;
use nomos_capability::ProviderOffer;
use nomos_capability::Resolution;
use nomos_contracts::SubjectId;
use nomos_contracts::CapabilityId;
use nomos_capability::Registry;
use crate::FactError;
use crate::FactIdentity;
use crate::FactReader;
use crate::MaterializedFact;
use crate::InputDigest;
use crate::ReadOutcome;
use crate::FactKey;
use crate::Context;
use crate::MemoryFactStore;
pub struct Reader<'store, 'registry>
{
    store: &'store MemoryFactStore,
    registry: &'registry Registry,
    context: Context,
    trail: Trail,
}

impl<'store, 'registry> Reader<'store, 'registry>
{
    #[must_use]
    pub fn On(
        store: &'store MemoryFactStore,
        registry: &'registry Registry,
        context: Context,
    ) -> Self
    {
        return Self {
            store,
            registry,
            context,
            trail: Trail::New(),
        };
    }

    #[must_use]
    pub const fn Context(&self) -> &Context
    {
        return &self.context;
    }

    #[must_use]
    pub fn Into_Dependencies(self) -> Vec<Dependency>
    {
        return self.trail.Into_Dependencies();
    }

    fn Key_For(
        &self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        resolution: &Resolution,
    ) -> Option<FactKey>
    {
        return Some(self.Key_From(capability, subject, inputs, resolution.Offer()?));
    }

    /// The key one named offer's answer about a subject would be filed under.
    ///
    /// The provider and its guarantee are components of the key, which is the reason
    /// per-subject fallback is admissible at all: two providers answering about one file
    /// are two addresses rather than two values at one.
    fn Key_From(
        &self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        offer: &ProviderOffer,
    ) -> FactKey
    {
        use crate::GuaranteeDigest;

        return FactKey {
            contract: capability.clone(),
            contract_version: offer.version,
            subject: *subject,
            semantic_inputs: inputs,
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: self.context.variant,
            configuration: self.context.configuration,
        };
    }
}

/// What a walk of the admitted offers found: the first that answered, and every one before
/// it that did not.
struct Walked
{
    /// The key that answered, and where in the selection its offer sat.
    answered: Option<(FactKey, usize)>,
    /// The keys asked and not answered, in the order they were asked.
    missed: Vec<FactKey>,
}

impl Walked
{
    /// Writes every read this walk performed onto a trail, and says which one answered.
    ///
    /// The misses are written after the walk rather than during it, because recording takes
    /// the reader mutably and the candidates borrow the resolution. They are written either
    /// way: "the parser had nothing here" is a real read and a real dependency, and a fact
    /// that must be invalidated once the parser does have something needs that edge.
    fn Recorded_Into(self, trail: &mut Trail) -> Option<(FactKey, usize)>
    {
        for key in &self.missed
        {
            trail.Note_Miss(key);
        }

        let (key, rank) = self.answered?;
        trail.Note(&key, ReadOutcome::Materialized);

        return Some((key, rank));
    }
}

/// How good an answer is, given where in the selection it came from.
///
/// Only the chosen offer answers at the requirement's own applicability. Anything below it
/// is a fallback however good the offer is in itself, because the caller asked for a floor
/// and this answer came from under it.
const fn Answered_As(rank: usize, chosen: Applicability) -> Applicability
{
    if rank == 0
    {
        return chosen;
    }

    return Applicability::SupportedWithFallback;
}

impl Reader<'_, '_>
{
    /// What a key's read amounts to, before anything is decided about it.
    ///
    /// Superseded and absent are two answers. A fact that was here and was invalidated tells
    /// a caller its inputs moved; one that was never here tells it nobody has answered yet.
    fn Outcome_For(&self, key: &FactKey, at: GenerationId) -> ReadOutcome
    {
        if self.store.Lookup(key, at).is_ok()
        {
            return ReadOutcome::Materialized;
        }

        if self.store.Superseded_At(key).is_some()
        {
            return ReadOutcome::Superseded;
        }

        return ReadOutcome::Absent;
    }

    /// The fact behind a key that has just been shown to answer.
    ///
    /// The read is repeated rather than carried out of the walk, because the walk borrows
    /// the store immutably and recording the reads it made needs it mutably. Repeating a
    /// lookup is cheaper than threading a borrow through both.
    fn Answering(&self, key: &FactKey) -> Result<&MaterializedFact, Applicability>
    {
        return self
            .store
            .Lookup(key, self.context.generation)
            .map_err(|()| return Applicability::DependencyUnavailable);
    }

    /// Asks the chosen offer, then everything the floor also admitted, weakest last.
    fn Walk_Offers(
        &self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        selection: &Selection,
    ) -> Walked
    {
        let mut candidates: Vec<&ProviderOffer> = vec![&selection.chosen];
        candidates.extend(selection.Weaker());

        return self.Walk_Candidates(capability, subject, inputs, &candidates);
    }

    /// Asks each admitted offer in turn and stops at the first that answers.
    ///
    /// It stops rather than reading them all, because a weaker provider's answer is only
    /// wanted where the chosen one has none — asking on past an answer would make the
    /// selection order decorative.
    fn Walk_Candidates(
        &self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        candidates: &[&ProviderOffer],
    ) -> Walked
    {
        let at = self.context.generation;
        let mut walked = Walked {
            answered: None,
            missed: Vec::new(),
        };

        for (rank, offer) in candidates.iter().enumerate()
        {
            let key = self.Key_From(capability, subject, inputs, offer);
            if self.store.Lookup(&key, at).is_ok()
            {
                walked.answered = Some((key, rank));
                break;
            }
            walked.missed.push(key);
        }

        return walked;
    }
}

impl FactReader for Reader<'_, '_>
{
    fn Get(&mut self, identity: &FactIdentity) -> Result<&MaterializedFact, FactError>
    {
        let at = self.context.generation;
        let outcome = self.Outcome_For(&identity.key, at);
        self.trail.Note(&identity.key, outcome);

        let superseded = self.store.Superseded_At(&identity.key);
        if let Some(invalidated_at) = superseded
            && !outcome.Is_Answered()
        {
            return Err(FactError::Superseded {
                identity: Box::new(identity.clone()),
                invalidated_at,
            });
        }

        return self.store.Lookup(&identity.key, at).map_err(|()| {
            return FactError::Absent {
                identity: Box::new(identity.clone()),
            };
        });
    }

    fn Require(
        &mut self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        need: &Requirement,
    ) -> Result<&MaterializedFact, Applicability>
    {
        let resolution = self.registry.Resolve(need);
        let Some(key) = self.Key_For(capability, subject, inputs, &resolution)
        else
        {
            return Err(resolution.Applicability());
        };

        let at = self.context.generation;
        if self.store.Lookup(&key, at).is_err()
        {
            self.trail.Note_Miss(&key);

            return Err(Applicability::DependencyUnavailable);
        }

        self.trail.Note(&key, ReadOutcome::Materialized);

        return self
            .store
            .Lookup(&key, at)
            .map_err(|()| return Applicability::DependencyUnavailable);
    }

    fn Require_Any(
        &mut self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        need: &Requirement,
    ) -> Result<(&MaterializedFact, Applicability), Applicability>
    {
        let resolution = self.registry.Resolve(need);
        let Resolution::Satisfied {
            selection,
            applicability,
        } = &resolution
        else
        {
            return Err(resolution.Applicability());
        };

        let walked = self.Walk_Offers(capability, subject, inputs, selection);
        let Some((key, rank)) = walked.Recorded_Into(&mut self.trail)
        else
        {
            return Err(Applicability::DependencyUnavailable);
        };
        let answered_as = Answered_As(rank, *applicability);
        let fact = self.Answering(&key)?;

        return Ok((fact, answered_as));
    }

    fn Dependencies(&self) -> &[Dependency]
    {
        return self.trail.Recorded();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::GuaranteeDigest;
    use nomos_capability::Registry;
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, ContractVersion, Digest128, FactVariant,
        Guarantee, IncrementalGranularity, ProviderId, SnapshotId,
    };

    #[test]
    fn Test_On_Should_Build_A_Reader_With_An_Empty_Trail()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();

        let reader = Reader::On(&store, &registry, Sample_Context());

        assert!(reader.Into_Dependencies().is_empty());
    }

    #[test]
    fn Test_Context_Should_Return_The_Context_The_Reader_Was_Built_With()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let context = Sample_Context();

        let reader = Reader::On(&store, &registry, context.clone());

        assert_eq!(reader.Context(), &context);
    }

    #[test]
    fn Test_Into_Dependencies_Should_Report_Every_Read_A_Get_Recorded()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut reader = Reader::On(&store, &registry, Sample_Context());

        let identity = Sample_Key().At(Sample_Context().generation);
        assert!(reader.Get(&identity).is_err());

        assert_eq!(reader.Into_Dependencies().len(), 1);
    }

    fn Sample_Key() -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.reader"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(5)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            )),
            variant: Sample_Context().variant,
            configuration: Sample_Context().configuration,
        };
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Sample_Context() -> Context
    {
        return Context {
            snapshot: SnapshotId::From_Digest(Seeded(1)),
            variant: BuildVariantId::From_Digest(Seeded(2)),
            configuration: ConfigurationId::From_Digest(Seeded(3)),
            generation: GenerationId::From_Raw(4),
        };
    }
}
