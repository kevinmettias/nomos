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
//!   decided: which key it reads, what one entry beneath it must look like, and which error
//!   a refusal returns. [`Discover_Scoped_Rows`] walks that structure, but decides none of
//!   those three; all three arrive from the caller's own module.
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

use crate::standards_document::{Read_Standards_Document, StandardsDocumentError};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_capability::ProviderOffer;
use nomos_contracts::{CapabilityId, ContractVersion, EvidenceClass, Guarantee, ProviderId, SchemaId, SubjectId};
use nomos_platform::FileSystem;
use std::path::Path;

/// A provider's offer, assembled from its own identity and its own already-declared
/// guarantee -- the `ProviderOffer` shell every one of the five modules' own `guarantee.rs`
/// built by hand, identically.
#[must_use]
pub(crate) fn Provider_Offer(provider: &str, capability: CapabilityId, version: ContractVersion, guarantee: Guarantee) -> ProviderOffer
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

/// One provider's own fixed identity, from the four values its module already declares.
///
/// Shared because all six modules' own `fact_context.rs` built the identical four-field
/// literal, once in its production path and again in its test module. The two version
/// parameters stay separate rather than collapsing into one: every provider here happens to
/// version itself with its own contract, and a provider that stops doing so must still be
/// able to say so through this function without the function changing.
pub(crate) fn Declared_Identity(
    capability: CapabilityId,
    contract_version: ContractVersion,
    provider: &str,
    provider_version: ContractVersion,
) -> CapabilityIdentity<'_>
{
    return CapabilityIdentity { capability, contract_version, provider, provider_version };
}

/// The subject every whole-workspace provider in this crate files its fact under.
///
/// Written once because all six providers already answered with the identical value. A
/// whole-workspace fact is a fact about the workspace, and the workspace has one name in
/// `nomos_model`'s own subject grammar rather than six -- the empty path, which is the
/// subject `nomos_model::Subject_Of_Path` reads as "the workspace itself" rather than any
/// member of it.
pub(crate) fn Workspace_Subject() -> SubjectId
{
    return nomos_model::Subject_Of_Path("");
}

