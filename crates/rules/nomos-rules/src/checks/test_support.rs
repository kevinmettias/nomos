//! Test-only scaffolding every rule's own test module was hand-rebuilding: a fresh
//! [`Registry`] and [`MemoryFactStore`] with one [`ProviderOffer`] declared and offered
//! against one capability contract, the fixed [`Context`] every fixture materializes a
//! fact under, the one [`FactKey`]-then-`Materialize_Fact` shape every fixture reached a
//! fact store through, whatever the payload, and the materialize-then-read pairing a
//! syntax rule's own test module rebuilt to get from the payload it had just filed back
//! to a [`Reader`].
//!
//! Declared behind `#[cfg(test)]` at its `mod test_support;` site in `checks.rs` rather
//! than gated again here: nothing in this file has a reason to exist outside a test
//! binary, and a module that compiled in production for no consumer would be exactly the
//! dead code `check-dead-code` exists to find.

use crate::SourceFile;
use nomos_analysis::{
    Context, FactError, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry, RegistryError};
use nomos_contracts::{
    BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128, EvidenceClass, GenerationId, Guarantee,
    ProviderId, SchemaId, SnapshotId, SubjectId,
};

/// The seed byte each of [`Test_Context`]'s three digests is filled with, distinct per
/// field so a snapshot, a variant and a configuration can never collide by accident.
const SNAPSHOT_SEED: u8 = 1;
const VARIANT_SEED: u8 = 2;
const CONFIGURATION_SEED: u8 = 3;

/// A fresh fact store, registry, and the one [`ProviderOffer`] declared into it — named so
/// a call site reads `offering.store`, not a position it has to count.
pub(crate) struct TestOffering
{
    pub(crate) store: MemoryFactStore,
    pub(crate) registry: Registry,
    pub(crate) offer: ProviderOffer,
}

/// One capability's declaration and the single provider offered against it, grouped so a
/// call site names what each `&str` position is for rather than counting along a five-arg
/// list it could transpose without the compiler objecting.
pub(crate) struct OfferedProvider<'text>
{
    /// The contract declared into the fresh registry.
    pub(crate) contract: CapabilityContract,
    /// The capability the offer answers.
    pub(crate) capability: CapabilityId,
    /// The contract version the declaration and the offer are both written against.
    pub(crate) version: ContractVersion,
    /// The provider id the offer registers.
    pub(crate) provider: &'text str,
    /// The guarantee the offered provider claims.
    pub(crate) guarantee: Guarantee,
}

