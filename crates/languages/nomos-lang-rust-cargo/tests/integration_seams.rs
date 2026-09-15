//! The real crate-to-crate seams `nomos-lang-rust-cargo` reaches across, driven only
//! through this crate's own public API -- the same view a real consumer has.
//!
//! `nomos-lang-rust-cargo` depends on `nomos_contracts`, `nomos_model`, `nomos_capability`,
//! `nomos_analysis`, `nomos_cap_dependency` and `nomos_platform` in its source.
//! `nomos_cap_dependency` and `nomos_model` each already had an inline `#[cfg(test)]`
//! exercising them, but only through this crate's own private access -- both are moved
//! here, thin and unchanged in what they prove. `nomos_capability` and `nomos_platform`
//! had no test anywhere reaching them; both get one here.
//!
//! `nomos_platform` is exercised through a fake [`ProcessLauncher`] rather than a real
//! `cargo metadata` subprocess (`tests/invalidation.rs` and this crate's own remaining
//! inline tests already pay for that real invocation): a fake is what makes this the real
//! trait boundary check rather than a second slow copy of an existing one, and it lets the
//! `nomos_model` seam below reuse the same fabricated document instead of shelling out
//! again.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_rust_cargo::{Declared_Guarantee, Discover_Workspace, FactContext, Materialize_Workspace, Provider_Offer};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::path::{Path, PathBuf};
use nomos_platform_std::StdEnvironment;

/// A [`ProcessLauncher`] that never runs anything -- it returns a canned answer regardless
/// of what `command` names, which is what makes the boundary check here cheap and
/// deterministic instead of a second real `cargo metadata` subprocess.
struct FakeLauncher
{
    outcome: ExitOutcome,
    stdout: String,
    stderr: String,
}

impl FakeLauncher
{
    fn Succeeding(stdout: &str) -> Self
    {
        return Self {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: stdout.to_owned(),
            stderr: String::new(),
        };
    }

    fn Failing(stderr: &str) -> Self
    {
        return Self {
            outcome: ExitOutcome::Exited { code: 1 },
            stdout: String::new(),
            stderr: stderr.to_owned(),
        };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for FakeLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        return Ok(ProcessOutput {
            outcome: self.outcome,
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
        });
    }
}

/// A minimal, fabricated `cargo metadata --format-version 1` document: two workspace
/// members, `alpha` depending on `beta` -- the exact shape `Discover_Workspace` reads
/// (`workspace_root`, `workspace_members`, and each `packages` entry's `id`/`name`/
/// `manifest_path`/`dependencies`), small enough to read at a glance rather than a real
/// `cargo metadata` dump's hundreds of fields.
///
/// `workspace_root` must be `root` itself, byte for byte before `serde_json` re-encodes
/// it: `Discover_Workspace` now canonicalizes both sides and refuses if they disagree
/// (`P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`), so a fixture whose `workspace_root`
/// does not correspond to a real, canonicalizable directory matching `root` would be
/// refused as an escape rather than read as the well-formed document it is meant to be.
///
/// It names two workspace members, `alpha` and `beta` -- the count every test that reads
/// this fixture's own output back asserts against.
const FABRICATED_MEMBERS: usize = 2;

fn Fake_Metadata_Document(root: &Path) -> String
{
    let workspace_root = root.to_string_lossy().into_owned();
    let alpha_manifest = root.join("alpha").join("Cargo.toml").to_string_lossy().into_owned();
    let beta_manifest = root.join("beta").join("Cargo.toml").to_string_lossy().into_owned();

    return serde_json::json!({
        "workspace_root": workspace_root,
        "workspace_members": [
            "alpha 0.1.0 (path+file:///workspace/alpha)",
            "beta 0.1.0 (path+file:///workspace/beta)"
        ],
        "packages": [
            {
                "id": "alpha 0.1.0 (path+file:///workspace/alpha)",
                "name": "alpha",
                "manifest_path": alpha_manifest,
                "dependencies": [
                    { "name": "beta", "kind": null, "optional": false }
                ]
            },
            {
                "id": "beta 0.1.0 (path+file:///workspace/beta)",
                "name": "beta",
                "manifest_path": beta_manifest,
                "dependencies": []
            }
        ]
    })
    .to_string();
}

/// A real, empty scratch directory for a fabricated `cargo metadata` document's own
/// `"workspace_root"` to resolve to. `Require_Workspace_Root_Is` canonicalizes both sides
/// for real (it must, to catch a real escape), so this fixture needs a real directory on
/// disk to canonicalize against -- unlike `Fake_Metadata_Document`'s member paths, which
/// `Manifest_Relative_Root` only ever strips a textual prefix from and never touches disk
/// for.
struct FakeWorkspaceRoot
{
    path: PathBuf,
}

impl FakeWorkspaceRoot
{
    fn New(name: &str) -> Self
    {
        let path = std::env::temp_dir().join(format!("nomos-lang-rust-cargo-fake-workspace-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory for the fixture's own workspace_root");
        let canonical = std::fs::canonicalize(&path).expect("the directory this call just created");

        return Self { path: canonical };
    }

    fn Path(&self) -> &Path
    {
        return &self.path;
    }
}

impl Drop for FakeWorkspaceRoot
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.path);
    }
}

/// `Discover_Workspace`'s own `Require_Clean_Exit` check runs before `Require_Workspace_
/// Root_Is` ever reads a document, so a launcher that never produces one does not need a
/// fixture that could canonicalize -- this stays the simple, non-existent placeholder
/// `Test_Discover_Workspace_Should_Report_The_Launchers_Own_Stderr_On_A_Nonzero_Exit`
/// alone still needs.
fn Fake_Workspace_Root() -> PathBuf
{
    return PathBuf::from("/workspace");
}

