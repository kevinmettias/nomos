//! What the `policy` verb renders: which layer and artifact decided each field of the policy
//! a run was judged under.
//!
//! Every line here comes from [`nomos_gate_orchestration::Effective_Policy_Report`] and none
//! is composed in this crate. That is the whole point of the function: `nomos-api` reports the
//! same answer to a headless caller, and two assemblies of one fact would let the two surfaces
//! disagree about what decided a field while both looked right on their own.

use super::super::ExitCode;
use nomos_gate_orchestration::{Effective_Policy_Report, GateRunResult, NoVerdict};
use std::io::Write;

/// What a reader is told when the resolution this verb reports on refused.
const UNRESOLVED: &str = "this run's policy did not resolve, so there is no effective policy to \
report: it judged under the build's own defaults rather than under half of what was stated.";

/// The same, for a refusal that recorded no reason -- which no real run produces.
const UNRESOLVED_WITHOUT_A_REASON: &str = "this run carries no effective policy and did not \
record why its resolution refused.";

/// Renders what decided every field of `result`'s own effective policy.
///
/// The exit code is a property of whether the question could be answered, not of what the
/// answer was: a policy that resolved is reported and the verb is clean, whatever the run
/// itself judged. `OD-POLICY-001`'s rendering changes no disposition and no verdict, so a
/// gate that failed and a gate that passed report their policy identically.
pub(in crate::gate) fn Render_Policy(result: &GateRunResult, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let Some(policy) = result.policy.as_deref()
    else
    {
        return Render_Unresolved(result.no_verdict.as_ref(), stderr);
    };

    for line in Effective_Policy_Report(policy)
    {
        let _ = writeln!(stdout, "{line}");
    }

    return ExitCode::Ok;
}

/// A run whose resolution refused, reported with the refusal's own sentence when it has one.
///
/// `ExitCode::Contradictory` is the code `run` already gives a policy it cannot accept, and
/// the reason is the same: what the caller asked about was never assembled. A clean exit over
/// an empty report would say the repository states nothing, which is a different answer.
fn Render_Unresolved(cause: Option<&NoVerdict>, stderr: &mut impl Write) -> ExitCode
{
    let _ = match cause
    {
        Some(NoVerdict::UnreadablePolicy(detail) | NoVerdict::MalformedPolicy(detail)) =>
        {
            writeln!(stderr, "{UNRESOLVED}\n  {detail}")
        }
        _ => writeln!(stderr, "{UNRESOLVED_WITHOUT_A_REASON}"),
    };

    return ExitCode::Contradictory;
}
