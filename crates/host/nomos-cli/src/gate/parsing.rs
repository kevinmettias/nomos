//! What `nomos gate` was asked for.

use super::{GateCommand, Named_Value, PathBuf};

pub(super) const USAGE: &str = "usage: nomos gate plan [--root <path>]\n\n\
     Composes this gate's rule registry and reports what it holds.\n\n\
     exit codes: 0 the plan was composed and reported, 2 usage,\n\
     \x20           5 the rule registry is self-contradictory";

/// Parses the group's arguments.
///
/// `run`, `explain` and `compare` are `ARC-ROADMAP-001`'s other three named verbs, but
/// none has a real implementation behind it yet -- see
/// `nomos_gate_orchestration`'s own `lib.rs` doc -- so only `plan` is recognized here.
/// Accepting an unimplemented verb name and silently running `plan` instead would answer
/// a question nobody asked; refusing it as usage is the same "no invented shape ahead of a
/// real body" choice the crate itself already made.
///
/// # Errors
///
/// Returns the usage message when the verb is missing or unrecognized, or an argument is
/// not understood.
pub fn Parse(arguments: &[String]) -> Result<GateCommand, String>
{
    let Some((verb, rest)) = arguments.split_first()
    else
    {
        return Err(USAGE.to_owned());
    };

    if verb != "plan"
    {
        return Err(format!("unknown verb `{verb}`.\n\n{USAGE}"));
    }

    if let Some(unknown) = rest
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(GateCommand {
        root: Named_Value(rest, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    });
}
