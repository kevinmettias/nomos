//! A fact as it sits in the store: what it says and what produced it.

use nomos_contracts::GenerationId;
use nomos_contracts::Guarantee;
use nomos_contracts::EvidenceClass;
use nomos_contracts::SnapshotId;
use crate::FactKey;
use crate::FactPayload;
use crate::FactIdentity;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedFact
{
    pub identity: FactIdentity,
    /// The workspace state this fact was measured against.
    ///
    /// Provenance, not identity. It is deliberately not a component of [`FactKey`] — see
    /// [`crate::Component`] and `docs/records/OD-ANALYSIS-001` — because a workspace
    /// snapshot is a digest over every member, so keying a fact on one makes editing any
    /// file re-address every fact in the corpus.
    ///
    /// What it is here for is the question a key cannot answer: *which tree was this read
    /// from*. A fact that outlives several workspace states keeps naming the first one it
    /// was measured against, which is correct — it is a record of an observation, and the
    /// observation happened once. Re-stamping it on reuse would be a claim that the
    /// measurement was repeated.
    pub snapshot: SnapshotId,
    pub evidence: EvidenceClass,
    pub guarantee: Guarantee,
    pub payload: FactPayload,
}

impl MaterializedFact
{
    #[must_use]
    pub const fn Key(&self) -> &FactKey
    {
        return self.identity.Key();
    }

    #[must_use]
    pub const fn Generation(&self) -> GenerationId
    {
        return self.identity.generation;
    }
}
