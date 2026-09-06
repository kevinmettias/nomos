//! The shared mechanics beneath this crate's five `nomos.cap.*.policy` providers.
//!
//! `P41-REPOSITORY-CRATE-CONSOLIDATION-2` moved [`crate::naming`], [`crate::limits`],
//! [`crate::scripting`], [`crate::words`] and [`crate::goals`] from five crates into five
//! modules of one, but carried each module's own copy of its registration and fact-assembly
//! plumbing over unchanged. Read side by side, `goals/guarantee.rs` and `limits/guarantee.rs`
//! (and the other three pairs) differ only in which capability crate they import and in
//! lightly paraphrased doc prose -- never in structure or in logic. This module is that
//! plumbing, written once: the `FactContext`/`PolicyFact` shape every whole-workspace
//! provider here answers with, and the mechanical steps of building a `FactKey`, a
//! `ProviderOffer` and a `MaterializedFact` from values a caller already holds.
//!
//! `docs/records/OD-CAPABILITY-008` found, checking field by field rather than assuming it,
//! that this exact shape already agrees across every whole-workspace provider in the
//! workspace (these five, plus `nomos_lang_rust_deny` and `nomos_cap_requirement_trace`
//! outside this crate): the four `FactContext` fields are provenance -- which snapshot,
//! build variant, configuration and generation a fact was measured against -- not policy, so
//! nothing about *what* any of the five capabilities declares depends on which one is asking.
//! `crates/orchestration/nomos-check-orchestration/src/facts/policy_materialization.rs`
//! already builds all five `FactContext` values identically from one `Context`, which is the
//! same finding from the caller's side.
//!
//! # What stays out of this module, on purpose
//!
//! What genuinely differs per provider stays in that provider's own module, stated
//! independently rather than derived from here:
//!
//! - Each module's own `reading.rs` -- the only place a `standards.json` block's meaning is
//!   decided.
//! - Each module's own `PROVIDER` name and `Declared_Guarantee` (`FactVariant`, both
//!   `Assurance` axes, `IncrementalGranularity`): this module's functions take a guarantee as
//!   a plain value rather than reaching for one, precisely so a future policy provider that
//!   needs to declare a different guarantee or a different granularity still can, without
//!   this module changing or growing a special case for it.
//! - Each module's own error type, returned by its own `reading.rs`.
//! - Each module's own `Encode_Payload`. `docs/records/OD-CAPABILITY-008` already reasoned
//!   about this exact boundary for a sibling convention -- "a shared writer would make two
//!   providers of one capability agree by construction and prove nothing"; "the duplication
//!   is the interface." Every one of the five `nomos_cap_*_policy::Encode_Payload` functions
//!   stays exactly where it is, in its own contract crate, imported and called by its own
//!   provider alone. This module never encodes a payload; it only ever receives the bytes a
//!   provider's own `Encode_Payload` already produced.

use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_capability::ProviderOffer;
use nomos_contracts::{
    BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, EvidenceClass, GenerationId,
    Guarantee, ProviderId, SchemaId, SnapshotId, SubjectId,
};

/// Where in the workspace's history a fact is being produced -- the same four fields every
/// `crates/repository/` provider's own fact needs, because they are provenance rather than
/// policy. See this module's own doc for why one shape now serves all five.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// The one fact each of this crate's five `IncrementalGranularity::WholeWorkspace` ceilings
/// allows, together with the subject it was filed under -- `nomos_model::Subject_Of_Path("")`
/// for every one of the five, the whole-tree subject a workspace-wide answer is always
/// attributed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// A provider's offer, assembled from its own identity and its own already-declared
/// guarantee -- the `ProviderOffer` shell every one of the five modules' own `guarantee.rs`
/// built by hand, identically.
#[must_use]
pub(crate) fn Offer(provider: &str, capability: CapabilityId, version: ContractVersion, guarantee: Guarantee) -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(provider),
        capability,
        version,
        guarantee,
    };
}

