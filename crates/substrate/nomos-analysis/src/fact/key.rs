//! What a fact is about, before anything has been said about it.

use nomos_contracts::GenerationId;
use nomos_model::Digest_Of_Parts;
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
pub struct FactKey
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

impl FactKey
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
