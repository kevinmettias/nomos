//! `nomos gate` — the composition root and renderer for `nomos-gate-orchestration`.
//!
//! `P13-GATE-ORCHESTRATION-1` gave `Gate` its own crate and a real, already-implemented
//! `Run(&GateCommand) -> GateOutcome` -- but that crate cannot read argv, choose a platform
//! or write to a terminal, and until this module existed nothing called it. This is the
//! same seam `OD-HOST-002` already built for `check`/`work`/`spec`: `Parse` turns argv into
//! a typed command, `Run` below calls the orchestration crate's own `Run`, and
//! [`report::Render`] turns what came back into text and an [`ExitCode`].
//!
//! # What this module does not do
//!
//! It does not read [`GateCommand::root`] for anything beyond carrying it, because
//! `nomos-gate-orchestration`'s `Plan` does not read it yet -- `OD-GATE-014` is the open
//! question about when a selector would give it something to read. It does not implement
//! `run`, `explain` or `compare`: [`parsing::Parse`] refuses any verb but `plan` as usage,
//! the same "no invented shape ahead of a real body" discipline the orchestration crate's
//! own `command.rs` already documents.

mod parsing;
mod report;

#[cfg(test)]
mod tests;

pub use parsing::Parse;
use report::Render;

mod exit_code;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_gate_orchestration::GateCommand;

use crate::arguments::Named_Value;
use std::io::Write;
use std::path::PathBuf;

/// Runs this gate's plan and renders what it says.
pub fn Run(command: &GateCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let outcome = nomos_gate_orchestration::Run(command);

    return Render(&outcome, stdout, stderr);
}
