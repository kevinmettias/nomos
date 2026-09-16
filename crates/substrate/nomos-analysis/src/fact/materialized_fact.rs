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

    /// The generation the sample fact was taken at, asserted again by
    /// `Test_Generation_Should_Return_The_Fact_Identitys_Own_Generation`.
    const SAMPLE_GENERATION: u64 = 5;

    /// The workspace snapshot the sample fact was measured against — a different value from
    /// the key components, so a fact read against the wrong tree is not silently the same.
    const SNAPSHOT_SEED: u8 = 2;

    /// The variant component of the sample key, seeded apart from the subject and the
    /// configuration so a key assembled with the wrong one still asserts unequal.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of the sample key; distinct from [`VARIANT_SEED`] for the
    /// reason given there.
    const CONFIGURATION_SEED: u8 = 4;

    fn Sample_Fact() -> MaterializedFact
    {
        return MaterializedFact {
            identity: Sample_Key().At(GenerationId::From_Raw(SAMPLE_GENERATION)),
            snapshot: SnapshotId::From_Digest(Seeded(SNAPSHOT_SEED)),
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
            variant: BuildVariantId::From_Digest(Seeded(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded(CONFIGURATION_SEED)),
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
        assert_eq!(Sample_Fact().Generation(), GenerationId::From_Raw(SAMPLE_GENERATION));
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
