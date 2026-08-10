//! Walking a capability's admitted offers until one of them answers.

use nomos_capability::Requirement;
use nomos_capability::Selection;
use nomos_contracts::GenerationId;
use nomos_contracts::Applicability;
use nomos_capability::ProviderOffer;
use nomos_capability::Resolution;
use nomos_contracts::SubjectId;
use nomos_contracts::CapabilityId;
use nomos_capability::Registry;
use crate::fact_error::FactError;
use crate::fact_identity::FactIdentity;
use crate::fact_reader::FactReader;
use crate::materialized_fact::MaterializedFact;
use crate::guarantee_digest::GuaranteeDigest;
use crate::identity::InputDigest;
use crate::reader::ReadOutcome;
use crate::fact_key::FactKey;
use crate::dependency::Dependency;
use crate::context::Context;
use crate::memory_fact_store::MemoryFactStore;
pub struct Reader<'store, 'registry>
{
    store: &'store MemoryFactStore,
    registry: &'registry Registry,
    context: Context,
    recorded: Vec<Dependency>,
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
            recorded: Vec::new(),
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
        return self.recorded;
    }

    fn Record(&mut self, key: &FactKey, outcome: ReadOutcome)
    {
        self.recorded.push(Dependency {
            key: key.clone(),
            outcome,
        });
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

    /// Records every read the walk performed, and says which one answered.
    ///
    /// The misses are recorded after the walk rather than during it, because recording takes
    /// `&mut self` and the candidates borrow the resolution. They are recorded either way:
    /// "the parser had nothing here" is a real read and a real dependency, and a rollup that
    /// later has to be invalidated when the parser does have something needs the edge.
    fn Record_Walk(&mut self, walked: Walked) -> Option<(FactKey, usize)>
    {
        for key in &walked.missed
        {
            self.Record(key, ReadOutcome::Degraded(Applicability::DependencyUnavailable));
        }

        let (key, rank) = walked.answered?;
        self.Record(&key, ReadOutcome::Materialized);

        return Some((key, rank));
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
        self.Record(&identity.key, outcome);

        let superseded = self.store.Superseded_At(&identity.key);
        if let Some(invalidated_at) = superseded
            && !outcome.Answered()
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
            self.Record(&key, ReadOutcome::Degraded(Applicability::DependencyUnavailable));

            return Err(Applicability::DependencyUnavailable);
        }

        self.Record(&key, ReadOutcome::Materialized);

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
        let Some((key, rank)) = self.Record_Walk(walked)
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
        return &self.recorded;
    }
}