/// The key a whole-workspace policy fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the choice every `crates/repository/` provider
/// already made independently for the identical reason: each provider's real input is
/// `standards.json`'s own current text, which no caller has independently, so a caller
/// building a lookup key has nothing to reconstruct it from.
pub(crate) fn Compute_Fact_Key(
    capability: CapabilityId,
    contract_version: ContractVersion,
    provider: &str,
    provider_version: ContractVersion,
    subject: SubjectId,
    guarantee: Guarantee,
    context: FactContext,
) -> FactKey
{
    return FactKey {
        contract: capability,
        contract_version,
        subject,
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(provider),
        provider_version,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// Wrapping an already-encoded payload into the one fact a whole-workspace provider answers.
///
/// The `MaterializedFact`/`PolicyFact` shell every one of the five modules' own
/// `fact_context.rs` built by hand, identically; only the bytes and schema -- each
/// provider's own `Encode_Payload`/`Payload_Schema` already produced them before calling
/// this -- differ per module.
pub(crate) fn Materialize_Fact(subject: SubjectId, guarantee: Guarantee, context: FactContext, key: FactKey, schema: SchemaId, payload_bytes: Vec<u8>) -> PolicyFact
{
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(schema, payload_bytes),
    };

    return PolicyFact { subject, fact };
}

/// A `FactContext` test fixture, shared for the identical reason [`Compute_Fact_Key`] and
/// [`Materialize_Fact`] are: every one of the five modules' own `fact_context.rs` test
/// module declared this exact function, with the exact same fill bytes, five times over.
/// `pub(crate)` rather than `#[cfg(test)] mod tests`-private, because its callers are each a
/// different module's own test code, not this module's.
#[cfg(test)]
pub(crate) mod test_support
{
    use super::FactContext;
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};

    /// Fill bytes distinct enough that the returned context's three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    /// A `FactContext` for a test that only needs "some real value", not any particular one.
    pub(crate) fn Sample_Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([
                CONFIGURATION_DIGEST_FILL;
                Digest128::BYTE_LENGTH
            ])),
            generation: GenerationId::INITIAL,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, Digest128, FactVariant, IncrementalGranularity};
    use test_support::Sample_Context;

    const CAPABILITY: &str = "nomos.cap.test.scaffolding";
    const PROVIDER: &str = "nomos.repo.test.scaffolding";

    fn Sample_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::WholeWorkspace,
        );
    }

    #[test]
    fn Test_Offer_Should_Carry_The_Values_Given()
    {
        let offer = Offer(PROVIDER, CapabilityId::New(CAPABILITY), ContractVersion::New(1, 0), Sample_Guarantee());

        assert_eq!(offer.provider, ProviderId::New(PROVIDER));
        assert_eq!(offer.capability, CapabilityId::New(CAPABILITY));
        assert_eq!(offer.version, ContractVersion::New(1, 0));
        assert_eq!(offer.guarantee, Sample_Guarantee());
    }

    #[test]
    fn Test_Compute_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH]));
        let weaker = Guarantee::New(FactVariant::Syntactic, Assurance::Unsound, Assurance::Unknown, IncrementalGranularity::WholeWorkspace);

        let strong_key = Compute_Fact_Key(
            CapabilityId::New(CAPABILITY),
            ContractVersion::New(1, 0),
            PROVIDER,
            ContractVersion::New(1, 0),
            subject,
            Sample_Guarantee(),
            Sample_Context(),
        );
        let weak_key = Compute_Fact_Key(
            CapabilityId::New(CAPABILITY),
            ContractVersion::New(1, 0),
            PROVIDER,
            ContractVersion::New(1, 0),
            subject,
            weaker,
            Sample_Context(),
        );

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    #[test]
    fn Test_Materialize_Fact_Should_Carry_The_Guarantee_And_Payload_Given()
    {
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH]));
        let context = Sample_Context();
        let key = Compute_Fact_Key(
            CapabilityId::New(CAPABILITY),
            ContractVersion::New(1, 0),
            PROVIDER,
            ContractVersion::New(1, 0),
            subject,
            Sample_Guarantee(),
            context,
        );

        let PolicyFact { subject: filed_subject, fact } =
            Materialize_Fact(subject, Sample_Guarantee(), context, key, SchemaId::New("nomos.test.scaffolding.v1"), b"payload".to_vec());

        assert_eq!(filed_subject, subject);
        assert_eq!(fact.guarantee, Sample_Guarantee());
        assert_eq!(fact.snapshot, context.snapshot);
        assert_eq!(fact.evidence, EvidenceClass::Verified);
        assert_eq!(fact.payload.schema, SchemaId::New("nomos.test.scaffolding.v1"));
        assert_eq!(fact.payload.bytes, b"payload".to_vec());
        assert_eq!(fact.Generation(), context.generation);
    }
}
