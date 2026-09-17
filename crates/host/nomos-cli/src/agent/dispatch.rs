//! Dispatching `execute`'s own bare goal through `nomos-agent-orchestration`'s
//! [`Run_Agent_Execute`], and rendering whichever of the two outcome shapes it produces --
//! the rendering both `execute` and `judge-role` end at.
//!
//! `P43-AGENT-CANONICAL-SEAM-2` moved this module's own former composition -- assembling a
//! bare `TaskEnvelope`, matching on which backend to reach, and rendering whichever of the
//! two outcome shapes it produces -- into `nomos_agent_orchestration::Run_Agent_Execute`
//! (and, for `judge-role`, `Run_Agent_Judgment`), so `nomos-api` could reach the same
//! dispatch without depending on this crate. What is left here is choosing a
//! [`nomos_composer_std::LAUNCHER`] for the seam's own one platform port and
//! rendering an [`nomos_agent_orchestration::AgentDispatchOutcome`] into the exact text and
//! [`ExitCode`] this command has always reported.
//!
//! # A deliberate change from what this file's own comment used to defend
//!
//! The former `Dispatch_Task` here was fixed to one concrete launcher rather than generic,
//! matching `check.rs`'s and `work.rs`'s own composition-root choice for their own
//! subprocesses -- correct advice for a CLI-only module with exactly one caller. It stopped
//! being correct the moment this dispatch moved into a crate two hosts call:
//! `nomos_agent_orchestration::Run_Agent_Execute`/`Run_Agent_Judgment` are generic over
//! `nomos_platform::ProgramLauncher`, the identical reason
//! `nomos_correction_orchestration::Run_Correction` already is, so `nomos-api` supplies its
//! own launcher, through `nomos-composer-std`, at its own call site rather than depending
//! on this crate's
//! choice, or on this crate at all. Genericizing also retires the `// check-test-coverage:
//! allow-untested` exclusion this file's own two match arms used to carry: they were
//! untestable here only because they were fixed to a real launcher, and
//! `nomos-agent-orchestration`'s own tests now reach the identical match with a scripted
//! one instead -- see that crate's own `run` module doc for the fuller account.

use super::{DispatchConfig, ExitCode};
use nomos_agent_orchestration::{AgentDispatchOutcome, AgentEnvironment, Run_Agent_Execute};

pub(super) fn Execute_Goal(goal: &str, config: DispatchConfig, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    use nomos_composer_std::LAUNCHER;

    let outcome = Run_Agent_Execute(goal, config, &AgentEnvironment { launcher: &LAUNCHER });

    return Rendered_Dispatch_Outcome(&outcome, output, notes);
}

/// Renders `outcome` into the exact text and [`ExitCode`] this command has always
/// reported -- shared with [`super::judge_role::Judge_Role`], the one place both commands'
/// own dispatch ends at.
///
/// `denied_tool_uses` is printed unconditionally, empty or not, so its absence is a
/// caller's own observation rather than a line that only appears when there is bad news to
/// report -- `OD-EXECUTOR-001`'s rule, restated at the one place this workspace renders an
/// executor's answer for a person to read.
pub(super) fn Rendered_Dispatch_Outcome(outcome: &AgentDispatchOutcome, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match outcome
    {
        AgentDispatchOutcome::ClaudeCode(outcome) =>
        {
            let _ = writeln!(
                output,
                "assumptions: {:?}\nunresolved questions: {:?}\ndenied tool uses: {:?}\nis_error: {}  cost_usd: {}.{:06}  duration_ms: {}",
                outcome.result.assumptions,
                outcome.result.unresolved_questions,
                outcome.denied_tool_uses,
                outcome.is_error,
                outcome.cost.Whole_Dollars(),
                outcome.cost.Fractional_Micros(),
                outcome.duration_ms
            );
            ExitCode::Ok
        }
        AgentDispatchOutcome::Ollama(outcome) =>
        {
            let _ = writeln!(output, "{}", outcome.response);
            ExitCode::Ok
        }
        AgentDispatchOutcome::Unavailable(reason) =>
        {
            let _ = writeln!(notes, "{reason}");
            ExitCode::Unavailable
        }
    };
}
