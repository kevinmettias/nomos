//! The one key every test here varies, and the store, registry and context it is read
//! against.

use nomos_analysis::{
    Component, Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact,
    MemoryFactStore,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry, Requirement};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SchemaId, SnapshotId, SubjectId,
};

pub(crate) const SYNTAX: &str = "nomos.cap.syntax.tree";

pub(crate) const SEMANTIC: &str = "nomos.cap.semantic.resolution";

fn Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

pub(crate) fn Subject(seed: u8) -> SubjectId
{
    return SubjectId::From_Digest(Digest(seed));
}

pub(crate) fn Snapshot(seed: u8) -> SnapshotId
{
    return SnapshotId::From_Digest(Digest(seed));
}

fn Variant(seed: u8) -> BuildVariantId
{
    return BuildVariantId::From_Digest(Digest(seed));
}

pub(crate) fn Configuration(seed: u8) -> ConfigurationId
{
    return ConfigurationId::From_Digest(Digest(seed));
}

pub(crate) fn Syntactic() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

pub(crate) fn Coarse() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
    );
}

pub(crate) fn Base() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New(SYNTAX),
        contract_version: ContractVersion::New(1, 0),
        subject: Subject(1),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.rust-syntax"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Syntactic()),
        variant: Variant(3),
        configuration: Configuration(4),
    };
}

pub(crate) fn Varied(component: Component) -> FactKey
{
    let mut key = Base();
    match component
    {
        Component::Contract => key.contract = CapabilityId::New(SEMANTIC),
        Component::ContractVersion => key.contract_version = ContractVersion::New(2, 0),
        Component::Subject => key.subject = Subject(9),
        Component::SemanticInputs => key.semantic_inputs = InputDigest::Of(&[b"fn main() { x }"]),
        Component::Provider => key.provider = ProviderId::New("nomos.provider.other"),
        Component::ProviderVersion => key.provider_version = ContractVersion::New(1, 1),
        Component::Guarantee => key.guarantee = GuaranteeDigest::Of(&Coarse()),
        Component::Variant => key.variant = Variant(9),
        Component::Configuration => key.configuration = Configuration(9),
    }

    return key;
}

pub(crate) fn Fact(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: Snapshot(2),
        evidence: EvidenceClass::Derived,
        guarantee: Syntactic(),
        payload: FactPayload::New(SchemaId::New("nomos.syntax.v1"), b"tree".to_vec()),
    };
}

pub(crate) fn Stored(key: &FactKey) -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    let fact = Fact(key, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");

    return store;
}

pub(crate) fn Context_At(generation: GenerationId) -> Context
{
    return Context {
        snapshot: Snapshot(2),
        variant: Variant(3),
        configuration: Configuration(4),
        generation,
    };
}

pub(crate) fn Offering(guarantee: Guarantee) -> Registry
{
    let mut registry = Registry::New();
    registry
        .Declare(CapabilityContract {
            id: CapabilityId::New(SYNTAX),
            version: ContractVersion::New(1, 0),
            summary: "A syntax tree".to_owned(),
            ceiling: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Symbol,
            ),
        })
        .expect("declares");
    registry
        .Offer(ProviderOffer {
            provider: ProviderId::New("nomos.provider.rust-syntax"),
            capability: CapabilityId::New(SYNTAX),
            version: ContractVersion::New(1, 0),
            guarantee,
        })
        .expect("offers");

    return registry;
}

pub(crate) fn Needing(guarantee: Guarantee) -> Requirement
{
    let capability = CapabilityId::New(SYNTAX);
    let version = ContractVersion::New(1, 0);

    return Requirement::New(capability, version, guarantee);
}
