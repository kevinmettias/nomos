//! What `nomos profile` was asked for.

use super::{Named_Value_From_String_Arguments, PathBuf, ProfileCommand};

/// The flag asking for a starting gate policy file.
const WRITE_GATE_POLICY: &str = "--write-gate-policy";

/// The flag naming the tree.
const ROOT: &str = "--root";

/// Every flag this group accepts.
const KNOWN_ARGUMENTS: [&str; 2] = [ROOT, WRITE_GATE_POLICY];

pub(super) const USAGE: &str = "usage: nomos profile [--root <path>] [--write-gate-policy]\n\n\
     Reports what a root holds before anything judges it: how many sources of each \
registered language the walk finds, which language manifests sit at the root, which \
repository policy files are present and which are absent, and whether the root carries the \
repository marker. An absence is printed as plainly as a presence, because an absent policy \
file is the actionable half.\n\n\
     It also reports what this host settles about answering each capability this workspace \
declares. A provider that answers from inside this binary is available here whatever the \
machine carries; a provider that runs a program is available only if that program is, and \
where it is not, the tool is named. Where a program is present but whether it answers could \
only be established by running it, this verb reports the capability as undetermined rather \
than guessing -- running the tools is what a real check run is for.\n\n\
     --write-gate-policy writes a starting `nomos-gate.json` for a root that has none. It \
refuses rather than overwriting one that exists, and what it writes declares nothing: a \
starter gate that blocked on findings nobody has read yet would teach its reader to bypass \
it.\n\n\
     exit codes: 0 the profile was rendered and any starter file asked for was written, \
2 usage,\n\
     \x20           4 a starter file was asked for and one is already there, so nothing was \
written,\n\
     \x20           5 the root is not a directory, this build's own composition is \
contradictory, or the starter file could not be written";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when an argument is not understood.
pub fn Profile_Command_From_String_Arguments(arguments: &[String]) -> Result<ProfileCommand, String>
{
    No_Unknown_Argument(arguments)?;

    return Ok(ProfileCommand {
        root: Named_Value_From_String_Arguments(arguments, ROOT).map_or_else(|| return PathBuf::from("."), PathBuf::from),
        write_gate_policy: arguments.iter().any(|argument| return argument == WRITE_GATE_POLICY),
    });
}

/// Refuses a flag this group does not accept, rather than ignoring it.
///
/// A mistyped `--write-gate-policies` that parsed as "no starter file wanted" would report
/// success for a write that never happened, which is the one failure this verb must not
/// have: the person is here because they do not yet know what this tool does.
fn No_Unknown_Argument(arguments: &[String]) -> Result<(), String>
{
    let unknown = arguments
        .iter()
        .find(|argument| return argument.starts_with('-') && !KNOWN_ARGUMENTS.contains(&argument.as_str()));

    return match unknown
    {
        Some(argument) => Err(format!("unknown argument `{argument}`.\n\n{USAGE}")),
        None => Ok(()),
    };
}
