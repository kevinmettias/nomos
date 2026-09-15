//! The real crate-to-crate seams `nomos-lang-go-modules` reaches across, driven only
//! through this crate's own public API — the same view a real consumer has.
//!
//! `nomos-lang-go-modules` depends on five other crates in its source: `nomos_analysis`,
//! `nomos_cap_dependency`, `nomos_capability`, `nomos_contracts` and `nomos_model`. Each
//! test below exercises the actual call this crate makes into one of them, either as a new
//! seam this crate had no external coverage for at all (`nomos_analysis`,
//! `nomos_cap_dependency`), or moved here from an inline `#[cfg(test)]` module so the same
//! property is checked through the public API a real consumer has rather than only through
//! this crate's own private access (`nomos_capability`, `nomos_contracts`, `nomos_model`).

use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_go_modules::{
    Declared_Guarantee, Discover_Workspace, FactContext, Materialize_Workspace, ModuleFact, Provider_Offer,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh, uniquely-named directory under the system temp directory, so concurrently
/// running tests in this file never collide — the same fixture shape this crate's own
/// `discovery` tests use internally, written fresh here because this file reaches only the
/// crate's public API and has no access to that private helper.
struct IntegrationWorkspace
{
    root: PathBuf,
}

impl IntegrationWorkspace
{
    fn New() -> Self
    {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nomos-lang-go-modules-integration-seams-{}-{n}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("a fresh temp directory can be created");

        return Self { root };
    }

    /// `relative` is a `&Path` rather than a `&str` so the two positions have distinct
    /// types: a caller that swapped the fixture's path for its content would be writing
    /// a file named after the module source, and only the type system would say so.
    fn Write(&self, relative: &Path, content: &str)
    {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("the fixture's own parent directory can be created");
        }
        std::fs::write(&path, content).expect("the fixture file can be written");
    }
}

impl Drop for IntegrationWorkspace
{
    /// Drop cannot propagate a failure, so a failed removal is reported rather than
    /// silently discarded; each root is uniquely named, so the worst case is one leaked
    /// directory, not a sibling test's fixture disappearing out from under it.
    fn drop(&mut self)
    {
        if let Err(error) = std::fs::remove_dir_all(&self.root)
        {
            eprintln!("failed to remove temporary workspace {}: {error}", self.root.display());
        }
    }
}

/// Fill bytes distinct enough that the three digests below differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

/// A generation past `GenerationId::INITIAL`, which is what makes a fact materialized at
/// it newer than one materialized at `INITIAL`; the value itself means nothing further.
const NEWER_GENERATION: u64 = 5;

fn Context_At(generation: GenerationId) -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation,
    };
}

fn Context() -> FactContext
{
    return Context_At(GenerationId::INITIAL);
}

/// A two-module Go workspace, `a` requiring `b` — the same shape this crate's own
/// `discovery` tests use for the identical scenario, rebuilt here from public files on
/// disk rather than shared with that private fixture.
fn Two_Module_Workspace() -> IntegrationWorkspace
{
    let workspace = IntegrationWorkspace::New();
    workspace.Write(Path::new("go.work"), "go 1.21\n\nuse (\n\t./a\n\t./b\n)\n");
    workspace.Write(
        Path::new("a/go.mod"),
        "module example.com/a\n\ngo 1.21\n\nrequire example.com/b v0.0.0\n",
    );
    workspace.Write(Path::new("b/go.mod"), "module example.com/b\n\ngo 1.21\n");

    return workspace;
}

/// The single fact a workspace declaring one `go.mod` materializes, at `generation` — the
/// fixture the solo-module tests below share.
fn Solo_Modules_Fact_At(generation: GenerationId) -> ModuleFact
{
    let workspace = IntegrationWorkspace::New();
    workspace.Write(Path::new("go.mod"), "module example.com/solo\n");

    return Materialize_Workspace(&workspace.root, Context_At(generation))
        .expect("a real workspace")
        .into_iter()
        .next()
        .expect("the solo module produced one fact");
}