/// Fill bytes distinct enough that the three digests below differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

// -- nomos_platform ------------------------------------------------------------------

/// `nomos_platform`: `Discover_Workspace` reads a real `nomos_platform::ProcessLauncher`
/// implementation's output through the real trait boundary -- the happy path, over a
/// fake but well-formed `cargo metadata` document.
#[test]
fn Test_Discover_Workspace_Should_Read_Packages_And_Edges_From_A_Fake_Launchers_Own_Output()
{
    let workspace_root = FakeWorkspaceRoot::New("read-packages");
    let launcher = FakeLauncher::Succeeding(&Fake_Metadata_Document(workspace_root.Path()));

    let discovered = Discover_Workspace(workspace_root.Path(), &launcher, &StdEnvironment).expect("a well-formed fake metadata document");

    assert_eq!(discovered.len(), FABRICATED_MEMBERS);
    let alpha = discovered
        .iter()
        .find(|package| package.payload.package == "alpha")
        .expect("alpha is a fabricated workspace member");
    assert_eq!(alpha.manifest_relative_root, "alpha");
    assert!(
        alpha
            .payload
            .edges
            .iter()
            .any(|edge| edge.target == "beta" && edge.kind == nomos_cap_dependency::DependencyKind::Normal && !edge.optional),
        "got {:?}",
        alpha.payload.edges
    );
}

/// (the launcher's own stderr, the fragment `Discover_Workspace`'s own error must contain)
/// for a launcher reporting a nonzero exit -- a second case beside this one would extend
/// the table rather than duplicate the test.
fn Nonzero_Exit_Cases() -> Vec<(&'static str, &'static str)>
{
    return vec![("a fabricated cargo failure", "a fabricated cargo failure")];
}

/// `nomos_platform`: the error that crosses this boundary -- a launcher reporting a
/// nonzero exit becomes `Discover_Workspace`'s own `MetadataError`, carrying the
/// launcher's own stderr, not a panic and not a silently empty result.
#[test]
fn Test_Discover_Workspace_Should_Report_The_Launchers_Own_Stderr_On_A_Nonzero_Exit()
{
    for (stderr, expected_fragment) in Nonzero_Exit_Cases()
    {
        let launcher = FakeLauncher::Failing(stderr);

        let error = Discover_Workspace(&Fake_Workspace_Root(), &launcher, &StdEnvironment).expect_err("a nonzero exit must not be read as success");

        assert!(error.reason.contains(expected_fragment), "{}", error.reason);
    }
}

// -- nomos_capability ------------------------------------------------------------------

/// `nomos_capability`: this crate's own offer is accepted under `nomos_cap_dependency`'s
/// contract through `nomos_capability::Registry` -- the composition-time check every real
/// caller relies on. Previously reached by no test in this crate at all.
#[test]
fn Test_This_Crates_Offer_Should_Be_Accepted_By_Nomos_Capabilitys_Own_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// `nomos_capability`: the lifecycle the registry imposes -- a second declaration of the
/// same capability is refused by name, not silently replaced.
#[test]
fn Test_A_Second_Declaration_Of_Nomos_Cap_Dependencys_Contract_Should_Be_Refused_By_The_Registry()
{
    use nomos_capability::{Registry, RegistryErrorKind};

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect("the first declaration succeeds");

    let error = registry
        .Declare(nomos_cap_dependency::Capability_Contract())
        .expect_err("a capability declared twice must be refused, not silently replaced");

    assert_eq!(error.kind, RegistryErrorKind::AlreadyDeclared);
}

// -- nomos_cap_dependency (moved from src/guarantee.rs's own inline test) -----------------

/// `nomos_cap_dependency`: this crate's own claimed guarantee satisfies the capability's
/// ceiling -- moved here from an inline test so it is checked through the public API a
/// real consumer has, unchanged in what it proves.
#[test]
fn Test_The_Declared_Guarantee_Should_Satisfy_Nomos_Cap_Dependencys_Own_Contract_Ceiling()
{
    use nomos_cap_dependency::Ceiling;

    assert!(Declared_Guarantee().Satisfies(&Ceiling()));
}

// -- nomos_model (moved from src/fact_context.rs's own inline test) -----------------------

/// `nomos_model`: a fact's subject is `nomos_model::Subject_Of_Path` of its own manifest-
/// relative path -- moved here from an inline test so it is checked through the public API
/// a real consumer has, unchanged in what it proves, and over the same fake document the
/// `nomos_platform` tests above already built so this crate's boundary tests do not each
/// pay for their own real `cargo metadata` subprocess.
#[test]
fn Test_A_Facts_Subject_Should_Match_Nomos_Models_Own_Subject_Of_Its_Path()
{
    let workspace_root = FakeWorkspaceRoot::New("facts-subject");
    let launcher = FakeLauncher::Succeeding(&Fake_Metadata_Document(workspace_root.Path()));

    let facts = Materialize_Workspace(workspace_root.Path(), Context(), &launcher, &StdEnvironment).expect("a well-formed fake metadata document");

    let alpha = facts
        .iter()
        .find(|fact| fact.path == "alpha")
        .expect("alpha is a fabricated workspace member");

    assert_eq!(alpha.subject, nomos_model::Subject_Of_Path("alpha"));
}
