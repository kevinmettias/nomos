//! Test-only scaffolding every rule's own test module was hand-rebuilding: a fresh
//! [`Registry`] and [`MemoryFactStore`] with one [`ProviderOffer`] declared and offered
//! against one capability contract, the fixed [`Context`] every fixture materializes a
//! fact under, and the one [`FactKey`]-then-`Materialize` shape every fixture reached a
//! fact store through, whatever the payload.
//!
//! Declared behind `#[cfg(test)]` at its `mod test_support;` site in `checks.rs` rather
//! than gated again here: nothing in this file has a reason to exist outside a test
//! binary, and a module that compiled in production for no consumer would be exactly the
//! dead code `check-dead-code` exists to find.

use nomos_analysis::{Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry};
use nomos_contracts::{
    BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128, EvidenceClass, GenerationId, Guarantee,
    ProviderId, SchemaId, SnapshotId, SubjectId,
};

/// A fresh fact store, registry, and the one [`ProviderOffer`] declared into it — named so
/// a call site reads `offering.store`, not a position it has to count.
pub(crate) struct TestOffering
{
    pub(crate) store: MemoryFactStore,
    pub(crate) registry: Registry,
    pub(crate) offer: ProviderOffer,
}

/// Declares `contract` and offers `provider` against `capability` at `version` with
/// `guarantee` — the declare-then-offer pairing every rule's own test module built by
/// hand, one capability import at a time, before this existed.
pub(crate) fn Offering(contract: CapabilityContract, capability: CapabilityId, version: ContractVersion, provider: &str, guarantee: Guarantee) -> TestOffering
{
    let mut registry = Registry::New();
    let offer = ProviderOffer {
        provider: ProviderId::New(provider),
        capability,
        version,
        guarantee,
    };

    registry
        .Declare_And_Offer(contract, offer.clone())
        .expect("declared and offered within the ceiling");

    return TestOffering { store: MemoryFactStore::New(), registry, offer };
}

/// The fixed build/variant/configuration/generation every fixture materializes a fact
/// under. No rule's own test asserts these values themselves — only that a materialized
/// fact resolves at all — so one fixed [`Context`] serves every rule.
pub(crate) fn Test_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
        generation: GenerationId::INITIAL,
    };
}

/// Materializes one already-encoded payload into `store`, addressed the way every rule's
/// own fixture built a [`FactKey`] by hand: `offer`'s own capability, version and
/// provider, `subject`'s identity, and `semantic_inputs` the caller states because only
/// the caller's own capability knows whether its real provider is content-keyed.
pub(crate) fn Materialize(store: &mut MemoryFactStore, subject: SubjectId, offer: &ProviderOffer, semantic_inputs: InputDigest, schema: SchemaId, bytes: Vec<u8>)
{
    let context = Test_Context();
    let key = FactKey {
        contract: offer.capability.clone(),
        contract_version: offer.version,
        subject,
        semantic_inputs,
        provider: offer.provider.clone(),
        provider_version: offer.version,
        guarantee: GuaranteeDigest::Of(&offer.guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };

    store
        .Materialize(
            MaterializedFact {
                identity: key.At(context.generation),
                snapshot: context.snapshot,
                evidence: EvidenceClass::Verified,
                guarantee: offer.guarantee,
                payload: FactPayload::New(schema, bytes),
            },
            &[],
        )
        .expect("nothing here is backdated");
}
