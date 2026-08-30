//! Band 37 — the first real `ModelBackend` adapter, not a second `AgentExecutor`
//! (`OD-PACKAGE-013`): `nomos-agent-contracts::TaskEnvelope` in, a bounded local Ollama
//! model subprocess dispatched through it, [`AgentExecutionOutcome`] out. This crate was
//! built and named as "the second real `AgentExecutor`"; `OD-PACKAGE-013` measured its real
//! mechanism — a fixed model, one forwarded field, every other `TaskEnvelope` field read and
//! ignored, no tool-use loop, no MCP surface — against `PackageKind::ModelBackendPackage`'s
//! and `PackageKind::AgentExecutorPackage`'s own doc comments and found it matches the
//! former, not the latter. It is renamed here to say so; nothing about its behavior changed.
//! Independent of `nomos-agent-executor-claude-code`, the one real `AgentExecutor` — always
//! structurally parallel, never sharing a trait, and now also never the same package kind
//! (`OD-EXECUTOR-005`), per `OD-EXECUTOR-001`'s own restraint and `OD-EXECUTOR-004`'s.
//!
//! `OD-EXECUTOR-004` decided what this dispatch is permitted to do before this crate existed
//! to do it, measured against `ollama run`'s own real mechanism rather than inherited from
//! `OD-EXECUTOR-001`'s Claude-Code-specific rule by analogy: never pass `--experimental`,
//! `--experimental-yolo` or `--experimental-websearch` — the only flags that open any
//! tool-use capability, so their absence is the entire structural boundary, not an
//! allow-list naming nothing, because there is no tool subsystem to grant into; launch from
//! a freshly created, isolated working directory regardless of today's absence of any known
//! mechanism that reads it; bound the call with a wall-clock timeout, since local inference
//! has no per-call dollar cost to bound instead; and read a clean, non-empty stdout and a
//! zero exit as the entire evidence of what happened, never the response text as a report of
//! an action taken.
//!
//! It does not assemble a `nomos_agent_contracts::WorkResult`, for the same reason
//! `nomos-agent-executor-claude-code` does not: a free-text response has no honest, general
//! mapping into `claims`/`assumptions`/`unresolved_questions`.
//!
//! `TaskEnvelope.goal` is read. `scope`, `prohibited_changes`, `available_tools`,
//! `knowledge_context`, `applicable_rules` and `effort` are accepted and ignored — the same
//! "does not pretend otherwise" restraint `nomos-agent-executor-claude-code` already states
//! for the fields nothing in this workspace enforces yet. `effort` specifically: `ollama run`
//! has no `--effort`-shaped native control this crate found to map `EffortLevel` onto
//! honestly (its own `--think` flag is a distinct, model-support-dependent reasoning-depth
//! toggle for a narrower model set, not verified here, and inventing a mapping ahead of that
//! verification would repeat the exact "an approximation invented rather than measured"
//! mistake `OD-PACKAGE-011` already warns against) — a real gap, not a silent one.

#![forbid(unsafe_code)]

mod agent_execution_error;
mod agent_execution_outcome;

pub use agent_execution_error::AgentExecutionError;
pub use agent_execution_outcome::AgentExecutionOutcome;

use nomos_agent_contracts::TaskEnvelope;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::PathBuf;
use std::time::Duration;

/// `OD-EXECUTOR-004`'s own measured adversarial run took under two minutes on this crate's
/// own default model on real development hardware; five minutes is headroom for a longer
/// real task, a colder model load, or slower hardware, without leaving a hung subprocess to
/// wait out an unbounded timeout. There is no dollar-cost signal to bound instead, unlike
/// `nomos-agent-executor-claude-code`'s `--max-budget-usd` — local inference has no metered
/// charge a runaway invocation could inflate.
const TIMEOUT: Duration = Duration::from_secs(300);

/// A real, locally pulled, coding-oriented model, verified directly against this machine —
/// `ollama list` names it among several already present. Not a derived or configurable
/// choice: `nomos-model-package`'s own `ModelSelector` has no real second-case consumer yet
/// (`OD-PACKAGE-011`/`OD-PACKAGE-012`), and inventing a selection mechanism ahead of a real
/// caller that needs one would be exactly the premature vocabulary those records already
/// decline to build against. A future increment that needs a different or caller-chosen
/// model has a real, named next step to reach for; this one does not invent it early.
const MODEL: &str = "qwen2.5-coder:7b";

