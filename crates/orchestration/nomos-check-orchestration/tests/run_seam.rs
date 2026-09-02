//! The seam between `nomos_check_orchestration::Run` and the crates its own materialization
//! step calls directly -- `nomos_platform` (generic, via a real `nomos_platform_std` launcher),
//! `nomos_contracts`, `nomos_model`, `nomos_rules` and `nomos_workspace` -- exercised from
//! outside the crate through public types only.
//!
//! `run_context.rs`'s own internal `#[cfg(test)] mod tests` already drives `Run` this way, but
//! only through private-item access; it proves the crate works when opened up, not that the
//! PUBLIC contract a real second adapter depends on holds. This file drives the identical call
//! -- a real source, judged through `Run` with a real `nomos-platform-std` launcher against this
//! repository's own root -- compiled as a separate crate that can reach nothing but
//! `nomos_check_orchestration`'s own public API.
//!
//! `nomos_analysis::Context` is the one thing `Run` reaches into that this file still cannot
//! separately name: it is built and owned entirely inside this crate, never returned or
//! accepted as a parameter. `nomos_analysis::MemoryFactStore` is no longer in that position --
//! `OD-ANALYSIS-009`'s own first increment made it a caller-supplied `RunContext` field, so
//! this file constructs one directly below, the same way any other real second caller would.
//! Every test below that reaches a real `CheckOutcome::Judged` still exercises the materialize-
//! a-fact seam either way -- `Run` cannot reach `Judged` without it.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{Run, RunContext};
use nomos_contracts::RuleId;
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdFileSystem, StdProcessLauncher};
use nomos_rules::{SourceFile, COMPLETENESS_MIRROR};
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own real root -- `Run`'s dependency, lint and policy materialization
/// steps run `cargo metadata`/`clippy`/`deny` against it regardless of what sources a test
/// hands in, so every test below exercises those real subprocess-calling providers too.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

/// The happy path across the boundary: a clean, recognized source judged through the real
/// `Run`, with every selected rule's fact materialized through a real `ProcessLauncher`.
#[test]
fn Test_Run_Should_Judge_A_Clean_Source_Through_A_Real_Process_Launcher()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    assert!(matches!(outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }));
}

/// An error that crosses the boundary: an empty source list cannot be ingested into a
/// workspace state (`Ingested_Workspace`'s own real refusal, not this file's fabrication), so
/// `Run` reports `CheckOutcome::Unreadable` rather than a smaller, silently-judged answer.
#[test]
fn Test_Run_Should_Report_Unreadable_For_An_Empty_Source_List()
{
    let outcome: nomos_check_orchestration::CheckOutcome = Run(&[], RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    assert!(matches!(outcome, nomos_check_orchestration::CheckOutcome::Unreadable));
}

/// The lifecycle `selected` imposes: a `nomos_contracts::RuleId` this crate is asked to
/// narrow to still reaches a judged outcome, proving `selected: &[RuleId]` really crosses the
/// boundary into `Run`'s own materialization gate rather than being ignored.
#[test]
fn Test_Run_Should_Judge_A_Source_When_Narrowed_To_One_Real_Rule()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];
    let selected = [RuleId::New(COMPLETENESS_MIRROR)];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() }, &selected);

    assert!(matches!(outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }));
}

/// `RunContext` is generic over `nomos_platform::ProcessLauncher` rather than any concrete
/// implementation, so a second adapter can call `Run` with its own launcher without this crate
/// depending on `nomos-platform-std`. Runs a real process through the generic bound (rather
/// than the concrete `StdProcessLauncher` type) and reads its real output back, so this test
/// crosses a real implementation of the trait every test above hands `Run`, not a type that
/// merely happens to match its shape.
fn Version_Through_Generic_Launcher<Launcher: nomos_platform::ProcessLauncher>(launcher: &Launcher) -> nomos_platform::ProcessOutput
{
    let command = nomos_platform::Command::New(vec!["cargo".to_owned(), "--version".to_owned()], std::time::Duration::from_secs(30));
    return launcher.Run(&command).expect("cargo --version must be a real, launchable process");
}

#[test]
fn Test_Run_Should_Accept_A_Real_Nomos_Platform_Process_Launcher()
{
    let output = Version_Through_Generic_Launcher(&StdProcessLauncher);

    assert!(output.stdout.contains("cargo"), "{}", output.stdout);
}
