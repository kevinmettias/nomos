//! A fact key together with the inputs and guarantee that produced it.

use nomos_contracts::Digest128;
use nomos_contracts::GenerationId;
use crate::FactKey;
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identity
{
    pub key: FactKey,
    pub generation: GenerationId,
}

impl Identity
{
    #[must_use]
    pub const fn Key(&self) -> &FactKey
    {
        return &self.key;
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        return self.key.Digest();
    }
}

impl core::fmt::Display for Identity
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "{}@{} of {} by {} at {}",
            self.key.contract, self.key.contract_version, self.key.subject, self.key.provider,
            self.generation
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::GuaranteeDigest;
    use crate::InputDigest;
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, FactVariant,
        Guarantee, IncrementalGranularity, ProviderId, SubjectId,
    };

    #[test]
    fn Test_Key_Should_Return_The_Identitys_Own_Key()
    {
        let key = Sample_Key();
        let identity = key.clone().At(GenerationId::From_Raw(5));

        assert_eq!(identity.Key(), &key);
    }

    #[test]
    fn Test_Digest_Should_Match_The_Keys_Own_Digest()
    {
        let key = Sample_Key();
        let identity = key.clone().At(GenerationId::From_Raw(5));

        assert_eq!(identity.Digest(), key.Digest());
    }

    fn Seeded(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Sample_Key() -> FactKey
    {
        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.identity"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded(1)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            )),
            variant: BuildVariantId::From_Digest(Seeded(3)),
            configuration: ConfigurationId::From_Digest(Seeded(4)),
        };
    }
}
