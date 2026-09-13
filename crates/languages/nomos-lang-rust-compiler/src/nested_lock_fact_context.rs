//! Turning a real `ra_ap_hir` analysis into this crate's second fact.

use crate::guarantee::PROVIDER;
use crate::nested_lock_contract::{Capability, Payload_Schema, CONTRACT_VERSION};
use crate::nested_lock_guarantee::Declared_Guarantee;
use crate::nested_lock_reading::Discover_Nested_Locks;
use nomos_platform::Environment;
use crate::payload::Encode_Nested_Lock_Payload;
use crate::payload::nested_lock_payload::NestedLockPayload;
use crate::provider::FactContext;
use crate::reading::CompilerError;
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{EvidenceClass, Guarantee, ProviderId, SubjectId};
use std::path::Path;

/// One materialized fact, addressed to the crate it was produced for -- the same
/// `{subject, fact}` pairing [`crate::provider::CloneOnCopyFact`] returns, for the
/// identical reason.
pub struct NestedLockFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// Analyzes the crate rooted at `root` through `ra_ap_hir` and produces the one fact
/// [`nomos_contracts::IncrementalGranularity::Project`] allows for it -- a leaf: nothing
/// here reads another fact this or any other provider produced.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, the same division
/// [`crate::provider::Materialize_Crate`] draws for the identical reason: a composition
/// root calls this and writes the returned [`NestedLockFact`] into the store it owns.
///
/// # Errors
///
/// Whatever [`Discover_Nested_Locks`] returns.
pub fn Materialize_Nested_Locks<Env: Environment>(root: &Path, context: FactContext, environment: &Env) -> Result<NestedLockFact, CompilerError>
{
    let findings = Discover_Nested_Locks(root, environment)?;
    let payload = NestedLockPayload { findings };
    let subject = nomos_model::Subject_Of_Path(&root.to_string_lossy());
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Nested_Lock_Payload(&payload);
    let key = Compute_Fact_Key(subject, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload_bytes),
    };

    return Ok(NestedLockFact { subject, fact });
}

/// The key this fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the same choice `crate::provider`'s own
/// `Compute_Fact_Key` already makes for the identical reason: this provider's real input
/// is a real compiler frontend's own analysis of every file reachable from `root` plus
/// the standard library it resolved `Mutex`/`RwLock` against, which no caller has
/// independently, so a caller building a lookup key has nothing to reconstruct it from.
fn Compute_Fact_Key(subject: SubjectId, guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[]),
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
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};

    #[test]
    fn Test_Materialize_Nested_Locks_Should_Find_The_Real_Fixtures_Own_Finding()
    {
        let NestedLockFact { subject, fact } =
            Materialize_Nested_Locks(&Fixture_Root(), Context(), &nomos_platform_std::StdEnvironment).expect("this crate's own fixture is a real, loadable Cargo project");

        assert_eq!(subject, nomos_model::Subject_Of_Path(&Fixture_Root().to_string_lossy()));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());

        let decoded = crate::payload::Parse_Nested_Lock_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.findings.len(), 1, "{decoded:?}");
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = nomos_model::Subject_Of_Path("");
        let weaker = nomos_contracts::Guarantee::New(
            nomos_contracts::FactVariant::Syntactic,
            nomos_contracts::Assurance::Unsound,
            nomos_contracts::Assurance::Unknown,
            nomos_contracts::IncrementalGranularity::Project,
        );

        let strong_key = Compute_Fact_Key(subject, Declared_Guarantee(), Context());
        let weak_key = Compute_Fact_Key(subject, weaker, Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    fn Fixture_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.join("fixtures").join("nested_lock_sample");
    }

    /// Fill bytes distinct enough that `Context()`'s three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    fn Context() -> FactContext
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