/// Dispatches `task.goal` to a local Ollama model as a subprocess, bounded by
/// `OD-EXECUTOR-004`'s structural capability boundary, and reads back what it produced.
///
/// # Errors
///
/// [`AgentExecutionError::Unavailable`] if the isolated working directory could not be
/// created, the process could not be started, exited non-zero (including when the `ollama
/// serve` daemon this backend requires is unreachable), or was killed for timing out or
/// stalling.
pub fn Execute_Task<Launcher: ProcessLauncher>(task: &TaskEnvelope, launcher: &Launcher) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let working_directory = Isolated_Working_Directory()?;

    return Execute_In(task, launcher, &working_directory);
}

/// A freshly created, empty directory under the system temp root, never this repository's
/// own tree — `OD-EXECUTOR-004`'s rule applies this defensively, even though it found no
/// mechanism by which `ollama run` reads its own working directory, on the same "a property
/// of the invocation, not a property inferred from today's absence of a mechanism that could
/// read it" reasoning that record states.
///
/// Delegates to `nomos_agent_contracts::Isolated_Working_Directory`, shared with
/// `nomos-agent-executor-claude-code`'s own isolation step; this crate's only distinct part
/// is the prefix its directories are named from, so two crates' isolated directories are
/// never mistaken for one another on the same machine.
fn Isolated_Working_Directory() -> Result<PathBuf, AgentExecutionError>
{
    return nomos_agent_contracts::Isolated_Working_Directory("nomos-model-backend-ollama").map_err(|error| {
        return AgentExecutionError::Unavailable(error.to_string());
    });
}

/// [`Execute_Task`], over a caller-chosen `working_directory` rather than a freshly generated
/// one — the same seam `nomos_agent_executor_claude_code::Execute_In` offers its own real
/// adversarial integration test.
pub(crate) fn Execute_In<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    working_directory: &std::path::Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let command = Command_For(task, working_directory);
    let output = launcher.Run(&command).map_err(AgentExecutionError::Unavailable)?;

    Require_Clean_Exit(&output.outcome, &output.stderr)?;

    return Ok(AgentExecutionOutcome { response: output.stdout.trim().to_owned() });
}

/// `ollama` resolves to a native `.exe` on every platform this workspace's own
/// `StdProcessLauncher` runs on — verified directly on Windows (`where ollama` names
/// `ollama.exe`, not a `.cmd`/`.bat` shim). Unlike `nomos-agent-executor-claude-code`'s
/// `claude.cmd`, `StdProcessLauncher`'s "no shell, ever" `CreateProcess` dispatch needs no
/// special-casing here: there is no batch-file argument re-parsing layer for a goal
/// containing a newline or a quote to trip over, so this crate does not carry that crate's
/// own `Single_Line` normalization — verified by the scripted tests below rather than
/// assumed absent.
const OLLAMA_PROGRAM: &str = "ollama";

/// The invocation `OD-EXECUTOR-004`'s rule describes, over `task.goal`, run from
/// `working_directory`. No `--experimental`, `--experimental-yolo` or
/// `--experimental-websearch` ever appears — the entire structural boundary is their
/// absence, verified adversarially (see this crate's real integration test).
fn Command_For(task: &TaskEnvelope, working_directory: &std::path::Path) -> Command
{
    let argv = vec![OLLAMA_PROGRAM.to_owned(), "run".to_owned(), MODEL.to_owned(), task.goal.clone()];

    let mut command = Command::New(argv, TIMEOUT);
    command.working_directory = Some(working_directory.to_path_buf());

    return command;
}

/// Refuses every outcome a launched process can report other than a clean, zero exit — the
/// same shape `nomos_agent_executor_claude_code::Require_Clean_Exit` and
/// `nomos_lang_rust_cargo`'s own use for the other subprocesses this workspace runs through
/// `ProcessLauncher`.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), AgentExecutionError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } => {
            Err(AgentExecutionError::Unavailable(format!("ollama exited {code}: {stderr}")))
        }
        ExitOutcome::TimedOut => {
            Err(AgentExecutionError::Unavailable(format!("ollama was still running after {TIMEOUT:?} and was killed")))
        }
        ExitOutcome::Stalled { idle_elapsed } => Err(AgentExecutionError::Unavailable(format!(
            "ollama produced no output for {idle_elapsed:?} and was judged stalled"
        ))),
        ExitOutcome::Terminated => {
            Err(AgentExecutionError::Unavailable("ollama was terminated before it could finish".to_owned()))
        }
    };
}

#[cfg(test)]
mod tests;
