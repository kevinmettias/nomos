//! Turning a real `ra_ap_hir` analysis into the one fact this capability answers.

use crate::contract::{Capability, Payload_Schema, CONTRACT_VERSION};
use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::payload::clone_on_copy_payload::CloneOnCopyPayload;
use crate::payload::Encode_Payload;
use crate::reading::{CompilerError, Discover_Crate};
use nomos_platform::Environment;
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId, SubjectId,
};
use std::path::Path;

#[path = "fact_context/clone_on_copy_fact.rs"]
mod clone_on_copy_fact;

pub use clone_on_copy_fact::CloneOnCopyFact;

/// Where in the workspace's history a fact is being produced -- the same four-field
/// shape `nomos_lang_rust_deny::FactContext` carries, for the identical reason: these
/// four always travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// Analyzes the crate rooted at `root` through `ra_ap_hir` and produces the one fact
/// [`IncrementalGranularity::Project`](nomos_contracts::IncrementalGranularity::Project)
/// allows for it -- a leaf: nothing here reads another fact this or any other provider
/// produced.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, the same division
/// `nomos_lang_rust_deny::Materialize_Workspace` draws for the identical reason: a
/// composition root calls this and writes the returned [`CloneOnCopyFact`] into the
/// store it owns.
///
/// # Errors
///
/// Whatever [`Discover_Crate`] returns.
pub fn Materialize_Crate<Env: Environment>(root: &Path, context: FactContext, environment: &Env) -> Result<CloneOnCopyFact, CompilerError>
{
    let findings = Discover_Crate(root, environment)?;
    let payload = CloneOnCopyPayload { findings };
    let subject = nomos_model::Subject_Of_Path(&root.to_string_lossy());
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Payload(&payload);
    let key = Compute_Fact_Key(subject, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload_bytes),
    };

    return Ok(CloneOnCopyFact { subject, fact });
}

/// The key this fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the same choice
/// `nomos_lang_rust_deny::fact_context::Compute_Fact_Key` already makes for the
/// identical reason: this provider's real input is a real compiler frontend's own
/// analysis of every file reachable from `root`, which no caller has independently, so a
/// caller building a lookup key has nothing to reconstruct it from.
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
    use crate::contract::Payload_Schema;
    use nomos_contracts::Digest128;

    #[test]
    fn Test_Materialize_Crate_Should_Find_The_Real_Fixtures_Own_Finding()
    {
        let CloneOnCopyFact { subject, fact } = Materialize_Crate(&Fixture_Root(), Context(), &nomos_platform_std::StdEnvironment).expect("this crate's own fixture is a real, loadable Cargo project");

        assert_eq!(subject, nomos_model::Subject_Of_Path(&Fixture_Root().to_string_lossy()));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, Payload_Schema());

        let decoded = crate::payload::Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.findings.len(), 1, "{decoded:?}");
    }

    fn Fixture_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.join("fixtures").join("clone_on_copy_sample");
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
