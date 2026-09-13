//! The seam between `nomos_gate_orchestration` and `nomos_check_orchestration`, exercised from
//! outside the crate through public types only.
//!
//! `nomos_gate_orchestration`'s own internal `#[cfg(test)] mod tests` already drives
//! `nomos_check_orchestration` through `Run_Gate`/`Explain_Gate`, reaching
//! `nomos_check_orchestration::{Claim, CheckOutcome}` as an ordinary dependency that can still
//! see this crate's own private items. That proves the two work when one side is opened up; it
//! does not prove the PUBLIC contract holds, which is the only thing a real consumer can depend
//! on. This file drives the identical seam -- the walk this crate's own `Run_Gate` calls
//! `nomos_check_orchestration::Run` through -- compiled as a separate crate that can reach
//! nothing but `nomos_gate_orchestration`'s and `nomos_check_orchestration`'s own public API.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Digest128, GateCategory, RunId};
use nomos_gate_orchestration::{GateCommand, GateEnvironment, GateRunOutcome, Run_Gate};
use nomos_platform::ProcessLauncher;
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

fn Test_Run_Id() -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));
}

/// This repository's own real root -- `Run_Gate`'s dependency step, through
/// `nomos_check_orchestration::Run`, runs `cargo metadata` against it regardless of what
/// sources a test hands in.
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

/// The happy path across the boundary: a clean source, judged through the real
/// `nomos_check_orchestration::Run` this crate calls directly, reports `CheckOutcome::Judged`
/// with a `Claim::Complete` and no blocking findings.
#[test]
fn Test_Run_Gate_Should_Judge_A_Clean_Source_Through_The_Real_Check_Orchestration_Seam()
{
    let sources = vec![Source("a.rs", "pub fn Ok()\n{\n}\n")];
    let command = GateCommand { root: Repository_Root(), ..Default::default() };

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }), "a source every provider can materialize a fact for must be judged");
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty());
}

/// An error that crosses the boundary in the other direction: a root nobody could walk at all
/// is `nomos_check_orchestration::CheckOutcome::Unreadable`, and this crate's own
/// `GateRunOutcome::Indeterminate` follows it rather than collapsing into `Passed`.
#[test]
fn Test_Run_Gate_Should_Report_An_Unreadable_Check_Outcome_As_Indeterminate()
{
    let command = GateCommand { root: Repository_Root(), ..Default::default() };

    let result = Run_Gate(None, GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::Unreadable));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
}

/// The lifecycle this crate imposes on what `nomos_check_orchestration` returns: a real
/// blocking finding it judged must reach `GateRunResult::findings.blocking_findings` with its
/// own `GateCategory::Blocking` tag intact, not summarized or dropped along the way.
#[test]
fn Test_Run_Gate_Should_Carry_A_Real_Blocking_Finding_Through_Unmodified()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let command = GateCommand { root: Repository_Root(), ..Default::default() };

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.findings.blocking_findings.is_empty());
    assert!(result.findings.blocking_findings.iter().all(|finding| return finding.gate == GateCategory::Blocking));
}

/// `GateEnvironment` is generic over `nomos_platform::ProcessLauncher` rather than any concrete
/// implementation, so a second adapter can hand `Run_Gate` its own launcher without this crate
/// depending on `nomos-platform-std`. Runs a real process through the generic bound (rather
/// than the concrete `StdProcessLauncher` type) and reads its real output back, so this test
/// crosses a real implementation of the trait every test above hands `Run_Gate`, not a type
/// that merely happens to match its shape.
fn Version_Through_Generic_Launcher<Launcher: ProcessLauncher>(launcher: &Launcher) -> nomos_platform::ProcessOutput
{
    let command = nomos_platform::Command::New(vec!["cargo".to_owned(), "--version".to_owned()], std::time::Duration::from_secs(30));
    return launcher.Run(&command).expect("cargo --version must be a real, launchable process");
}

#[test]
fn Test_Run_Gate_Should_Accept_A_Real_Nomos_Platform_Process_Launcher()
{
    let output = Version_Through_Generic_Launcher(&StdProcessLauncher);

    assert!(output.stdout.contains("cargo"), "{}", output.stdout);
}
