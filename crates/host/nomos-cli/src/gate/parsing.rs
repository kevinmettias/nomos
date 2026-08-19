//! What `nomos gate` was asked for.

use super::{GateCommand, GateInvocation, Named_Value, PathBuf};

pub(super) const USAGE: &str = "usage: nomos gate plan [--root <path>]\n       \
     nomos gate run  [--root <path>]\n\n\
     plan composes this gate's rule registry and reports what it holds.\n\
     run walks the tree, judges it, and reports a real disposition.\n\n\
     exit codes: 0 clean (the plan was composed, or nothing judged can fail a build),\n\
     \x20           1 at least one finding can fail a build, 2 usage,\n\
     \x20           5 this build's own composition is self-contradictory, or the tree could \
not be read,\n\
     \x20           6 nothing was judged: the walk found no source, or no fact was \
materialized for any of it";

/// Parses the group's arguments.
///
/// `explain` and `compare` are `ARC-ROADMAP-001`'s other two named verbs, but neither has
/// a real implementation behind it yet -- see `nomos_gate_orchestration`'s own `lib.rs`
/// doc -- so only `plan` and `run` are recognized here. Accepting an unimplemented verb
/// name and silently running `plan` instead would answer a question nobody asked;
/// refusing it as usage is the same "no invented shape ahead of a real body" choice the
/// crate itself already made.
///
/// # Errors
///
/// Returns the usage message when the verb is missing or unrecognized, or an argument is
/// not understood.
pub fn Parse(arguments: &[String]) -> Result<GateInvocation, String>
{
    let Some((verb, rest)) = arguments.split_first()
    else
    {
        return Err(USAGE.to_owned());
    };

    if verb != "plan" && verb != "run"
    {
        return Err(format!("unknown verb `{verb}`.\n\n{USAGE}"));
    }

    if let Some(unknown) = rest
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    let command = GateCommand {
        root: Named_Value(rest, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    };

    return Ok(if verb == "run" { GateInvocation::Run(command) } else { GateInvocation::Plan(command) });
}
