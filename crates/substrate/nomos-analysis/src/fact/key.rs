//! What a fact is about, before anything has been said about it.

use nomos_contracts::GenerationId;
use nomos_contracts::Digest128;
use nomos_contracts::ConfigurationId;
use nomos_contracts::BuildVariantId;
use nomos_contracts::ProviderId;
use nomos_contracts::SubjectId;
use nomos_contracts::ContractVersion;
use nomos_contracts::CapabilityId;
use crate::FactIdentity;
use crate::GuaranteeDigest;
use crate::InputDigest;
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key
{
    pub contract: CapabilityId,
    pub contract_version: ContractVersion,
    pub subject: SubjectId,
    pub semantic_inputs: InputDigest,
    pub provider: ProviderId,
    pub provider_version: ContractVersion,
    pub guarantee: GuaranteeDigest,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
}

impl Key
{
    #[must_use]
    pub fn Parts(&self) -> Vec<Vec<u8>>
    {
        return vec![
            self.contract.As_Str().as_bytes().to_vec(),
            Version_Bytes(self.contract_version).to_vec(),
            self.subject.Digest().Bytes().to_vec(),
            self.semantic_inputs.Digest().Bytes().to_vec(),
            self.provider.As_Str().as_bytes().to_vec(),
            Version_Bytes(self.provider_version).to_vec(),
            self.guarantee.Digest().Bytes().to_vec(),
            self.variant.Digest().Bytes().to_vec(),
            self.configuration.Digest().Bytes().to_vec(),
        ];
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        use nomos_model::Digest_Of_Parts;

        let parts = self.Parts();
        let borrowed: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();

        return Digest_Of_Parts(&borrowed);
    }

    #[must_use]
    pub const fn At(self, generation: GenerationId) -> FactIdentity
    {
        return FactIdentity {
            key: self,
            generation,
        };
    }
}

/// A contract version is two 16-bit numbers, so four bytes carry it whole.
const VERSION_BYTES: usize = 4;

const fn Version_Bytes(version: ContractVersion) -> [u8; VERSION_BYTES]
{
    let major = version.major.to_be_bytes();
    let minor = version.minor.to_be_bytes();

    return [major[0], major[1], minor[0], minor[1]];
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    /// One part per field of [`Key`], in the order `Parts` lists them.
    const KEY_PART_COUNT: usize = 9;

    /// The generation the sample key is paired with. One named value, because these tests
    /// compare an identity against the generation it was built at and never across two.
    const SAMPLE_GENERATION: u64 = 7;

    /// The variant component of the sample key, seeded apart from the subject and the
    /// configuration so a key assembled with the wrong one still asserts unequal.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of the sample key; distinct from [`VARIANT_SEED`] for the
    /// reason given there.
    const CONFIGURATION_SEED: u8 = 4;

    #[test]
    fn Test_Parts_Should_Carry_Every_Field_As_A_Non_Empty_Byte_Vector()
    {
        let parts = Sample().Parts();

        assert_eq!(parts.len(), KEY_PART_COUNT);
        assert!(parts.iter().all(|part| !part.is_empty()));
    }

    #[test]
    fn Test_Digest_Should_Change_When_A_Single_Field_Changes()
    {
        let mut other = Sample();
        other.provider = ProviderId::New("nomos.provider.other");

        assert_ne!(Sample().Digest(), other.Digest());
    }

    #[test]
    fn Test_At_Should_Pair_The_Key_With_A_Generation()
    {
        let key = Sample();
        let identity = key.clone().At(GenerationId::From_Raw(SAMPLE_GENERATION));

        assert_eq!(identity.Key(), &key);
        assert_eq!(identity.generation, GenerationId::From_Raw(SAMPLE_GENERATION));
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Sample() -> Key
    {
        let guarantee = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );

        return Key {
            contract: CapabilityId::New("nomos.cap.test.key"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(1)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: BuildVariantId::From_Digest(Seeded(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded(CONFIGURATION_SEED)),
        };
    }
}
