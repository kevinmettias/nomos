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

/// The base key's non-subject components. Distinct seeds, so a component that drifted into
/// another's position changes the key rather than colliding with it.
const BASE_SNAPSHOT_SEED: u8 = 2;
const BASE_VARIANT_SEED: u8 = 3;
const BASE_CONFIGURATION_SEED: u8 = 4;

/// The seed [`Key_With_Varied_Component`] sets a component to: one value, so a changed component can be
/// compared against the same alternative whichever component it is.
const VARIED_SEED: u8 = 9;

/// The contract version [`Key_With_Varied_Component`] substitutes for the base key's major 1.
const VARIED_CONTRACT_MAJOR: u16 = 2;

fn Digest_From_Seed(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

pub(crate) fn Subject_Id_From_Seed(seed: u8) -> SubjectId
{
    return SubjectId::From_Digest(Digest_From_Seed(seed));
}

pub(crate) fn Snapshot_Id_From_Seed(seed: u8) -> SnapshotId
{
    return SnapshotId::From_Digest(Digest_From_Seed(seed));
}

fn Build_Variant_Id_From_Seed(seed: u8) -> BuildVariantId
{
    return BuildVariantId::From_Digest(Digest_From_Seed(seed));
}

pub(crate) fn Configuration_Id_From_Seed(seed: u8) -> ConfigurationId
{
    return ConfigurationId::From_Digest(Digest_From_Seed(seed));
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
        subject: Subject_Id_From_Seed(1),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.rust-syntax"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Syntactic()),
        variant: Build_Variant_Id_From_Seed(BASE_VARIANT_SEED),
        configuration: Configuration_Id_From_Seed(BASE_CONFIGURATION_SEED),
    };
}

pub(crate) fn Key_With_Varied_Component(component: Component) -> FactKey
{
    let mut key = Base();
    match component
    {
        Component::Contract => key.contract = CapabilityId::New(SEMANTIC),
        Component::ContractVersion => key.contract_version = ContractVersion::New(VARIED_CONTRACT_MAJOR, 0),
        Component::Subject => key.subject = Subject_Id_From_Seed(VARIED_SEED),
        Component::SemanticInputs => key.semantic_inputs = InputDigest::Of(&[b"fn main() { x }"]),
        Component::Provider => key.provider = ProviderId::New("nomos.provider.other"),
        Component::ProviderVersion => key.provider_version = ContractVersion::New(1, 1),
        Component::Guarantee => key.guarantee = GuaranteeDigest::Of(&Coarse()),
        Component::Variant => key.variant = Build_Variant_Id_From_Seed(VARIED_SEED),
        Component::Configuration => key.configuration = Configuration_Id_From_Seed(VARIED_SEED),
    }

    return key;
}

pub(crate) fn Materialized_Fact_From_Key(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: Snapshot_Id_From_Seed(BASE_SNAPSHOT_SEED),
        evidence: EvidenceClass::Derived,
        guarantee: Syntactic(),
        payload: FactPayload::New(SchemaId::New("nomos.syntax.v1"), b"tree".to_vec()),
    };
}

pub(crate) fn Memory_Store_For_Key(key: &FactKey) -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    let fact = Materialized_Fact_From_Key(key, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");

    return store;
}

pub(crate) fn Context_At(generation: GenerationId) -> Context
{
    return Context {
        snapshot: Snapshot_Id_From_Seed(BASE_SNAPSHOT_SEED),
        variant: Build_Variant_Id_From_Seed(BASE_VARIANT_SEED),
        configuration: Configuration_Id_From_Seed(BASE_CONFIGURATION_SEED),
        generation,
    };
}

pub(crate) fn Registry_For_Guarantee(guarantee: Guarantee) -> Registry
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
        .expect("Registry::New built this registry empty, so SYNTAX has no contract yet");
    registry
        .Offer(ProviderOffer {
            provider: ProviderId::New("nomos.provider.rust-syntax"),
            capability: CapabilityId::New(SYNTAX),
            version: ContractVersion::New(1, 0),
            guarantee,
        })
        .expect("every caller offers the Syntactic guarantee, which the ceiling above admits");

    return registry;
}

pub(crate) fn Requirement_For_Guarantee(guarantee: Guarantee) -> Requirement
{
    let capability = CapabilityId::New(SYNTAX);
    let version = ContractVersion::New(1, 0);

    return Requirement::New(capability, version, guarantee);
}