// -----------------------------------------------------------------------------------------
// nomos_analysis — no external suite exercised this pairing before.
// -----------------------------------------------------------------------------------------

/// `nomos_analysis`: the fact this crate materializes is a real
/// `nomos_analysis::MaterializedFact` that `nomos_analysis`'s own store accepts, and reads
/// back under the key this crate computed — the happy-path flow across the boundary.
#[test]
fn Test_A_Modules_Materialized_Fact_Should_Be_Accepted_And_Read_Back_By_Nomos_Analysiss_Own_Store()
{
    use nomos_analysis::{FactStore, MemoryFactStore};

    let module = Solo_Modules_Fact_At(GenerationId::INITIAL);
    let key = module.fact.Key().clone();
    let mut store = MemoryFactStore::New();

    store
        .Materialize(module.fact, &[])
        .expect("nomos_analysis accepts this crate's own fact shape without complaint");

    let current = store
        .Current(&key.At(GenerationId::INITIAL), GenerationId::INITIAL)
        .expect("nomos_analysis can address the fact this crate wrote by the key this crate computed");
    assert_eq!(current.guarantee, Declared_Guarantee());
}

/// `nomos_analysis`: the lifecycle constraint the store's own generation counter imposes —
/// a fact this crate materializes behind a generation the store has already recorded is
/// refused as `FactError::Backdated`, not silently accepted as a second history entry.
#[test]
fn Test_Materializing_An_Older_Generations_Fact_Over_A_Newer_One_Should_Be_Refused_As_Backdated()
{
    use nomos_analysis::{FactError, MemoryFactStore};

    let mut store = MemoryFactStore::New();
    store
        .Materialize(Solo_Modules_Fact_At(GenerationId::From_Raw(NEWER_GENERATION)).fact, &[])
        .expect("the newer generation is written first");

    let error = store
        .Materialize(Solo_Modules_Fact_At(GenerationId::INITIAL).fact, &[])
        .expect_err("writing behind the current generation must be refused");

    assert!(matches!(error, FactError::Backdated { .. }), "{error:?}");
    assert!(
        error.to_string().contains("would be written behind"),
        "expected a backdated refusal, got: {error}"
    );
}

// -----------------------------------------------------------------------------------------
// nomos_cap_dependency — no external suite exercised this pairing before.
// -----------------------------------------------------------------------------------------

/// `nomos_cap_dependency`: this crate's own encoded payload bytes decode under the shared
/// schema reader, and the edge it discovered — a workspace member's own `require` line —
/// survives the round trip with the kind and optionality this provider's own guarantee
/// promises (`DependencyKind::Normal`, never optional).
#[test]
fn Test_A_Discovered_Edges_Payload_Should_Decode_Under_Nomos_Cap_Dependencys_Own_Reader()
{
    use nomos_cap_dependency::{DependencyEdge, DependencyKind, Parse_Payload};

    let workspace = Two_Module_Workspace();

    let facts = Materialize_Workspace(&workspace.root, Context()).expect("a real two-module workspace");
    let a = facts.iter().find(|module| module.path == "a").expect("member a was materialized");

    let payload = Parse_Payload(&a.fact.payload.bytes)
        .expect("this crate writes the schema nomos_cap_dependency's own reader expects");

    assert_eq!(payload.package, "example.com/a");
    assert_eq!(
        payload.edges,
        vec![DependencyEdge {
            target: "example.com/b".to_owned(),
            kind: DependencyKind::Normal,
            optional: false,
        }]
    );
}

/// `nomos_cap_dependency`: `Discover_Workspace`'s own output — read directly, not only
/// through the encoded fact bytes above — is already this capability's own
/// `DependencyPayload` shape, restricted to first-party edges the way the capability's own
/// contract promises: a member with no `require` line on another workspace member reports
/// no edges at all.
#[test]
fn Test_Discover_Workspace_Should_Restrict_Nomos_Cap_Dependencys_Edges_To_Workspace_Members()
{
    let workspace = Two_Module_Workspace();

    let discovered = Discover_Workspace(&workspace.root).expect("a real two-module workspace");

    let b = discovered
        .iter()
        .find(|module| module.payload.package == "example.com/b")
        .expect("member b was discovered");
    assert!(b.payload.edges.is_empty(), "b requires nothing: {:?}", b.payload.edges);
}

