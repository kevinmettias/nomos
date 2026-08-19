//! `nomos gate` — the composition root and renderer for `nomos-gate-orchestration`, and for
//! the real judgment `run` now performs.
//!
//! `P13-GATE-ORCHESTRATION-1` gave `Gate` its own crate and a real, already-implemented
//! `Run(&GateCommand) -> GateOutcome` -- but that crate cannot read argv, choose a platform
//! or write to a terminal, and until this module existed nothing called it. This is the
//! same seam `OD-HOST-002` already built for `check`/`work`/`spec`: `Parse` turns argv into
//! a typed [`GateInvocation`], [`Run`] below dispatches to either the orchestration crate's
//! own `Run` (`plan`) or [`run::Run_Gate`] (`run`), and [`report`] turns what came back into
//! text and an [`ExitCode`].
//!
//! # Why `run` is not `nomos_gate_orchestration::Run`
//!
//! `nomos-gate-orchestration` and `nomos-check-orchestration` are both band 40
//! (`README.md`) and a band may not depend on its own band
//! (`tests/contract/tests/boundaries/graph.rs`). `plan`'s whole computation fits inside
//! `nomos-gate-orchestration` because it never needs a `CheckOutcome`; `run`'s does, so it
//! lives in this crate instead, the same way `check.rs` itself is where a tree gets walked
//! because `nomos-check-orchestration` cannot do that from inside its own band either.
//! `run.rs`'s own doc says this at length; `[GateInvocation::Run]`'s two branches in [`Run`]
//! below are the two places that division is visible from this module.
//!
//! # What this module does not do
//!
//! It does not select by scope or rule -- [`GateCommand::root`] is read by `run` now, but
//! nothing filters what `run` judges by it beyond "is this the tree" -- `OD-GATE-014` is
//! the open question about when a selector would narrow that further. It does not
//! implement `explain` or `compare`: [`parsing::Parse`] refuses either verb as usage, the
//! same "no invented shape ahead of a real body" discipline the orchestration crate's own
//! `command.rs` already documents.

mod composition;
mod parsing;
mod report;
mod run;
mod sources;

#[cfg(test)]
mod tests;

pub use parsing::Parse;
use report::{Render_Plan, Render_Run};

mod exit_code;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_gate_orchestration::GateCommand;
use run::Run_Gate;

use crate::arguments::Named_Value;
use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What `nomos gate` was asked to do -- which verb, over which command.
///
/// A struct-per-verb enum rather than `GateCommand` itself growing a variant:
/// [`GateInvocation::Plan`]'s real computation lives entirely inside
/// `nomos-gate-orchestration`; [`GateInvocation::Run`]'s cannot, for the band reason this
/// module's own doc gives. The two verbs do not share a downstream function to route
/// through, so there is nothing for a single command enum inside
/// `nomos-gate-orchestration` to gain by carrying the verb itself.
#[derive(Debug)]
pub enum GateInvocation
{
    /// Compose this gate's rule registry and report what it holds.
    Plan(GateCommand),
    /// Walk the tree, judge it exactly as `nomos check` would, and report a real
    /// disposition.
    Run(GateCommand),
}

/// Runs the requested verb and renders what it says.
pub fn Run(invocation: &GateInvocation, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match invocation
    {
        GateInvocation::Plan(command) =>
        {
            let outcome = nomos_gate_orchestration::Run(command);
            Render_Plan(&outcome, stdout, stderr)
        }
        GateInvocation::Run(command) =>
        {
            let result = Run_Gate(command);
            Render_Run(&result, stdout, stderr)
        }
    };
}
