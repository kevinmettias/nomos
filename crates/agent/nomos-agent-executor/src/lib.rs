//! Band 37 — the first real `AgentExecutor`: `nomos-agent-contracts::TaskEnvelope` in, a
//! bounded Claude Code subprocess dispatched through it, [`AgentExecutionOutcome`] out.
//!
//! `OD-EXECUTOR-001` decided what this dispatch is permitted to do before this crate existed
//! to do it: a freshly created, empty, isolated working directory; no MCP configuration; tool
//! access granted through an allow-list naming no real tool rather than denied through a list
//! that will always be one release behind the real tool set; one `--print` turn; and a result
//! read for what it structurally permitted, never for what its own free text claims happened.
//! [`Execute`] is that record's rule, and nothing more — it does not assemble a
//! `nomos_agent_contracts::WorkResult`. `WorkResult.plan` is a `CorrectionPlan`, and
//! `CorrectionPlan::New` refuses an empty candidate list; a judgment-only task genuinely has
//! no plan to report, and stretching an empty `ChangeSet` into a fabricated "candidate" to fit
//! the type would be inventing a correction nobody proposed. That gap is real and this crate
//! does not paper over it — assembling a `WorkResult` is a later increment's question, once a
//! real caller needs one.
//!
//! Only `TaskEnvelope.goal` is read. `scope`, `prohibited_changes`, `available_tools`,
//! `knowledge_context` and `applicable_rules` are accepted and ignored, matching
//! `OD-EXECUTOR-001`'s own finding that nothing in this workspace enforces them yet — this
//! crate does not pretend otherwise by silently honoring some of them and not others.

#![forbid(unsafe_code)]

mod error;
mod outcome;
mod response;

pub use error::AgentExecutionError;
pub use outcome::AgentExecutionOutcome;

use nomos_agent_contracts::TaskEnvelope;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A prompt this repository has already run against the real CLI takes well under a
/// minute; five minutes is headroom for a longer real task without leaving a hung
/// subprocess to wait out an unbounded timeout.
const TIMEOUT: Duration = Duration::from_secs(300);

/// Names no real tool, by construction. `--allowedTools` set to only this value grants
/// nothing: `OD-EXECUTOR-001`'s amendment measured a deny-list leaking twice — once for a
/// platform-specific tool name the list's author did not know to include, once for a
/// tool this crate never anticipated at all — before an allow-list naming this same kind
/// of placeholder held under an adversarial prompt. The leading underscores and the name
/// itself are chosen so a reader, or a future real tool, can never mistake this for
/// something meant to match.
const NO_TOOLS_GRANTED: &str = "__nomos_agent_executor_denies_all_tools__";

/// Dispatches `task.goal` to Claude Code as a subprocess, bounded by `OD-EXECUTOR-001`'s
/// structural capability boundary, and reads back what it reported.
///
/// # Errors
///
/// [`AgentExecutionError::Unavailable`] if the isolated working directory could not be
/// created, the process could not be started, exited non-zero, or was killed for timing
/// out or stalling. [`AgentExecutionError::Unparseable`] if its stdout was not the JSON
/// document `--output-format json` promises.
pub fn Execute<P: ProcessLauncher>(task: &TaskEnvelope, launcher: &P) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let working_directory = Isolated_Working_Directory()?;

    return Execute_In(task, launcher, &working_directory);
}

/// [`Execute`], over a caller-chosen `working_directory` rather than a freshly generated
/// one — the seam this crate's own real, adversarial integration test uses to inspect
/// that directory afterward, since `Execute`'s own isolated directory is otherwise
/// generated and discarded where no caller could ever name it.
pub(crate) fn Execute_In<P: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &P,
    working_directory: &std::path::Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let command = Command_For(task, working_directory);
    let output = launcher.Run(&command).map_err(AgentExecutionError::Unavailable)?;

    Require_Clean_Exit(&output.outcome, &output.stderr)?;

    return response::Parse(&output.stdout);
}

/// A freshly created, empty directory under the system temp root, never this repository's
/// own tree and never one carrying its own `.claude/settings*` or `CLAUDE.md` — the first
/// clause of `OD-EXECUTOR-001`'s rule. Named from this process's id and a per-process
/// counter rather than the wall clock, so two calls in the same process never collide and
/// nothing here depends on time having advanced.
fn Isolated_Working_Directory() -> Result<PathBuf, AgentExecutionError>
{
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!("nomos-agent-executor-{}-{sequence}", std::process::id()));

    std::fs::create_dir_all(&directory).map_err(|error| {
        return AgentExecutionError::Unavailable(format!(
            "could not create an isolated working directory at {}: {error}",
            directory.display()
        ));
    })?;

    return Ok(directory);
}

/// `claude` on every platform this workspace's own `StdProcessLauncher` runs on but
/// Windows, where the real entry point on `PATH` is an npm-generated `claude.cmd` shim
/// (confirmed directly against this machine: `where claude` names both an extensionless
/// POSIX shell script and `claude.cmd`, in that order). `StdProcessLauncher` spawns
/// `argv[0]` through `CreateProcess` directly, by its own deliberate "no shell, ever"
/// design — the same reason it does not attempt `PATHEXT` resolution a shell would do
/// silently, so the correct name is this crate's own responsibility to supply, once, here,
/// rather than a capability every caller of the shared launcher would otherwise need.
#[cfg(windows)]
const CLAUDE_PROGRAM: &str = "claude.cmd";

/// See [`CLAUDE_PROGRAM`]'s Windows doc.
#[cfg(not(windows))]
const CLAUDE_PROGRAM: &str = "claude";

/// The invocation `OD-EXECUTOR-001`'s rule describes, over `task.goal`, run from
/// `working_directory`.
fn Command_For(task: &TaskEnvelope, working_directory: &std::path::Path) -> Command
{
    let mut command = Command::New(
        vec![
            CLAUDE_PROGRAM.to_owned(),
            "--print".to_owned(),
            task.goal.clone(),
            "--output-format".to_owned(),
            "json".to_owned(),
            "--strict-mcp-config".to_owned(),
            "--allowedTools".to_owned(),
            NO_TOOLS_GRANTED.to_owned(),
        ],
        TIMEOUT,
    );
    command.working_directory = Some(working_directory.to_path_buf());

    return command;
}

/// Refuses every outcome a launched process can report other than a clean, zero exit —
/// the same shape `nomos_lang_rust_cargo`'s own `Require_Clean_Exit` uses for the one
/// other subprocess this workspace ever runs through `ProcessLauncher`.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), AgentExecutionError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } => {
            Err(AgentExecutionError::Unavailable(format!("claude exited {code}: {stderr}")))
        }
        ExitOutcome::TimedOut => {
            Err(AgentExecutionError::Unavailable(format!("claude was still running after {TIMEOUT:?} and was killed")))
        }
        ExitOutcome::Stalled { idle_elapsed } => Err(AgentExecutionError::Unavailable(format!(
            "claude produced no output for {idle_elapsed:?} and was judged stalled"
        ))),
        ExitOutcome::Terminated => {
            Err(AgentExecutionError::Unavailable("claude was terminated before it could finish".to_owned()))
        }
    };
}

#[cfg(test)]
mod tests;
