//! Reading `--since`, `--until`, `--root` and a repeatable `--crate` off a command line.
//!
//! Deliberately not shared with `nomos-cli::arguments` — that module is private to its
//! own crate, and duplicating eight lines of "find the value after this flag" is cheaper
//! than the dependency edge a shared crate would cost a tool this small.

use std::path::PathBuf;

/// One usage message, spelled once, so a refusal and `--help` never drift apart.
pub(crate) const USAGE: &str = "usage: nomos-surface-provenance --since <rev> --until <rev> \
     [--root <path>] [--crate <name> …]\n\n\
     For every crate with a tests/contract/surface/<crate>.txt snapshot (or only the ones \
     named by --crate), reports whether that file's blob differs between --since and \
     --until and whether any commit in that range touched docs/records/. A crate whose \
     surface changed with no accompanying records commit is a candidate for review — never \
     a verdict, and never a reason this exits non-zero. --root defaults to the current \
     directory and must be a git worktree.";

/// A parsed command line, before anything has tried to run against it.
pub(crate) struct Parsed
{
    pub(crate) since: String,
    pub(crate) until: String,
    pub(crate) root: PathBuf,
    /// The crates to check. Empty means "every crate with a snapshot" — resolved later,
    /// once the snapshot directory can be read — rather than defaulted here.
    pub(crate) crates: Vec<String>,
}

/// Parses `arguments` (excluding the program name), or names what usage requires.
///
/// # Errors
///
/// Returns [`USAGE`] when `--since` or `--until` is missing.
pub(crate) fn Parsed_From_String_Arguments(arguments: &[String]) -> Result<Parsed, String>
{
    let since = Named_Value_From_String_Arguments(arguments, "--since").ok_or(USAGE)?;
    let until = Named_Value_From_String_Arguments(arguments, "--until").ok_or(USAGE)?;
    let root = Named_Value_From_String_Arguments(arguments, "--root")
        .map(PathBuf::from)
        .map_or_else(
            || std::env::current_dir().map_err(|error| format!("{USAGE}\n\ncannot read the current directory: {error}")),
            Ok,
        )?;
    let crates = Named_Values_From_String_Arguments(arguments, "--crate");

    return Ok(Parsed {
        since,
        until,
        root,
        crates,
    });
}

/// The value following `name`, if it is present.
fn Named_Value_From_String_Arguments(arguments: &[String], name: &str) -> Option<String>
{
    let position = arguments.iter().position(|argument| return argument == name)?;

    return arguments.get(position.saturating_add(1)).cloned();
}

/// How many argv slots a recognized `--name value` pair occupies: the flag itself and the
/// value right after it — so a match skips both before looking for the next flag.
const FLAG_AND_VALUE_WIDTH: usize = 2;

/// Every value given for a repeatable flag.
fn Named_Values_From_String_Arguments(arguments: &[String], name: &str) -> Vec<String>
{
    let mut values = Vec::new();
    let mut index = 0_usize;

    while let Some(argument) = arguments.get(index)
    {
        if argument == name
            && let Some(value) = arguments.get(index.saturating_add(1))
        {
            values.push(value.clone());
            index = index.saturating_add(FLAG_AND_VALUE_WIDTH);
            continue;
        }
        index = index.saturating_add(1);
    }

    return values;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Since_And_Until_Are_Required()
    {
        assert!(Parsed_From_String_Arguments(&Arguments_From_Text("--until HEAD")).is_err());
        assert!(Parsed_From_String_Arguments(&Arguments_From_Text("--since HEAD~5")).is_err());
    }

    #[test]
    fn Test_A_Minimal_Line_Parses_With_No_Crates_Named()
    {
        let parsed = Parsed_From_String_Arguments(&Arguments_From_Text("--since a --until b")).expect("must parse");

        assert_eq!(parsed.since, "a");
        assert_eq!(parsed.until, "b");
        assert!(parsed.crates.is_empty());
    }

    #[test]
    fn Test_Crate_Repeats_And_Root_Are_Read()
    {
        let parsed = Parsed_From_String_Arguments(&Arguments_From_Text(
            "--since a --until b --root /work --crate nomos-model --crate nomos-store",
        ))
        .expect("must parse");

        assert_eq!(parsed.root, PathBuf::from("/work"));
        assert_eq!(parsed.crates, vec!["nomos-model", "nomos-store"]);
    }

    fn Arguments_From_Text(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }
}
