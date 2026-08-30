//! Turning discovered diagnostics into the facts this capability answers.

use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::reading::{ClippyError, Discover_Workspace};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_lint::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use nomos_platform::ProcessLauncher;
use std::path::Path;

#[path = "provider/diagnostics_fact.rs"]
mod diagnostics_fact;

pub use diagnostics_fact::DiagnosticsFact;

/// Where in the workspace's history a fact is being produced — the same four-field shape
/// `nomos_lang_rust_cargo::FactContext` and `nomos_lang_rust::FactContext` both carry, for
/// the identical reason: these four always travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// Runs `cargo clippy` over `root` and produces one fact per workspace member `cargo
/// clippy` named, each a leaf: nothing here reads another fact this or any other provider
/// produced.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, the same division
/// `nomos_lang_rust_cargo::Materialize_Workspace` draws for the identical reason: a
/// composition root calls this, loops over the returned [`DiagnosticsFact`]s, and writes
/// each into the store it owns.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Launcher: ProcessLauncher>(
    root: &Path,
    context: FactContext,
    launcher: &Launcher,
) -> Result<Vec<DiagnosticsFact>, ClippyError>
{
    let discovered = Discover_Workspace(root, launcher)?;

    return Ok(discovered
        .into_iter()
        .map(|member| {
            let subject = nomos_model::Subject_Of_Path(&member.manifest_relative_root);
            let guarantee = Declared_Guarantee();
            let payload_bytes = Encode_Payload(&member.payload);
            let key = Compute_Fact_Key(subject, guarantee, context);
            let fact = MaterializedFact {
                identity: key.At(context.generation),
                snapshot: context.snapshot,
                evidence: EvidenceClass::Verified,
                guarantee,
                payload: FactPayload::New(Payload_Schema(), payload_bytes),
            };

            return DiagnosticsFact { subject, path: member.manifest_relative_root, fact };
        })
        .collect());
}

/// The key this member's fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the same choice
/// `nomos_lang_rust_cargo::fact_context::Compute_Fact_Key` already makes for the identical reason: this
/// provider's real input is `cargo clippy`'s own analysis, which no caller has
/// independently, so a caller building a lookup key has nothing to reconstruct it from —
/// `subject` alone addresses what this key needs, and a change in what `cargo clippy`
/// reports is a new run's fact, addressed by generation, not by a second axis this
/// capability has no independent input to compute one from.
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
    use nomos_contracts::Digest128;
    use nomos_platform_std::StdProcessLauncher;

    const MANY_WORKSPACE_MEMBERS: usize = 10;

    /// One real, whole-workspace `cargo clippy` invocation, checked for every property this
    /// crate promises at once -- not split across several `#[test]`s the way
    /// `nomos_lang_rust_cargo::provider::tests` is, because that crate's own `cargo
    /// metadata` call is a manifest read and this one is a real clippy pass over the whole
    /// workspace: cheap to repeat there, not here.
    #[test]
    fn Test_Materialize_Workspace_Over_This_Repository()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context(), &StdProcessLauncher).expect("this repository is a real cargo workspace under clippy");

        assert!(
            facts.len() > MANY_WORKSPACE_MEMBERS,
            "this repository has far more than ten workspace members: {}",
            facts.len()
        );

        let rules = facts
            .iter()
            .find(|fact| return fact.path == "crates/rules/nomos-rules")
            .expect("nomos-rules is a workspace member");
        assert_eq!(rules.subject, nomos_model::Subject_Of_Path("crates/rules/nomos-rules"));
        assert_eq!(rules.fact.guarantee, Declared_Guarantee());
        assert_eq!(rules.fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(rules.fact.payload.schema, Payload_Schema());

        let decoded = nomos_cap_lint::Parse_Payload(&rules.fact.payload.bytes).expect("this crate's own encoding");
        assert_eq!(decoded.package, "nomos-rules");
    }

    fn Repository_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(std::path::Path::parent)
            .and_then(std::path::Path::parent)
            .map(std::path::PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = nomos_model::Subject_Of_Path("crates/rules/nomos-rules");
        let weaker = nomos_contracts::Guarantee::New(
            nomos_contracts::FactVariant::Syntactic,
            nomos_contracts::Assurance::Unsound,
            nomos_contracts::Assurance::Unknown,
            nomos_contracts::IncrementalGranularity::WholeWorkspace,
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
