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
//! `nomos_gate_orchestration::Fresh_Run_Id(CLOCK.Now())`, where `CLOCK` is
//! `nomos_composer_std::CLOCK` implementing `nomos_platform::Clock` and returning a
//! `nomos_platform::Timestamp`. That `RunId`'s `Display` is exactly what `nomos gate run`
//! prints on its `run: ` line (`gate/tests.rs`'s own
//! `Test_Read_Source_Should_Underlie_A_Real_Runs_RunId_Report` asserts the same shape in
//! process). This file proves the same tie from outside the binary: a `nomos_platform::Timestamp`
//! really drives a `Fresh_Run_Id`'s identity, and the real binary's own printed `RunId` shares
//! that same `Display` shape.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// How wide a `RunId`'s rendering is: `nomos_gate_orchestration::Fresh_Run_Id`'s own digest,
/// in lowercase hex digits. A `run: ` line of another width is not that type's own rendering,
/// which is the tie this file exists to hold.
const RUN_ID_HEX_DIGITS: usize = 32;

/// Two `RunId`s built directly from two different `nomos_platform::Timestamp` values must
/// differ, proving the platform type genuinely drives the identity `gate.rs` prints rather
/// than being an unused dependency -- and the real `nomos gate run`'s own `run: ` line must
/// share the same rendered shape (32 lowercase hex characters) a `RunId` built this way
/// always has.
#[test]
fn Test_Gate_Runs_RunId_Line_Should_Share_Its_Shape_With_A_RunId_Built_From_Nomos_Platforms_Timestamp()
{
    Assert_A_Later_Timestamp_Changes_The_RunId();

    let built_directly = A_Directly_Built_RunId();
    Assert_A_Lowercase_Hex_RunId(&built_directly);

    Assert_The_Real_Runs_Line_Shares_The_Directly_Built_Shape(&built_directly);
}

/// A later `nomos_platform::Timestamp` must change the `RunId` `Fresh_Run_Id` builds -- the
/// same seam `crate::gate::Run_Verb` drives with `CLOCK.Now()`.
fn Assert_A_Later_Timestamp_Changes_The_RunId()
{
    let earlier = nomos_gate_orchestration::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(0));
    let later = nomos_gate_orchestration::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(1));
    assert_ne!(
        earlier, later,
        "a later nomos_platform::Timestamp must change the RunId Fresh_Run_Id builds -- the \
         same seam crate::gate::Run_Verb drives with CLOCK.Now()"
    );
}

/// One `RunId` built straight from a `nomos_platform::Timestamp`, for the shape assertions
/// below to measure the real binary's printed line against.
fn A_Directly_Built_RunId() -> String
{
    return nomos_gate_orchestration::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(0))
        .to_string();
}

/// A `RunId`'s rendering is fixed-width lowercase hex, and nothing else.
fn Assert_A_Lowercase_Hex_RunId(run_id: &str)
{
    assert_eq!(run_id.len(), RUN_ID_HEX_DIGITS, "{run_id}");
    assert!(
        run_id.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "{run_id}"
    );
}

/// The real binary's own `run: ` line and the `RunId` built directly above must render at the
/// same width: an identity printed in another shape is not the `RunId` this crate builds from
/// `nomos_platform::Timestamp` at all.
fn Assert_The_Real_Runs_Line_Shares_The_Directly_Built_Shape(built_directly: &str)
{
    let printed = The_Run_Line_The_Real_Gate_Run_Printed();

    assert_eq!(
        printed.len(),
        built_directly.len(),
        "the real binary's own RunId and one built directly from nomos_platform::Timestamp \
         must render in the same shape: {printed}"
    );
    Assert_A_Lowercase_Hex_RunId(&printed);
}

/// The `run: ` line the real `nomos gate run` printed, or a failure naming what it printed
/// instead.
fn The_Run_Line_The_Real_Gate_Run_Printed() -> String
{
    let tree = Tree::New("platform-seam").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");
    let outcome = Run(&["gate", "run", "--root", &tree.Root()]);

    assert_eq!(outcome.code, 0, "{}", outcome.stderr);

    let run_line = outcome
        .stdout
        .lines()
        .find(|line| line.starts_with("run: "))
        .unwrap_or_else(|| panic!("no `run: ` line in: {}", outcome.stdout));

    return run_line.trim_start_matches("run: ").to_owned();
}
