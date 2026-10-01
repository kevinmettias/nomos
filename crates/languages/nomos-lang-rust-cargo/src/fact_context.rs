//! Turning a discovered workspace into the facts this capability answers.

use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::metadata::{Discover_Workspace, MetadataError};
use nomos_cap_dependency::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use nomos_platform::{Environment, ProgramLauncher};
use std::path::Path;

#[path = "provider/package_fact.rs"]
mod package_fact;

pub use package_fact::PackageFact;

/// Where in the workspace's history a fact is being produced.
///
/// The same shape `nomos_lang_rust::FactContext` carries, for the same reason: these four
/// always travel together, and a call site that transposed two of them would compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// Runs `cargo metadata` over `root` and produces one fact per workspace member, each a
/// leaf: nothing here reads another fact this or any other provider produced, so every
/// call to `store.Materialize` a caller makes from this function's output should pass no
/// dependency edges, the same shape `nomos_lang_rust::Materialize_Syntax_Fact`'s own doc comment
/// states for the identical reason.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer: this crate is
/// composed into `nomos-check-orchestration`'s `Run` through that crate's own
/// `crate::facts::Materialize_Dependencies`, which calls this function over the tree a
/// check run walked and then loops over the returned `PackageFact`s itself, calling
/// `store.Materialize` for each one. Writing to a store is that composition root's
/// responsibility, the same way it already owns `Materialize_Syntax`'s writes; a function
/// here that took `&mut MemoryFactStore` would be duplicating a decision `Run` already
/// makes, not filling a gap it left open.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path,
    context: FactContext,
    launcher: &Launcher,
    environment: &Env,
) -> Result<Vec<PackageFact>, MetadataError>
{
    let discovered = Discover_Workspace(root, launcher, environment)?;

    return Ok(discovered
        .into_iter()
        .map(|package| {
            let subject = nomos_model::Subject_Of_Path(&package.manifest_relative_root);
            let guarantee = Declared_Guarantee();
            let payload_bytes = Encode_Payload(&package.payload);
            let key = Compute_Fact_Key(subject, guarantee, context);
            let fact = MaterializedFact {
                identity: key.At(context.generation),
                snapshot: context.snapshot,
                evidence: EvidenceClass::Verified,
                guarantee,
                payload: FactPayload::New(Payload_Schema(), payload_bytes),
            };

            return PackageFact {
                subject,
                path: package.manifest_relative_root,
                fact,
            };
        })
        .collect());
}

/// The key this package's fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, unlike `nomos_lang_rust::Materialize_Syntax_Fact`'s own
/// key -- that provider's semantic input is `source.text`, bytes the *caller* already
/// holds and can recompute the identical digest from without asking the provider anything.
/// This provider's real input is Cargo's own resolution of a manifest, which no caller has
/// independently; a caller building a lookup key from `Encode_Payload`'s own output, the
/// way an earlier version of this function did, could only ever reproduce a key by already
/// knowing the answer, which is not a key a `FactReader::Require` caller can ever
/// construct. `IncrementalGranularity::Project` already says there is nothing below "the
/// whole package" to distinguish, so `subject` alone carries what this key needs to
/// address; a change in what cargo resolves is a new run's fact, addressed by generation,
/// not by a second axis this capability has no independent input to compute one from.
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
    use nomos_platform_std::{StdEnvironment, StdProgramLauncher};

    /// Fill bytes distinct enough that `Context()`'s three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    #[test]
    fn Test_Materialize_Workspace_Should_Produce_One_Fact_Per_Workspace_Member()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context(), &StdProgramLauncher, &StdEnvironment).expect("a real workspace");

        assert!(
            facts.iter().any(|fact| fact.fact.payload.bytes.starts_with(b"package\tnomos-rules\n")),
            "expected a fact for nomos-rules among {} facts",
            facts.len()
        );
    }

    // Test_A_Facts_Subject_Should_Match_Subject_Of_Its_Own_Path moved to
    // tests/integration_seams.rs: it exercises `nomos_model::Subject_Of_Path` through this
    // crate's own public API, so it belongs in an external test rather than a private
    // inline one. See that file's own doc comment for why.

    #[test]
    fn Test_A_Facts_Path_Should_Be_What_Its_Subject_Was_Addressed_By()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context(), &StdProgramLauncher, &StdEnvironment).expect("a real workspace");

        let rules = facts
            .iter()
            .find(|fact| fact.fact.payload.bytes.starts_with(b"package\tnomos-rules\n"))
            .expect("nomos-rules is a workspace member");

        assert_eq!(rules.path, "crates/rules/nomos-rules");
        assert_eq!(rules.subject, nomos_model::Subject_Of_Path(&rules.path));
    }

    #[test]
    fn Test_A_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let facts = Materialize_Workspace(&Repository_Root(), Context(), &StdProgramLauncher, &StdEnvironment).expect("a real workspace");

        for fact in &facts
        {
            assert_eq!(fact.fact.guarantee, Declared_Guarantee());
            assert_eq!(fact.fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
            assert_eq!(fact.fact.payload.schema, Payload_Schema());
        }
    }

    #[test]
    fn Test_Two_Runs_Over_The_Same_Tree_Should_Reach_The_Same_Semantic_Inputs()
    {
        let first = Materialize_Workspace(&Repository_Root(), Context(), &StdProgramLauncher, &StdEnvironment).expect("a real workspace");
        let second = Materialize_Workspace(&Repository_Root(), Context(), &StdProgramLauncher, &StdEnvironment).expect("a real workspace");

        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter())
        {
            assert_eq!(left.fact.Key().semantic_inputs, right.fact.Key().semantic_inputs);
            assert_eq!(left.fact.Key().Digest(), right.fact.Key().Digest());
        }
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