/// Declares `offered.contract` and offers `offered.provider` against `offered.capability`
/// at `offered.version` with `offered.guarantee` — the declare-then-offer pairing every
/// rule's own test module built by hand, one capability import at a time, before this
/// existed.
///
/// # Errors
///
/// Returns whatever [`Registry::Declare_And_Offer`] refuses —
/// [`RegistryError::AlreadyDeclared`] or [`RegistryError::AlreadyOffered`]. The registry is
/// built empty on the line above and nothing else is offered into it, so neither is
/// reachable here; the error is in the signature because the operation this wraps is
/// fallible, not because the fixture anticipates a failure.
pub(crate) fn Offered_Registry(offered: OfferedProvider<'_>) -> Result<TestOffering, RegistryError>
{
    let mut registry = Registry::New();
    let offer = ProviderOffer {
        provider: ProviderId::New(offered.provider),
        capability: offered.capability,
        version: offered.version,
        guarantee: offered.guarantee,
    };

    registry.Declare_And_Offer(offered.contract, offer.clone())?;

    return Ok(TestOffering { store: MemoryFactStore::New(), registry, offer });
}

/// The fixed build/variant/configuration/generation every fixture materializes a fact
/// under. No rule's own test asserts these values themselves — only that a materialized
/// fact resolves at all — so one fixed [`Context`] serves every rule.
pub(crate) fn Test_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_SEED; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// One fact a caller wants filed into a [`MemoryFactStore`]: the subject it is about, the
/// offer it is filed under, the input it was derived from, and the encoded payload itself.
pub(crate) struct FactToFile<'text>
{
    /// The subject the fact is about.
    pub(crate) subject: SubjectId,
    /// The offer whose capability, version and provider the fact is filed under.
    pub(crate) offer: &'text ProviderOffer,
    /// The input the fact was derived from. The caller states it because only the caller's
    /// own capability knows whether its real provider is content-keyed.
    pub(crate) semantic_inputs: InputDigest,
    /// The schema the bytes below are encoded against.
    pub(crate) schema: SchemaId,
    /// The encoded payload itself.
    pub(crate) bytes: Vec<u8>,
}

/// Materializes `fact` into `store`, addressed the way every rule's own fixture built a
/// [`FactKey`] by hand: `fact.offer`'s own capability, version and provider, `fact.subject`'s
/// identity, and `fact.semantic_inputs`, under the fixed [`Test_Context`] every fixture uses.
///
/// # Errors
///
/// Returns [`FactError::Backdated`] if `store` already held a fact under this identity at a
/// newer generation. Each fixture files one fact per identity into a store it built for the
/// purpose, so this is not reachable here; the error is in the signature because
/// [`MemoryFactStore::Materialize`] is fallible, not because the fixture anticipates a
/// failure.
pub(crate) fn Materialize_Fact(store: &mut MemoryFactStore, fact: FactToFile<'_>) -> Result<(), FactError>
{
    let context = Test_Context();

    store
        .Materialize(
            MaterializedFact {
                identity: Key_Of(&fact, &context).At(context.generation),
                snapshot: context.snapshot,
                evidence: EvidenceClass::Verified,
                guarantee: fact.offer.guarantee,
                payload: FactPayload::New(fact.schema, fact.bytes),
            },
            &[],
        )?;

    return Ok(());
}

/// The [`FactKey`] `fact` is filed under: `fact.offer`'s own capability, version and
/// provider, `fact.subject`'s identity, and `fact.semantic_inputs`, under `context`'s
/// variant and configuration. Both of a fact's provider versions name the offer's one
/// version, because a test fixture offers exactly one version of each capability.
fn Key_Of(fact: &FactToFile<'_>, context: &Context) -> FactKey
{
    return FactKey {
        contract: fact.offer.capability.clone(),
        contract_version: fact.offer.version,
        subject: fact.subject,
        semantic_inputs: fact.semantic_inputs,
        provider: fact.offer.provider.clone(),
        provider_version: fact.offer.version,
        guarantee: GuaranteeDigest::Of(&fact.offer.guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// Files `source`'s own syntax payload into `offering`'s store and answers a [`Reader`] over
/// the result: one fact, keyed by `source`'s subject with `source`'s own text as its semantic
/// input, carrying `payload_text` encoded against [`nomos_cap_syntax::Payload_Schema`].
///
/// The materialize-then-read pairing two syntax-reading rules' own test modules rebuilt key by
/// key before this existed — `crosslang`'s and `naming`'s — which differed only in which source
/// and which payload text they filed. What the caller does with the reader is the caller's
/// business, and the only thing the two ever did differently.
///
/// # Errors
///
/// Returns [`FactError::Backdated`] if `offering`'s store already held a fact under this
/// identity at a newer generation. Every call site hands it a store built for the case at hand,
/// so this is not reachable here; the error is in the signature because
/// [`MemoryFactStore::Materialize`] is fallible, not because the fixture anticipates a failure.
pub(crate) fn Reader_Over_A_Syntax_Fact<'offering>(
    offering: &'offering mut TestOffering,
    source: &SourceFile,
    payload_text: &str,
) -> Result<Reader<'offering, 'offering>, FactError>
{
    Materialize_Fact(
        &mut offering.store,
        FactToFile {
            subject: source.subject,
            offer: &offering.offer,
            semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
            schema: nomos_cap_syntax::Payload_Schema(),
            bytes: payload_text.as_bytes().to_vec(),
        },
    )?;

    return Ok(Reader::On(&offering.store, &offering.registry, Test_Context()));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::FactReader as _;

    #[test]
    fn Test_Offering_Should_Declare_The_Contract_And_Admit_The_One_Named_Provider()
    {
        let offering = Offered_Registry(OfferedProvider { contract: nomos_cap_syntax::Capability_Contract(), capability: nomos_cap_syntax::Capability(), version: nomos_cap_syntax::CONTRACT_VERSION, provider: "nomos.test.test_support.offers", guarantee: Floor() }).expect("a fresh Registry holds neither this contract nor this provider");

        assert_eq!(offering.offer.provider, ProviderId::New("nomos.test.test_support.offers"));
        let registered: Vec<&ProviderId> = offering.registry.Offers(&nomos_cap_syntax::Capability()).iter().map(|offer| &offer.provider).collect();
        assert_eq!(registered, vec![&offering.offer.provider]);
    }

    #[test]
    fn Test_Test_Context_Should_Be_A_Fixed_Value_Every_Call_Reproduces()
    {
        assert_eq!(Test_Context(), Test_Context());
    }

    #[test]
    fn Test_Materialize_Should_File_A_Fact_A_Real_Reader_Can_Read_Back()
    {
        let offering = Offered_Registry(OfferedProvider { contract: nomos_cap_syntax::Capability_Contract(), capability: nomos_cap_syntax::Capability(), version: nomos_cap_syntax::CONTRACT_VERSION, provider: "nomos.test.test_support.materializes", guarantee: Floor() }).expect("a fresh Registry holds neither this contract nor this provider");
        let mut store = offering.store;
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_SEED; Digest128::BYTE_LENGTH]));
        Materialize_Fact(&mut store, FactToFile { subject, offer: &offering.offer, semantic_inputs: InputDigest::Of(&[]), schema: nomos_cap_syntax::Payload_Schema(), bytes: b"unexpanded\t0\n".to_vec() }).expect("the fixture's store holds no fact under this key at a newer generation");

        let mut reader = nomos_analysis::Reader::On(&store, &offering.registry, Test_Context());
        let need = nomos_capability::Requirement::New(nomos_cap_syntax::Capability(), nomos_cap_syntax::CONTRACT_VERSION, Floor());
        let fact = reader.Require(&nomos_cap_syntax::Capability(), &subject, InputDigest::Of(&[]), &need).expect("just materialized");

        assert_eq!(fact.payload.schema, nomos_cap_syntax::Payload_Schema());
        assert_eq!(fact.payload.bytes, b"unexpanded\t0\n".to_vec());
    }

    /// The subject seed this file's one read-back test uses — distinct from the three
    /// [`Test_Context`] seeds so the subject cannot collide with a snapshot, a variant or a
    /// configuration digest.
    const SUBJECT_SEED: u8 = 9;

    fn Floor() -> Guarantee
    {
        return Guarantee::New(
            nomos_contracts::FactVariant::Syntactic,
            nomos_contracts::Assurance::Sound,
            nomos_contracts::Assurance::Unknown,
            nomos_contracts::IncrementalGranularity::File,
        );
    }
}
