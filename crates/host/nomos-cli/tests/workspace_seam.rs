//! The `nomos_workspace` seam `check-integration-coverage` found with no suite.
//!
//! `nomos-cli::check.rs` and `nomos-cli::gate.rs` both `use nomos_workspace::BuildVariant;`
//! and thread one through `check::composition::Host_Variant()` /
//! `gate::composition::Host_Variant()` into `nomos_check_orchestration::RunContext` and
//! `nomos_gate_orchestration::GateEnvironment` — the identity every fact either
//! orchestration crate materializes is filed under. Both `Host_Variant` functions are
//! private to their own module, and every existing test drives the compiled binary as a
//! subprocess, so nothing under `tests/` ever named `nomos_workspace` directly.
//!
//! `build.rs` captures the four components (`NOMOS_TARGET`, `NOMOS_PROFILE`,
//! `NOMOS_TOOLCHAIN`, `NOMOS_FEATURES`) as `cargo::rustc-env` output, which Cargo applies to
//! every target of this *package* — including a `tests/*.rs` integration binary, not only
//! the `nomos` bin target. That means this file can read the identical four `env!` values
//! `check::composition::Host_Variant` and `gate::composition::Host_Variant` each read, and
//! construct the exact `BuildVariant` this binary's own composition roots build — not an
//! approximation of it.

use nomos_workspace::BuildVariant;

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// The same `BuildVariant` `check::composition::Host_Variant` and
/// `gate::composition::Host_Variant` each construct, built from the identical `env!`
/// sources `build.rs` bakes into every target of this package.
fn Host_Variant_As_The_Composition_Roots_Build_It() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
}

/// Every non-feature component this binary's own composition roots depend on must survive
/// into a real, directly constructed `BuildVariant` -- the same guarantee
/// `check/composition.rs`'s own inline test makes about the private function this
/// reconstructs, proven instead against the public type from outside the crate.
#[test]
fn Test_The_Real_Host_Variant_Should_Carry_Every_Non_Feature_Component()
{
    let variant = Host_Variant_As_The_Composition_Roots_Build_It();

    assert!(!variant.target.is_empty());
    assert!(!variant.profile.is_empty());
    assert!(!variant.toolchain.is_empty());

    // Constructing it twice from the same source must agree, the same identity guarantee
    // every fact filed under a `BuildVariantId` depends on.
    assert_eq!(variant, Host_Variant_As_The_Composition_Roots_Build_It());
    assert_eq!(variant.Id(), Host_Variant_As_The_Composition_Roots_Build_It().Id());
}

/// The end-to-end control: a real `nomos check` run over a scratch tree must complete
/// cleanly, which only happens if the `BuildVariant` this binary's composition root builds
/// (the same one constructed directly above) is accepted by `nomos_check_orchestration::Run`
/// and threaded all the way through to a judged outcome.
#[test]
fn Test_A_Real_Run_Should_Complete_With_The_Real_Host_Variant_Feeding_It()
{
    let tree = Tree::New("workspace-happy-path").With("subject.rs", "pub const TABLES: &[&str] = &[];\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 0, "{}", ran.stdout);
    assert!(
        ran.stdout.contains("1 file(s) examined"),
        "the composition (build variant included) must have reached a judged outcome: {}",
        ran.stdout
    );
}
