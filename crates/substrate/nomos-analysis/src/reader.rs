use crate::fact::{FactError, MaterializedFact};
use crate::identity::{FactIdentity, FactKey, GuaranteeDigest, InputDigest};
use crate::store::MemoryFactStore;
use nomos_capability::{Registry, Requirement, Resolution};
use nomos_contracts::{
    Applicability, BuildVariantId, CapabilityId, ConfigurationId, GenerationId, SnapshotId,
    SubjectId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadOutcome
{
    Materialized,
    Absent,
    Superseded,
    Degraded(Applicability),
}

impl ReadOutcome
{
    #[must_use]
    pub const fn Answered(self) -> bool
    {
        return matches!(self, Self::Materialized);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency
{
    pub key: FactKey,
    pub outcome: ReadOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

pub trait FactReader
{
    fn Get(&mut self, identity: &FactIdentity) -> Result<&MaterializedFact, FactError>;

    fn Require(
        &mut self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        need: &Requirement,
    ) -> Result<&MaterializedFact, Applicability>;

    fn Dependencies(&self) -> &[Dependency];
}

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
        let offer = resolution.Offer()?;

        return Some(FactKey {
            contract: capability.clone(),
            contract_version: offer.version,
            subject: *subject,
            semantic_inputs: inputs,
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            snapshot: self.context.snapshot,
            variant: self.context.variant,
            configuration: self.context.configuration,
        });
    }
}

impl FactReader for Reader<'_, '_>
{
    fn Get(&mut self, identity: &FactIdentity) -> Result<&MaterializedFact, FactError>
    {
        let at = self.context.generation;
        let outcome = match self.store.Lookup(&identity.key, at)
        {
            Ok(_) => ReadOutcome::Materialized,
            Err(()) if self.store.Superseded_At(&identity.key).is_some() => ReadOutcome::Superseded,
            Err(()) => ReadOutcome::Absent,
        };
        self.Record(&identity.key, outcome);

        if let Some(invalidated_at) = self.store.Superseded_At(&identity.key)
        {
            if !outcome.Answered()
            {
                return Err(FactError::Superseded {
                    identity: Box::new(identity.clone()),
                    invalidated_at,
                });
            }
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

    fn Dependencies(&self) -> &[Dependency]
    {
        return &self.recorded;
    }
}