// -----------------------------------------------------------------------------------------
// nomos_capability — moved from an inline test that exercised this only through this
// crate's own private access.
// -----------------------------------------------------------------------------------------

/// `nomos_capability`: this crate's own offer is accepted under `nomos_cap_dependency`'s
/// contract through `nomos_capability::Registry` — the composition-time check every real
/// caller relies on, checked here through the public API a real consumer has.
#[test]
fn Test_Provider_Offer_Should_Be_Accepted_By_Nomos_Capabilitys_Own_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// `nomos_capability`: the lifecycle the registry imposes — a second declaration of the
/// same capability is refused by name, not silently replaced.
#[test]
fn Test_A_Second_Declaration_Of_The_Same_Capability_Should_Be_Refused_By_The_Registry()
{
    use nomos_capability::{Registry, RegistryErrorKind};

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect("the first declaration succeeds");

    let error = registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect_err("a capability declared twice is a defect the registry must catch, not absorb");

    assert_eq!(error.kind, RegistryErrorKind::AlreadyDeclared);
    assert!(error.to_string().contains("is already declared"), "{error}");
}

// -----------------------------------------------------------------------------------------
// nomos_contracts — moved from an inline test that exercised this only through this
// crate's own private access.
// -----------------------------------------------------------------------------------------

/// `nomos_contracts`: this crate's own declared guarantee does not satisfy a caller's floor
/// asking for more completeness than this provider claims — checked through
/// `nomos_contracts::Guarantee::Satisfies`, the exact comparison a real registry runs.
#[test]
fn Test_The_Declared_Guarantee_Should_Not_Satisfy_A_Requirement_For_Sound_Completeness()
{
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    let needs_sound_completeness = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
    );

    assert!(!Declared_Guarantee().Satisfies(&needs_sound_completeness));
}

/// `nomos_contracts`: the negative control — this guarantee does satisfy a requirement that
/// only asks for what it actually claims, so the refusal above is about the requirement's
/// own floor and not about this guarantee failing to satisfy anything at all.
#[test]
fn Test_The_Declared_Guarantee_Should_Satisfy_A_Requirement_That_Accepts_Unknown_Completeness()
{
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

    let accepts_unknown_completeness = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );

    assert!(Declared_Guarantee().Satisfies(&accepts_unknown_completeness));
}

// -----------------------------------------------------------------------------------------
// nomos_model — moved from an inline test that exercised this only through this crate's
// own private access.
// -----------------------------------------------------------------------------------------

/// `nomos_model`: the subject this crate files a module's fact under is exactly
/// `nomos_model::Subject_Of_Path` applied to the same repository-relative path this crate
/// reports back — the same construction any real caller performs before addressing the
/// fact this crate materialized.
#[test]
fn Test_A_Modules_Subject_Should_Match_Nomos_Models_Own_Subject_Of_Its_Path()
{
    let module = Solo_Modules_Fact_At(GenerationId::INITIAL);

    assert_eq!(module.subject, nomos_model::Subject_Of_Path(&module.path));
}

/// `nomos_model`: two modules at different repository-relative paths are two different
/// subjects — the lifecycle property a caller filing more than one module's fact into the
/// same store relies on to keep them apart.
#[test]
fn Test_Two_Modules_At_Different_Paths_Should_Be_Different_Nomos_Model_Subjects()
{
    let workspace = Two_Module_Workspace();

    let facts = Materialize_Workspace(&workspace.root, Context()).expect("a real two-module workspace");
    let a = facts.iter().find(|module| module.path == "a").expect("member a was materialized");
    let b = facts.iter().find(|module| module.path == "b").expect("member b was materialized");

    assert_ne!(a.subject, b.subject);
    assert_eq!(a.subject, nomos_model::Subject_Of_Path(&a.path));
    assert_eq!(b.subject, nomos_model::Subject_Of_Path(&b.path));
}
