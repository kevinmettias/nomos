//! `nomos gate` — the composition root and renderer for `nomos-gate-orchestration`, and for
//! the real judgment `run` now performs.
//!
//! `P13-GATE-ORCHESTRATION-1` gave `Gate` its own crate and a real, already-implemented
//! `Run(&GateCommand) -> GateOutcome` -- but that crate cannot read argv, choose a platform
//! or write to a terminal, and until this module existed nothing called it. This is the
//! same seam `OD-HOST-002` already built for `check`/`work`/`spec`: `Gate_Invocation_From_String_Arguments` turns argv into
//! a typed [`Invocation`], [`Run`] below dispatches to either the orchestration crate's
//! own `Run` (`plan`) or `Run_Gate` (`run`), and [`report`] turns what came back into text
//! and an [`ExitCode`].
//!
//! # What `run` still does here, after `P13-GATE-RUN-SEAM-CRATE`
//!
//! `nomos_gate_orchestration::Run_Gate` now owns the walk-judge-reduce composition itself --
//! this module used to duplicate it (`crates/host/nomos-cli/src/gate/run.rs`, deleted by
//! `P13-GATE-RUN-SEAM-CLI`) because `nomos-gate-orchestration` and `nomos-check-orchestration`
//! were both band 40 and a band may not depend on its own band
//! (`tests/contract/tests/boundaries/graph.rs`); `nomos-gate-orchestration` moved to band 41
//! to depend on it instead. What stays here is exactly what `check.rs` also keeps for the
//! same reason: the directory walk ([`sources::Walked_Sources`] -- no
//! [`nomos_platform::FileSystem`] directory-listing port exists) and the host build variant
//! ([`composition::Host_Variant`] -- `env!` resolves against the crate that calls it). Both
//! cross into `Run_Gate` as arguments; nothing about judging or reducing lives in this crate
//! any more.
//!
//! # `--include` / `--exclude` / `--rule`, after `P13-GATE-014-SCOPE-RULE-SELECTORS`
//!
//! [`parsing::Gate_Invocation_From_String_Arguments`] reads these repeatable flags into [`GateCommand::scope`] and
//! [`GateCommand::rules`] for every verb; `Run_Gate` is what actually consults them for
//! `run`, and `nomos_gate_orchestration::Run` (`plan`) still does not, the same asymmetry
//! `root` already had. See `nomos_gate_orchestration`'s own `lib.rs` doc for what
//! selection here does and does not mean.
//!
//! # `explain`, after `P13-GATE-EXPLAIN-FIRST-INCREMENT`
//!
//! [`Invocation::Explain`] carries a [`GateCommand`] and a
//! `nomos_gate_orchestration::FindingQuery` -- `--rule <id> --location <path>` --
//! [`parsing::Gate_Invocation_From_String_Arguments`] now recognizes as a third verb. `Explain_Gate` is what actually
//! answers it; this module still owns only the walk and the host build variant, the same
//! division `run` already has.
//!
//! # What this module still does not do
//!
//! It does not implement `compare`: [`parsing::Gate_Invocation_From_String_Arguments`] refuses it as usage, the same "no
//! invented shape ahead of a real body" discipline the orchestration crate's own
//! `command.rs` already documents.

mod composition;
mod invocation;
mod parsing;
mod report;
mod sources;

#[cfg(test)]
mod tests;

pub use invocation::Invocation;
pub use parsing::Gate_Invocation_From_String_Arguments;
use report::{Render_Explain, Render_Plan, Render_Run};

mod exit_code;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_gate_orchestration::{FindingQuery, GateCommand};
use nomos_platform::Clock;
use nomos_platform_std::{StdFileSystem, StdProcessLauncher, SystemClock};

use crate::arguments::Named_Value_From_String_Arguments;
use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Runs the requested verb and renders what it says.
pub fn Run(invocation: &Invocation, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match invocation
    {
        Invocation::Plan(command) =>
        {
            let outcome = nomos_gate_orchestration::Run(command);
            Render_Plan(&outcome, stdout, stderr)
        }
        Invocation::Run(command) => Run_Verb(command, stdout, stderr),
        Invocation::Explain { command, query } =>
        {
            let walked = sources::Walked_Sources(&command.root);
            let result = nomos_gate_orchestration::Explain_Gate(
                walked,
                nomos_gate_orchestration::GateEnvironment {
                    variant: composition::Host_Variant(),
                    launcher: &StdProcessLauncher,
                    filesystem: &StdFileSystem,
                },
                command,
                query,
            );
            Render_Explain(&result, stdout, stderr)
        }
    };
}

/// Walks `command.root`, runs `Run_Gate` over it under a freshly minted `RunId`, and
/// renders what came back -- the self-contained unit `Invocation::Run`'s own arm was.
fn Run_Verb(command: &GateCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let walked = sources::Walked_Sources(&command.root);
    let run = nomos_gate_orchestration::Fresh_Run_Id(SystemClock.Now());
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment {
            variant: composition::Host_Variant(),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
        },
        command,
        run,
    );

    return Render_Run(&result, stdout, stderr);
}

#[cfg(test)]
mod run_coverage
{
    use super::*;

    /// `Run` dispatched through its `Plan` arm, driving the real top-level function end to
    /// end. `Plan` composes and reports this gate's own rule registry -- it never walks
    /// `command.root` -- so this is the fast, side-effect-free arm; `Run_Verb` (the `run`
    /// arm) and `Explain_Gate` (the `explain` arm) are exercised through `gate/tests.rs`'s
    /// own broader coverage.
    #[test]
    fn Test_Run_Should_Dispatch_A_Plan_Invocation_And_Report_Ok()
    {
        let invocation = Invocation::Plan(GateCommand::default());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&invocation, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Ok);
    }
}
