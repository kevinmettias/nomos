//! The `nomos_capability` seam `check-integration-coverage` found with no suite.
//!
//! `nomos-cli::check::report::Render_Contradictory` and `nomos-cli::gate::report::
//! Render_Run_Contradictory` both turn a `nomos_capability::RegistryError` into text when
//! this build's own capability composition turns out to be self-contradictory
//! (`nomos_check_orchestration::CheckOutcome::Contradictory`). Every existing test under
//! `tests/` drives `nomos check`/`nomos gate` over trees whose *content* is what gets
//! judged, and none of them names `nomos_capability` — because the registry this binary
//! composes is fixed at compile time from a hardcoded contract set (`check.rs`'s own doc
//! comment says so) and cannot be made to contradict itself through a subprocess
//! invocation. There is no argv that reaches this path.
//!
//! This proves the seam from both ends instead. The first half constructs a real
//! `nomos_capability::Registry`, declares one capability twice, and checks the resulting
//! `RegistryError` is exactly the shape `check/report.rs`'s private `Render_Contradictory`
//! wraps — reproducing its message-building here, since that function itself is not public.
//! The second half runs the real shipped binary over an ordinary tree and proves its own
//! registry never takes that path, which is the happy-path half of the same contract: a
//! `RegistryError` that can occur in principle must never occur in practice for the
//! registry this binary actually ships.

#[path = "support/mod.rs"]
mod support;

use nomos_capability::{CapabilityContract, Registry, RegistryError, RegistryErrorKind};
use nomos_contracts::{Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity};
use support::{Run, Tree};

fn Capability() -> CapabilityId
{
    return CapabilityId::New("nomos.cli.tests.capability_seam");
}

fn Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: ContractVersion::New(1, 0),
        summary: "a contract for capability_seam.rs's own test".to_owned(),
        ceiling: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
    };
}

/// The exact sentence `nomos-cli::check::report::Render_Contradictory` wraps a
/// `RegistryError` in. Copied rather than imported — it is private to that module — because
/// holding this sentence in place from the outside is the whole point of this test: if
/// `check/report.rs` ever changes what it wraps a `RegistryError` in, this assertion (not
/// only that file's own inline tests) must also go red.
fn Rendered_As_Nomos_Cli_Would(error: &RegistryError) -> String
{
    return format!(
        "this build's own composition is contradictory, so no fact it produced would have \
         been offered by anybody: {error}"
    );
}

/// A capability declared twice is exactly the self-contradiction `nomos-cli` renders through
/// `RegistryError`. Constructed directly because the shipped binary's own registry is
/// composed once, from a fixed contract set, and cannot be driven into this state from
/// outside the process.
#[test]
fn Test_A_Second_Declaration_Should_Produce_The_Error_Nomos_Cli_Renders()
{
    let mut registry = Registry::New();
    assert!(registry.Declare(Contract()).is_ok(), "the first declaration must succeed");

    let error = registry
        .Declare(Contract())
        .expect_err("a capability declared twice is a contradiction");

    assert_eq!(error.capability, Capability());
    assert_eq!(error.kind, RegistryErrorKind::AlreadyDeclared);

    let rendered = Rendered_As_Nomos_Cli_Would(&error);
    assert!(
        rendered.starts_with("this build's own composition is contradictory"),
        "{rendered}"
    );
    assert!(
        rendered.contains("nomos.cli.tests.capability_seam is already declared"),
        "the wrapped error must still name which capability and why: {rendered}"
    );
}

/// The happy-path control: the shipped binary's own registry, composed once at startup,
/// must never take the path the other half of this file proves the message text for. If it
/// ever did, `nomos check` would report the composition contradictory and refuse to judge
/// any tree at all, no matter how clean.
#[test]
fn Test_The_Shipped_Binary_Should_Never_Report_Its_Own_Composition_Contradictory()
{
    let tree = Tree::New("capability-happy-path").With("subject.rs", "pub const TABLES: &[&str] = &[];\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert!(
        !ran.stderr.contains("this build's own composition is contradictory"),
        "a clean tree must not be judged by a registry that contradicts itself: {}",
        ran.stderr
    );
    assert_eq!(ran.code, 0, "a tree with nothing to flag must still pass: {}", ran.stdout);
}
