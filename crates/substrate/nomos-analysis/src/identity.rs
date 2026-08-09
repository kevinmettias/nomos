use nomos_contracts::{
    BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128, GenerationId,
    Guarantee, ProviderId, SubjectId,
};
use nomos_model::Digest_Of_Parts;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InputDigest(Digest128);

impl InputDigest
{
    #[must_use]
    pub fn Of(parts: &[&[u8]]) -> Self
    {
        return Self(Digest_Of_Parts(parts));
    }

    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for InputDigest
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GuaranteeDigest(Digest128);

impl GuaranteeDigest
{
    #[must_use]
    pub fn Of(guarantee: &Guarantee) -> Self
    {
        return Self(Digest_Of_Parts(&[
            &[guarantee.variant as u8],
            &[guarantee.soundness as u8],
            &[guarantee.completeness as u8],
            &[guarantee.incremental as u8],
        ]));
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for GuaranteeDigest
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}

/// The parts of a fact's identity.
///
/// # Why there is no `Snapshot`
///
/// There was one, and it defeated the two components either side of it.
///
/// A workspace snapshot identity is a digest over *every* member of the workspace, so a key
/// carrying one changes for every fact in the corpus whenever any one file is edited. The
/// vertical slice measured it: over a six-file corpus, editing one file recomputed eight
/// facts where two had changed.
///
/// It was also redundant twice over. [`Component::SemanticInputs`] already says what a fact
/// was computed from — for a leaf, the file's content, exactly and no more coarsely — and
/// the store's generation interval already says which analysis state a fact is current at.
/// The snapshot added every other file in the workspace to the first and, being the
/// coarsest of the three, overrode the second.
///
/// A fact's relation to a workspace state is now provenance on
/// [`crate::MaterializedFact::snapshot`]: what it was measured against, not part of what it
/// is. `docs/records/OD-ANALYSIS-001` carries the finding and what closed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component
{
    Contract,
    ContractVersion,
    Subject,
    SemanticInputs,
    Provider,
    ProviderVersion,
    Guarantee,
    Variant,
    Configuration,
}

impl Component
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Contract => "contract",
            Self::ContractVersion => "contract_version",
            Self::Subject => "subject",
            Self::SemanticInputs => "semantic_inputs",
            Self::Provider => "provider",
            Self::ProviderVersion => "provider_version",
            Self::Guarantee => "guarantee",
            Self::Variant => "variant",
            Self::Configuration => "configuration",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Contract,
            Self::ContractVersion,
            Self::Subject,
            Self::SemanticInputs,
            Self::Provider,
            Self::ProviderVersion,
            Self::Guarantee,
            Self::Variant,
            Self::Configuration,
        ];
    }
}

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

const fn Version_Bytes(version: ContractVersion) -> [u8; 4]
{
    let major = version.major.to_be_bytes();
    let minor = version.minor.to_be_bytes();

    return [major[0], major[1], minor[0], minor[1]];
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactIdentity
{
    pub key: FactKey,
    pub generation: GenerationId,
}

impl FactIdentity
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

impl core::fmt::Display for FactIdentity
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
