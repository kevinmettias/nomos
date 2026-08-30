//! Turning a discovered Go workspace into the facts this capability answers.

use crate::discovery::{DiscoveredModule, Discover_Workspace, ModuleError};
use crate::guarantee::{Declared_Guarantee, PROVIDER};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_dependency::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use std::path::Path;

#[path = "provider/module_fact.rs"]
mod module_fact;

pub use module_fact::ModuleFact;

/// Where in the workspace's history a fact is being produced.
///
/// The same shape `nomos_lang_rust_cargo::FactContext` carries, for the same reason: these
/// four always travel together, and a call site that transposed two of them would compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// Reads `root`'s own Go workspace and produces one fact per module, each a leaf: nothing
/// here reads another fact this or any other provider produced, so every call to
/// `store.Materialize` a caller makes from this function's output should pass no
/// dependency edges, the same shape `nomos_lang_rust_cargo::Materialize_Workspace`'s own
/// doc states for the identical reason.
///
/// Deliberately not itself a `nomos_analysis::MemoryFactStore` writer, for the identical
/// reason `nomos_lang_rust_cargo::Materialize_Workspace` gives: writing to a store is a
/// composition root's responsibility, not this function's.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace(root: &Path, context: FactContext) -> Result<Vec<ModuleFact>, ModuleError>
{
    let discovered = Discover_Workspace(root)?;

    return Ok(discovered.into_iter().map(|module| Fact_Of(module, context)).collect());
}

fn Fact_Of(module: DiscoveredModule, context: FactContext) -> ModuleFact
{
    let subject = nomos_model::Subject_Of_Path(&module.manifest_relative_root);
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Payload(&module.payload);
    let key = Compute_Fact_Key(subject, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload_bytes),
    };

    return ModuleFact {
        subject,
        path: module.manifest_relative_root,
        fact,
    };
}

/// The key this module's fact is filed under.
///
/// `semantic_inputs` is empty, the identical reasoning
/// `nomos_lang_rust_cargo::fact_context::Compute_Fact_Key` gives for its own capability: this provider's
/// real input is what `go.work`/`go.mod` declare, which no caller has independently, so a
/// lookup key built from `Encode_Payload`'s own output could only ever reproduce a key by
/// already knowing the answer.
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

    struct TemporaryWorkspace
    {
        root: std::path::PathBuf,
    }

    impl TemporaryWorkspace
    {
        /// A fresh, uniquely-named directory per call, not merely per process: these
        /// tests run concurrently within one process, and a name shared across them would
        /// let one test's `Drop` remove a directory a sibling test is still reading from
        /// or writing to.
        ///
        /// Named from the thread id and a wall-clock timestamp rather than a shared
        /// counter, so uniqueness needs no global mutable state with an owner to name: two
        /// concurrently live threads never share a `ThreadId`, and two calls on one reused
        /// worker thread never land on the same nanosecond, so the pair together can never
        /// collide the way two calls racing a shared counter's read-modify-write could.
        fn New() -> Self
        {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "nomos-lang-go-modules-provider-test-{}-{:?}-{stamp}",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::create_dir_all(&root).expect("a fresh temp directory can be created");

            return Self { root };
        }

        fn Write(&self, relative: &str, content: &str)
        {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent()
            {
                std::fs::create_dir_all(parent).expect("the fixture's own parent exists");
            }
            std::fs::write(&path, content).expect("the fixture file can be written");
        }
    }

    impl Drop for TemporaryWorkspace
    {
        /// Drop cannot propagate a failure, so a failed removal is reported rather than
        /// silently discarded; each root is uniquely named, so the worst case is one
        /// leaked directory, not a sibling test's fixture disappearing out from under it.
        fn drop(&mut self)
        {
            if let Err(error) = std::fs::remove_dir_all(&self.root)
            {
                eprintln!("failed to remove temporary workspace {}: {error}", self.root.display());
            }
        }
    }

    /// Fill bytes distinct enough that `Context()`'s three digests differ from one
    /// another; each value carries no meaning beyond "not equal to the others".
    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    #[test]
    fn Test_Materialize_Workspace_Should_Produce_One_Fact_Per_Module()
    {
        let workspace = TemporaryWorkspace::New();
        workspace.Write("go.mod", "module example.com/solo\n");

        let facts = Materialize_Workspace(&workspace.root, Context()).expect("a real workspace");

        assert_eq!(facts.len(), 1);
        let fact = facts.first().expect("just asserted len() == 1");
        assert!(fact.fact.payload.bytes.starts_with(b"package\texample.com/solo\n"));
    }

    #[test]
    fn Test_A_Facts_Subject_Should_Match_Subject_Of_Its_Own_Path()
    {
        let workspace = TemporaryWorkspace::New();
        workspace.Write("go.mod", "module example.com/solo\n");

        let facts = Materialize_Workspace(&workspace.root, Context()).expect("a real workspace");
        let fact = facts.first().expect("the solo module produced one fact");

        assert_eq!(fact.subject, nomos_model::Subject_Of_Path(&fact.path));
        assert_eq!(fact.path, "");
    }

    #[test]
    fn Test_A_Fact_Should_Carry_The_Declared_Guarantee()
    {
        let workspace = TemporaryWorkspace::New();
        workspace.Write("go.mod", "module example.com/solo\n");

        let facts = Materialize_Workspace(&workspace.root, Context()).expect("a real workspace");

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
        let workspace = TemporaryWorkspace::New();
        workspace.Write("go.mod", "module example.com/solo\n");

        let first = Materialize_Workspace(&workspace.root, Context()).expect("a real workspace");
        let second = Materialize_Workspace(&workspace.root, Context()).expect("a real workspace");

        assert_eq!(first.len(), second.len());
        for (left, right) in first.iter().zip(second.iter())
        {
            assert_eq!(left.fact.Key().semantic_inputs, right.fact.Key().semantic_inputs);
            assert_eq!(left.fact.Key().Digest(), right.fact.Key().Digest());
        }
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
