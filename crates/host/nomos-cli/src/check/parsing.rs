//! What `nomos check` was asked for.

use super::{CheckCommand, Named_Value, PathBuf};

pub(super) const USAGE: &str = "usage: nomos check [--root <path>]\n\n\
     Runs every rule over the tree and reports what they find.\n\n\
     rules:\n  \
     completeness-mirror   a declared universe must name a check that compares it\n\
     \x20                     against the reality it enumerates, and that check must\n\
     \x20                     exist. See OD-COMPLETENESS-001.\n\n\
     exit codes: 0 nothing blocking, 1 findings that can fail a build, 2 usage,\n\
     \x20           5 unreadable tree, 6 nothing was judged";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when an argument is not understood.
pub fn Parse(arguments: &[String]) -> Result<CheckCommand, String>
{
    if let Some(unknown) = arguments
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(CheckCommand {
        root: Named_Value(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    });
}
