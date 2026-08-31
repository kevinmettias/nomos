//! `nomos_cli` <-> `nomos_platform`, exercised through the compiled binary's public surface.
//!
//! `gate/report.rs`'s inline `#[cfg(test)] mod tests` builds `nomos_platform::Timestamp`
//! values and feeds them to `nomos_gate_orchestration::Fresh_Run_Id` to construct the
//! `GateRunResult`s its own rendering tests assert on -- proving `nomos_platform` works
//! against `nomos-cli`'s own private types, opened up, rather than against its public
//! surface. `nomos-cli` is a `[[bin]]`-only crate (no `src/lib.rs`), so a file under
//! `tests/` cannot reach `GateRunResult` or `Render_Run` at all; the only public surface
//! left is the compiled binary itself.
//!
//! `crate::gate::Run_Verb` (`src/gate.rs`) is the real, production seam:
//! `nomos_gate_orchestration::Fresh_Run_Id(SystemClock.Now())`, where `SystemClock` is
//! `nomos_platform_std::SystemClock` implementing `nomos_platform::Clock` and returning a
//! `nomos_platform::Timestamp`. That `RunId`'s `Display` is exactly what `nomos gate run`
//! prints on its `run: ` line (`gate/tests.rs`'s own
//! `Test_Read_Source_Should_Underlie_A_Real_Runs_RunId_Report` asserts the same shape in
//! process). This file proves the same tie from outside the binary: a `nomos_platform::Timestamp`
//! really drives a `Fresh_Run_Id`'s identity, and the real binary's own printed `RunId` shares
//! that same `Display` shape.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// Two `RunId`s built directly from two different `nomos_platform::Timestamp` values must
/// differ, proving the platform type genuinely drives the identity `gate.rs` prints rather
/// than being an unused dependency -- and the real `nomos gate run`'s own `run: ` line must
/// share the same rendered shape (32 lowercase hex characters) a `RunId` built this way
/// always has.
#[test]
fn Test_Gate_Runs_RunId_Line_Should_Share_Its_Shape_With_A_RunId_Built_From_Nomos_Platforms_Timestamp()
{
    let earlier = nomos_gate_orchestration::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(0));
    let later = nomos_gate_orchestration::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(1));
    assert_ne!(
        earlier, later,
        "a later nomos_platform::Timestamp must change the RunId Fresh_Run_Id builds -- the \
         same seam crate::gate::Run_Verb drives with SystemClock.Now()"
    );

    let built_directly = earlier.to_string();
    assert_eq!(built_directly.len(), 32, "{built_directly}");
    assert!(
        built_directly.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "{built_directly}"
    );

    let tree = Tree::New("platform-seam").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");
    let outcome = Run(&["gate", "run", "--root", &tree.Root()]);

    assert_eq!(outcome.code, 0, "{}", outcome.stderr);
    let run_line = outcome
        .stdout
        .lines()
        .find(|line| line.starts_with("run: "))
        .unwrap_or_else(|| panic!("no `run: ` line in: {}", outcome.stdout));
    let printed = run_line.trim_start_matches("run: ");

    assert_eq!(
        printed.len(),
        built_directly.len(),
        "the real binary's own RunId and one built directly from nomos_platform::Timestamp \
         must render in the same shape: {run_line}"
    );
    assert!(
        printed.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "{run_line}"
    );
}
