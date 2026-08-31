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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::GuaranteeDigest;
    use crate::InputDigest;
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
        FactVariant, IncrementalGranularity, ProviderId, SchemaId, SubjectId,
    };

    fn Sample_Fact() -> MaterializedFact
    {
        return MaterializedFact {
            identity: Sample_Key().At(GenerationId::From_Raw(5)),
            snapshot: SnapshotId::From_Digest(Seeded(2)),
            evidence: EvidenceClass::Derived,
            guarantee: Sample_Guarantee(),
            payload: FactPayload::New(SchemaId::New("nomos.test.materialized.v1"), b"tree".to_vec()),
        };
    }

    fn Sample_Key() -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.materialized"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(1)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Sample_Guarantee()),
            variant: BuildVariantId::From_Digest(Seeded(3)),
            configuration: ConfigurationId::From_Digest(Seeded(4)),
        };
    }

    #[test]
    fn Test_Key_Should_Return_The_Fact_Identitys_Own_Key()
    {
        assert_eq!(Sample_Fact().Key(), &Sample_Key());
    }

    #[test]
    fn Test_Generation_Should_Return_The_Fact_Identitys_Own_Generation()
    {
        assert_eq!(Sample_Fact().Generation(), GenerationId::From_Raw(5));
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Sample_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );
    }
}
