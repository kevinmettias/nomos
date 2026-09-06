//! Turning one live `gh api` review-comment call into the one fact this capability
//! answers.

use crate::contract::{Capability, Payload_Schema, CONTRACT_VERSION};
use crate::fetching::{Fetch_Review_Comment, FetchError};
use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::payload::{finding_payload::FindingPayload, Encode_Payload};
use crate::translation::{Translate_Review_Comment, TranslationError};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use nomos_platform::ProcessLauncher;

/// Where in the workspace's history a fact is being produced -- the same four-field shape
/// every other real provider's own `FactContext` carries, for the identical reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// The one fact this capability's `IncrementalGranularity::None` ceiling allows for one
/// review comment, together with the subject it was filed under.
///
/// `subject` is always `nomos_model::Subject_Of_Path("")` today -- a connector's own fact
/// about the finding itself has no Nomos subject until a bears-on relation names a
/// narrower one, per `ARC-CONNECTOR-001`'s "What This Record Does Not Do". Nothing in the
/// review comment's own fields -- not even `path`, which names a file in the *reviewed*
/// repository, not necessarily a path this workspace's own analysis can address -- is
/// treated as that relation; `path` travels as payload data describing the finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewFindingFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// This connector could not produce a fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectorError
{
    Fetch(FetchError),
    Translation(TranslationError),
}

impl core::fmt::Display for ConnectorError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Fetch(error) => write!(formatter, "{error}"),
            Self::Translation(error) => write!(formatter, "{error}"),
        };
    }
}

impl From<FetchError> for ConnectorError
{
    fn from(error: FetchError) -> Self
    {
        return Self::Fetch(error);
    }
}

impl From<TranslationError> for ConnectorError
{
    fn from(error: TranslationError) -> Self
    {
        return Self::Translation(error);
    }
}

/// Fetches one review comment live and materializes the one fact this capability answers
/// for it -- a leaf: nothing here reads another fact this or any other provider produced.
///
/// Composes [`crate::fetching::Fetch_Review_Comment`] (the generic-connector-substrate side
/// of `ARC-CONNECTOR-001`'s seam) with [`crate::translation::Translate_Review_Comment`] (the
/// vendor-to-canonical side) rather than doing either itself, the same layering
/// `OD-CONNECTOR-002`'s fixture rule depends on.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer: a composition root
/// calls this and writes the returned [`ReviewFindingFact`] into the store it owns, the
/// same division this workspace's other real providers draw.
///
/// # Errors
///
/// Whatever [`Fetch_Review_Comment`] or [`Translate_Review_Comment`] returns.
pub fn Materialize_Review_Comment<P: ProcessLauncher>(
    repository: &str,
    comment_id: u64,
    context: FactContext,
    launcher: &P,
) -> Result<ReviewFindingFact, ConnectorError>
{
    let vendor_bytes = Fetch_Review_Comment(repository, comment_id, launcher)?;
    let payload = Translate_Review_Comment(repository, &vendor_bytes)?;

    return Ok(Fact_Of(&payload, context));
}

/// Builds the fact for an already-translated payload -- split from
/// [`Materialize_Review_Comment`] so [`crate::determinism::ReviewFindingProduction`]'s own
/// recorded-fixture test can exercise exactly the deterministic half: translation and
/// encoding, with no process run.
#[must_use]
pub fn Fact_Of(payload: &FindingPayload, context: FactContext) -> ReviewFindingFact
{
    let subject = nomos_model::Subject_Of_Path("");
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Payload(payload);
    let key = Keyed(subject, &payload.external_id, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Observed,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload_bytes),
    };

    return ReviewFindingFact { subject, fact };
}

/// The key this fact is filed under.
///
/// `semantic_inputs` digests the finding's own external identity: two different findings
/// must file under two different keys even though both share the constant placeholder
/// subject every connector fact carries today.
fn Keyed(subject: SubjectId, external_id: &crate::identity::ReviewFindingId, guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[external_id.As_Str().as_bytes()]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Digest128;

    fn Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
            generation: GenerationId::INITIAL,
        };
    }

    fn Sample_Payload() -> FindingPayload
    {
        return Translate_Review_Comment(
            "coderabbitai/rabbits-playground",
            crate::fixture::Sample_Review_Comment_Response(),
        )
        .expect("this crate's own recorded fixture");
    }

    /// One real, recorded-fixture translation, checked for every property this crate
    /// promises at once -- the fixture-replay half of `OD-CONNECTOR-002`'s two-test shape.
    /// The live-`gh`-call half is `Test_A_Real_Live_Fetch_Should_Materialize_And_Decode`,
    /// `#[ignore]`d so this default suite runs with no network and no token.
    #[test]
    fn Test_Fact_Of_Should_Materialize_The_Recorded_Fixture()
    {
        let ReviewFindingFact { subject, fact } = Fact_Of(&Sample_Payload(), Context());

        assert_eq!(subject, nomos_model::Subject_Of_Path(""));
        assert_eq!(fact.evidence, EvidenceClass::Observed, "ARC-CONNECTOR-001 invariant 3");
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());

        let decoded = crate::payload::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(
            decoded.external_id.As_Str(),
            "coderabbitai/rabbits-playground#review-comment:3521038097"
        );
        assert_eq!(decoded.severity, "🟡 Minor");
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_External_Identity()
    {
        let subject = nomos_model::Subject_Of_Path("");
        let guarantee = Declared_Guarantee();

        let one = crate::identity::ReviewFindingId::Of_Review_Comment("coderabbitai/rabbits-playground", 3_521_038_097);
        let other = crate::identity::ReviewFindingId::Of_Review_Comment("coderabbitai/rabbits-playground", 3_521_038_104);

        let first_key = Keyed(subject, &one, guarantee, Context());
        let second_key = Keyed(subject, &other, guarantee, Context());

        assert_ne!(first_key.Digest(), second_key.Digest(), "two different findings must file apart");
    }

    /// The one test in this crate that touches a network. Requires no `CodeRabbit`
    /// credential at all -- reading an already-posted review comment is an ordinary,
    /// public GitHub read, the same `gh` authentication this connector's own live test
    /// requires. `#[ignore]`d so `cargo test -p nomos-connector-coderabbit` runs with no
    /// network by default, and run explicitly (`cargo test -p nomos-connector-coderabbit --
    /// --ignored`) to prove this connector against the real external system, per
    /// `OD-CONNECTOR-002`: this is the `Observed` claim the replayed suite above is not
    /// entitled to make on its own.
    #[test]
    #[ignore = "requires a live, authenticated gh CLI and network access"]
    fn Test_A_Real_Live_Fetch_Should_Materialize_And_Decode()
    {
        let ReviewFindingFact { subject, fact } = Materialize_Review_Comment(
            "coderabbitai/rabbits-playground",
            3_521_038_097,
            Context(),
            &nomos_platform_std::StdProcessLauncher,
        )
        .expect("gh must be reachable and the comment must still exist");

        assert_eq!(subject, nomos_model::Subject_Of_Path(""));
        assert_eq!(fact.evidence, EvidenceClass::Observed);

        let decoded = crate::payload::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.external_system, "coderabbit");
        assert_eq!(
            decoded.external_id.As_Str(),
            "coderabbitai/rabbits-playground#review-comment:3521038097"
        );
        assert_eq!(decoded.path, "modules/security/main.tf");
        assert!(!decoded.severity.is_empty());
        assert!(!decoded.category.is_empty());
    }
}