/// Everything a whole-workspace provider already holds once its own reader has produced a
/// payload: the subject it assessed, the guarantee it declared, the identity it files under,
/// the context it measured against, and the bytes its own `Encode_Payload` produced together
/// with the schema they were encoded under.
///
/// Grouped because a caller holds all six together by the time it answers -- the same reason
/// [`CapabilityIdentity`], [`FactFiling`] and [`EncodedPayload`] exist -- and because six
/// separate parameters exceed this workspace's four-value-parameter cap.
pub(crate) struct WholeWorkspaceFiling<'a>
{
    pub subject: SubjectId,
    pub guarantee: Guarantee,
    pub identity: CapabilityIdentity<'a>,
    pub context: FactContext,
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

/// The one fact a whole-workspace provider answers, from the values it already holds.
///
/// The assembly every one of this crate's six providers' own `fact_context.rs` wrote out by
/// hand, identically: file the key, wrap the payload its own `Encode_Payload` already
/// produced, and materialize the fact. What differs per provider -- which capability, which
/// schema, which guarantee, which bytes -- arrives as values, so this function decides none
/// of them and keeps the providers from agreeing by construction about anything that carries
/// meaning. `OD-CAPABILITY-008` draws that same line for `Encode_Payload`, which stays in
/// each contract crate: here it is only ever called, never replaced.
pub(crate) fn Materialize_Whole_Workspace(filing: WholeWorkspaceFiling<'_>) -> PolicyFact
{
    let key = Compute_Fact_Key(&filing.identity, filing.subject, filing.guarantee, filing.context);
    let positioned = FactFiling { context: filing.context, key };
    let encoded = EncodedPayload { schema: filing.schema, bytes: filing.bytes };

    return Materialize_Fact(filing.subject, filing.guarantee, positioned, encoded);
}

/// One `standards.json` block's own rows, walked the one way every provider here walks
/// them: at repository scope first, then once per language under `languages`.
///
/// The walk is identical across providers; everything that could carry meaning is not, and
/// arrives from the caller rather than being decided here:
///
/// - `block` is the key this provider reads -- `"naming"`, `"limits"`, ...
/// - `repository_scope` and `language_scope` are this provider's own scope values, built
///   from its own contract crate's `Scope`; nothing in this module names that type.
/// - `decode` decides what one entry means and refuses one that is not what this provider
///   declares. That decision is what [`crate`]'s own module doc means by a block's
///   *meaning*, and it stays in each module's own `reading.rs`.
/// - `wrap` builds this provider's own error type, so a reader that refuses still refuses
///   with the type its own callers match on -- the five types `OD-RULES-019` kept apart on
///   purpose rather than collapsing into the one they are each built from.
///
/// Nothing here names a block, a scope, a row type or an error type, so two providers
/// cannot come to agree about any of them by way of this function.
pub(crate) struct ScopedBlock<'a, Scope, LanguageScope, Decode, Wrap>
{
    pub block: &'a str,
    pub repository_scope: Scope,
    pub language_scope: LanguageScope,
    pub decode: Decode,
    pub wrap: Wrap,
}

/// Every row `root`'s own `standards.json` declares in `block`, at the scopes `block` names.
///
/// A missing file, a missing block, or a missing per-language block declares no rows -- not
/// an error -- since an unconfigured repository is not a repository this crate failed to
/// read. Nothing is sorted here; each caller orders its own rows, its own row type having
/// its own name to order by.
///
/// # Errors
///
/// Whatever [`Read_Standards_Document`] refuses with, converted by `block.wrap`;
/// otherwise whatever `block.decode` refuses with.
pub(crate) fn Discover_Scoped_Rows<Row, Scope, LanguageScope, Decode, Wrap, Error>(
    root: &Path,
    filesystem: &impl FileSystem,
    block: &ScopedBlock<'_, Scope, LanguageScope, Decode, Wrap>,
) -> Result<Vec<Row>, Error>
where
    Scope: Clone,
    LanguageScope: Fn(String) -> Scope,
    Decode: Fn(&Scope, &str, &serde_json::Value) -> Result<Row, Error>,
    Wrap: Fn(StandardsDocumentError) -> Error,
{
    let value = Read_Standards_Document(root, filesystem).map_err(&block.wrap)?;
    let declarations = Scoped_Declarations(&value, &block.repository_scope, &block.language_scope);

    let mut rows = Vec::new();
    for (scope, declared) in declarations
    {
        Scoped_Rows_Into(declared, block, &scope, &mut rows)?;
    }

    return Ok(rows);
}

/// `value`'s own block at repository scope, then its own block under each language, each
/// paired with the scope it is read at.
fn Scoped_Declarations<'a, Scope, LanguageScope>(
    value: &'a serde_json::Value,
    repository_scope: &Scope,
    language_scope: &LanguageScope,
) -> Vec<(Scope, &'a serde_json::Value)>
where
    Scope: Clone,
    LanguageScope: Fn(String) -> Scope,
{
    let mut declarations = vec![(repository_scope.clone(), value)];

    if let Some(languages) = value.get("languages").and_then(serde_json::Value::as_object)
    {
        for (language, declared) in languages
        {
            declarations.push((language_scope(language.clone()), declared));
        }
    }

    return declarations;
}

/// Every entry `value` declares in `block`'s own object, appended to `rows` at `scope`.
fn Scoped_Rows_Into<Row, Scope, LanguageScope, Decode, Wrap, Error>(
    value: &serde_json::Value,
    block: &ScopedBlock<'_, Scope, LanguageScope, Decode, Wrap>,
    scope: &Scope,
    rows: &mut Vec<Row>,
) -> Result<(), Error>
where
    Decode: Fn(&Scope, &str, &serde_json::Value) -> Result<Row, Error>,
{
    let Some(entries) = value.get(block.block).and_then(serde_json::Value::as_object)
    else
    {
        return Ok(());
    };

    for (name, entry) in entries
    {
        rows.push((block.decode)(scope, name, entry)?);
    }

    return Ok(());
}

/// A `FactContext` test fixture, shared for the identical reason [`Compute_Fact_Key`] and
/// [`Materialize_Fact`] are: every one of the five modules' own `fact_context.rs` test
/// module declared this exact function, with the exact same fill bytes, five times over.
/// `pub(crate)` rather than `#[cfg(test)] mod tests`-private, because its callers are each a
/// different module's own test code, not this module's.
#[cfg(test)]
pub(crate) mod test_support
{
    use super::{CapabilityIdentity, Compute_Fact_Key, FactContext, Workspace_Subject};
    use nomos_analysis::{GuaranteeDigest, MaterializedFact};
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId, Guarantee, IncrementalGranularity,
        SchemaId, SnapshotId, SubjectId,
    };
    use std::path::PathBuf;

    /// Fill bytes distinct enough that the returned context's three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    /// How many directory levels separate this crate from the workspace root it sits under.
    const LEVELS_TO_WORKSPACE_ROOT: usize = 3;

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

    /// The workspace root this crate's own `standards.json` lives at.
    ///
    /// Shared because the four providers whose tests read this repository's own document
    /// walked up from `CARGO_MANIFEST_DIR` identically. The walk is the same one whichever
    /// module asks: a crate has one manifest directory.
    pub(crate) fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        return manifest
            .ancestors()
            .nth(LEVELS_TO_WORKSPACE_ROOT)
            .expect("this crate sits three levels below the workspace root")
            .to_path_buf();
    }

    /// Asserts the property [`Compute_Fact_Key`]'s guarantee digest exists to provide: two
    /// offers of one subject at different guarantees file under different keys.
    ///
    /// Shared because every one of this crate's six providers' own `fact_context.rs` test
    /// modules asserted it identically. Each caller still passes its own identity and its own
    /// declared guarantee in, so a provider that stopped filing apart by guarantee -- or
    /// stopped declaring the guarantee it says it declares -- still fails here.
    pub(crate) fn Assert_Keys_File_Apart_By_Guarantee(identity: &CapabilityIdentity<'_>, guarantee: Guarantee)
    {
        let weaker = Guarantee::New(FactVariant::Syntactic, Assurance::Unsound, Assurance::Unknown, IncrementalGranularity::WholeWorkspace);

        let strong_key = Compute_Fact_Key(identity, Workspace_Subject(), guarantee, Sample_Context());
        let weak_key = Compute_Fact_Key(identity, Workspace_Subject(), weaker, Sample_Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    /// Asserts the four things every materialized whole-workspace fact carries: the workspace
    /// subject, the guarantee its provider declared, a key whose guarantee digest agrees with
    /// it, and the schema its provider's own `Encode_Payload` encoded under.
    ///
    /// Shared for the same reason as the assertion above; each caller passes its own declared
    /// guarantee and its own schema in, so a provider that stopped carrying either still fails.
    pub(crate) fn Assert_Carries_Its_Declaration(subject: &SubjectId, fact: &MaterializedFact, guarantee: Guarantee, schema: &SchemaId)
    {
        assert_eq!(*subject, Workspace_Subject());
        assert_eq!(fact.guarantee, guarantee);
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&guarantee));
        assert_eq!(fact.payload.schema, *schema);
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

        let offer = Provider_Offer(PROVIDER, capability, version, Sample_Guarantee());

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
