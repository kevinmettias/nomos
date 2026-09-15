//! What the `admits` verb renders: whether one crate may name another under this
//! repository's declared architecture.

use super::super::ExitCode;
use nomos_gate_orchestration::Admissibility;
use std::io::Write;

/// What the architecture says about an edge that does not exist yet.
///
/// `Permitted` and `NotJudged` are both `Ok`; only `Refused` is `Violations`. That pairing is
/// deliberate and it is the one place this verb could mislead: a caller scripting `admits`
/// into a pre-commit hook reads the exit code, and giving `NotJudged` a failing code would
/// stop a build over a crate this workspace has no opinion about, while giving `Refused` a
/// clean one would let the edge through. So the code carries the judgment and the text
/// carries the difference between "yes" and "no answer" — which the line always says, in
/// words, whichever code it exits with.
pub(in crate::gate) fn Render_Admits(answer: Admissibility, pair: (&str, &str), stdout: &mut impl Write) -> ExitCode
{
    let (depending, depended) = pair;

    return match answer
    {
        Admissibility::Permitted =>
        {
            let _ = writeln!(stdout, "permitted: {depending} may name {depended}.");
            ExitCode::Ok
        }
        Admissibility::Refused =>
        {
            let _ = writeln!(
                stdout,
                "refused: {depending} may not name {depended} under this repository's declared \
                 architecture."
            );
            ExitCode::Violations
        }
        Admissibility::NotJudged =>
        {
            let _ = writeln!(
                stdout,
                "not judged: {depending} or {depended} has no declared zone, so there is nothing \
                 to judge this edge against. This is not permission."
            );
            ExitCode::Ok
        }
    };
}
