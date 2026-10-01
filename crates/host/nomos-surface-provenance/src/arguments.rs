//! Reading `--since`, `--until`, `--root` and a repeatable `--crate` off a command line.
//!
//! Deliberately not shared with `nomos-cli::arguments` — that module is private to its
//! own crate, and duplicating eight lines of "find the value after this flag" is cheaper
//! than the dependency edge a shared crate would cost a tool this small.

use nomos_platform::Environment;
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
/// `environment` decides what `--root` defaults to. Read through the port rather than
/// from `std::env::current_dir` since `P86`: the default is the one parsed value this
/// function does not take from `arguments`, so it was the one value no test could state
/// — and the two tests below that parse a line without `--root` said nothing about
/// `root` for exactly that reason.
///
/// # Errors
///
/// Returns [`USAGE`] when `--since` or `--until` is missing, and [`USAGE`] followed by
/// the port's own explanation when `--root` was omitted and the working directory it
/// would have defaulted to cannot be read.
pub(crate) fn Parsed_From_String_Arguments(
    arguments: &[String],
    environment: &impl Environment,
) -> Result<Parsed, String>
{
    let since = Named_Value_From_String_Arguments(arguments, "--since").ok_or(USAGE)?;
    let until = Named_Value_From_String_Arguments(arguments, "--until").ok_or(USAGE)?;
    let root = Named_Value_From_String_Arguments(arguments, "--root")
        .map(PathBuf::from)
        .map_or_else(
            // One spelling of this failure, not two: the port already renders it as "the
            // current directory could not be read: <cause>", which is what this line used
            // to spell itself. Appending that to USAGE keeps the usage-first shape every
            // other refusal here has.
            || environment.Working_Directory().map_err(|error| format!("{USAGE}\n\n{error}")),
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
    use crate::stated_environment::Stated;
    use std::path::Path;

    #[test]
    fn Test_Parsed_From_String_Arguments_Should_Require_Since_And_Until()
    {
        let missing_since = match Parsed_From_String_Arguments(&Arguments_From_Text("--until HEAD"), &Anywhere())
        {
            Err(error) => error,
            // This argument list omits --since on purpose, and Parsed_From_String_Arguments
            // requires it unconditionally -- reaching Ok here means that requiredness check
            // itself stopped enforcing, not a condition this test should assert around.
            Ok(_) => panic!("missing --since must refuse"),
        };
        let missing_until = match Parsed_From_String_Arguments(&Arguments_From_Text("--since HEAD~5"), &Anywhere())
        {
            Err(error) => error,
            // This argument list omits --until on purpose, and Parsed_From_String_Arguments
            // requires it unconditionally -- reaching Ok here means that requiredness check
            // itself stopped enforcing, not a condition this test should assert around.
            Ok(_) => panic!("missing --until must refuse"),
        };

        // `USAGE` is the one message either branch of this refusal can produce -- the root
        // path's own errors append text past it, so an exact match here also rules out
        // this test passing because a later stage refused for an unrelated reason.
        assert_eq!(missing_since, USAGE);
        assert_eq!(missing_until, USAGE);
    }

    #[test]
    fn Test_A_Minimal_Line_Parses_With_No_Crates_Named()
    {
        let environment = Stated::At(Path::new("/stated/working/directory"));

        let parsed = Parsed_From_String_Arguments(&Arguments_From_Text("--since a --until b"), &environment)
            .expect("the line above names every flag this call requires");

        assert_eq!(parsed.since, "a");
        assert_eq!(parsed.until, "b");
        assert!(parsed.crates.is_empty());
        // The assertion this test could not make before `P86`. The working directory is
        // deliberately not the one this test process is standing in, so a default that
        // still read `std::env::current_dir` would fail here rather than agree by
        // accident with whatever the test also read.
        assert_eq!(parsed.root, PathBuf::from("/stated/working/directory"));
    }

    /// The control for the assertion above: an explicit `--root` still wins, so that
    /// assertion is about the *default* rather than about `root` being set at all.
    #[test]
    fn Test_An_Explicit_Root_Wins_Over_The_Working_Directory()
    {
        let environment = Stated::At(Path::new("/stated/working/directory"));

        let parsed = Parsed_From_String_Arguments(&Arguments_From_Text("--since a --until b --root /given"), &environment)
            .expect("the line above names every flag this call requires");

        assert_eq!(parsed.root, PathBuf::from("/given"));
    }

    /// A working directory that cannot be read is a usage refusal carrying the port's own
    /// explanation, not a panic and not a silent fallback to some other directory.
    #[test]
    fn Test_An_Unreadable_Working_Directory_Refuses_With_Usage_And_A_Reason()
    {
        // `let ... else` rather than `expect_err`, because `Parsed` carries no `Debug` --
        // and deriving one on a production type so a test can phrase itself more briefly is
        // a widening of the crate's surface for the test's convenience. The two refusal
        // tests above match instead, each keeping an explanation inside its own `Ok` arm
        // that this one has no need of.
        let Err(error) = Parsed_From_String_Arguments(&Arguments_From_Text("--since a --until b"), &Stated::Standing_Nowhere())
        else
        {
            panic!("a fixture standing nowhere has no directory to default --root to");
        };

        assert!(error.starts_with(USAGE), "{error}");
        assert!(error.contains("the current directory could not be read: "), "{error}");
    }

    #[test]
    fn Test_Crate_Repeats_And_Root_Are_Read()
    {
        let parsed = Parsed_From_String_Arguments(
            &Arguments_From_Text("--since a --until b --root /work --crate nomos-model --crate nomos-store"),
            &Anywhere(),
        )
        .expect("the line above names every flag this call requires");

        assert_eq!(parsed.root, PathBuf::from("/work"));
        assert_eq!(parsed.crates, vec!["nomos-model", "nomos-store"]);
    }

    /// Where a test that does not care about `--root` stands. Named rather than repeated
    /// so the two tests that *do* care read as deliberately different from the ones that
    /// do not.
    fn Anywhere() -> Stated
    {
        return Stated::At(Path::new("/anywhere"));
    }

    fn Arguments_From_Text(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }
}
