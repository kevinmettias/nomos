//! The shared mechanics beneath this crate's five `nomos.cap.*.policy` providers.
//!
//! `P41-REPOSITORY-CRATE-CONSOLIDATION-2` moved [`crate::naming`], [`crate::limits`],
//! [`crate::scripting`], [`crate::words`] and [`crate::goals`] from five crates into five
//! modules of one, but carried each module's own copy of its registration and fact-assembly
//! plumbing over unchanged. Read side by side, `goals/guarantee.rs` and `limits/guarantee.rs`
//! (and the other three pairs) differ only in which capability crate they import and in
//! lightly paraphrased doc prose -- never in structure or in logic. This module is that
//! plumbing, written once: the [`FactContext`]/[`PolicyFact`] shape every whole-workspace
//! provider here answers with (each its own file, `file-name-matches-declared-type`'s own
//! requirement), and the mechanical steps of building a `FactKey`, a `ProviderOffer` and a
//! `MaterializedFact` from values a caller already holds.
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
//!
//! # Why `Compute_Fact_Key` and `Materialize_Fact` each take a grouped value
//!
//! Both functions' own real inputs exceed this workspace's four-value-parameter cap.
//! [`CapabilityIdentity`] groups the four values that name *which* capability and *which*
//! provider are filing -- a caller always supplies its own four together, never one without
//! the rest. [`FactFiling`] groups a context with the key it already produced, and
//! [`EncodedPayload`] groups a payload's bytes with the schema they were encoded under --
//! both pairs a caller already holds together by the time it calls [`Materialize_Fact`].

mod fact_context;
mod policy_fact;

pub use fact_context::FactContext;
pub use policy_fact::PolicyFact;

use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_capability::ProviderOffer;
use nomos_contracts::{CapabilityId, ContractVersion, EvidenceClass, Guarantee, ProviderId, SchemaId, SubjectId};

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

/// Which capability a fact answers, and which provider is answering it -- the four values
/// [`Compute_Fact_Key`] needs beyond the subject, guarantee and context every caller already
/// holds separately. Grouped because a caller always supplies its own four together: they
/// are one provider's own fixed identity, never assembled from values two different callers
/// contributed. Taken by reference rather than derived `Copy`, since `CapabilityId` is not.
#[derive(Clone)]
pub(crate) struct CapabilityIdentity<'a>
{
    pub capability: CapabilityId,
    pub contract_version: ContractVersion,
    pub provider: &'a str,
    pub provider_version: ContractVersion,
}

/// The key a whole-workspace policy fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the choice every `crates/repository/` provider
/// already made independently for the identical reason: each provider's real input is
/// `standards.json`'s own current text, which no caller has independently, so a caller
/// building a lookup key has nothing to reconstruct it from.
pub(crate) fn Compute_Fact_Key(identity: &CapabilityIdentity<'_>, subject: SubjectId, guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: identity.capability.clone(),
        contract_version: identity.contract_version,
        subject,
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(identity.provider),
        provider_version: identity.provider_version,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// Where a fact is positioned -- the context it was measured against and the key it was
/// already filed under, since a caller always computes the key from this same context and
/// never holds one without the other by the time it calls [`Materialize_Fact`].
pub(crate) struct FactFiling
{
    pub context: FactContext,
    pub key: FactKey,
}

/// An already-encoded payload -- the bytes a provider's own `Encode_Payload` produced, and
/// the schema they were encoded under.
pub(crate) struct EncodedPayload
{
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

/// Wrapping an already-encoded payload into the one fact a whole-workspace provider answers.
///
/// The `MaterializedFact`/`PolicyFact` shell every one of the five modules' own
/// `fact_context.rs` built by hand, identically; only the payload -- each provider's own
/// `Encode_Payload`/`Payload_Schema` already produced it before calling this -- differs per
/// module.
pub(crate) fn Materialize_Fact(subject: SubjectId, guarantee: Guarantee, filing: FactFiling, payload: EncodedPayload) -> PolicyFact
{
    let fact = MaterializedFact {
        identity: filing.key.At(filing.context.generation),
        snapshot: filing.context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(payload.schema, payload.bytes),
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

    /// The subject fill byte both key-filing tests measure against. Arbitrary, and distinct
    /// from the fills `Sample_Context` uses so that a subject and a context cannot be confused.
    const SUBJECT_DIGEST_FILL: u8 = 9;

    #[test]
    fn Test_Offer_Should_Carry_The_Values_Given()
    {
        let capability = CapabilityId::New(CAPABILITY);
        let version = ContractVersion::New(1, 0);

        let offer = Offer(PROVIDER, capability, version, Sample_Guarantee());

        assert_eq!(offer.provider, ProviderId::New(PROVIDER));
        assert_eq!(offer.capability, CapabilityId::New(CAPABILITY));
        assert_eq!(offer.version, ContractVersion::New(1, 0));
        assert_eq!(offer.guarantee, Sample_Guarantee());
    }

    #[test]
    fn Test_Compute_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_DIGEST_FILL; Digest128::BYTE_LENGTH]));
        let weaker = Guarantee::New(FactVariant::Syntactic, Assurance::Unsound, Assurance::Unknown, IncrementalGranularity::WholeWorkspace);

        let strong_key = Compute_Fact_Key(&Sample_Identity(), subject, Sample_Guarantee(), Sample_Context());
        let weak_key = Compute_Fact_Key(&Sample_Identity(), subject, weaker, Sample_Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    #[test]
    fn Test_Materialize_Fact_Should_Carry_The_Guarantee_And_Payload_Given()
    {
        let subject = SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_DIGEST_FILL; Digest128::BYTE_LENGTH]));
        let context = Sample_Context();
        let key = Compute_Fact_Key(&Sample_Identity(), subject, Sample_Guarantee(), context);

        let PolicyFact { subject: filed_subject, fact } = Materialize_Fact(
            subject,
            Sample_Guarantee(),
            FactFiling { context, key },
            EncodedPayload { schema: SchemaId::New("nomos.test.scaffolding.v1"), bytes: b"payload".to_vec() },
        );

        assert_eq!(filed_subject, subject);
        assert_eq!(fact.guarantee, Sample_Guarantee());
        assert_eq!(fact.snapshot, context.snapshot);
        assert_eq!(fact.evidence, EvidenceClass::Verified);
        assert_eq!(fact.payload.schema, SchemaId::New("nomos.test.scaffolding.v1"));
        assert_eq!(fact.payload.bytes, b"payload".to_vec());
        assert_eq!(fact.Generation(), context.generation);
    }

    fn Sample_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::WholeWorkspace,
        );
    }

    fn Sample_Identity() -> CapabilityIdentity<'static>
    {
        return CapabilityIdentity {
            capability: CapabilityId::New(CAPABILITY),
            contract_version: ContractVersion::New(1, 0),
            provider: PROVIDER,
            provider_version: ContractVersion::New(1, 0),
        };
    }
}
