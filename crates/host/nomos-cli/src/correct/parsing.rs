//! What `nomos correct phantom-mirrors` was asked for.

use super::{CorrectCommand, Named_Value_From_String_Arguments, PathBuf};

pub(super) const USAGE: &str = "usage: nomos correct phantom-mirrors [--root <path>] [--commit]\n\n\
     Runs Check_Completeness_Mirrors and Check_No_Trailing_Whitespace over the tree. For \
     the first real blocking claim either recognizes -- a doc comment claiming a check by \
     name that does not exist, or a file carrying trailing whitespace -- builds a real \
     CorrectionCandidate that fixes exactly that and nothing else, stages and validates it \
     against the file's own real content, and reports what it would do. The verb name is \
     historical: it named this command's first correction family, kept rather than \
     renamed once a second joined it under the same lifecycle.\n\n\
     --commit actually applies it: commits the plan through nomos_corrections::\
     ValidatedPlan::Commit and writes the corrected file to disk. Without --commit nothing \
     on disk changes -- the run stops after staging and validating, which is real proof the \
     plan applies cleanly, not a promise about what committing it would do.\n\n\
     Only a Phantom finding or trailing whitespace is ever corrected. An admitted gap (a \
     universe that claims no mirror at all) is left alone: inventing a check name for one \
     is a judgment this command does not make.\n\n\
     exit codes: 0 nothing to correct, or a correction was staged/validated/[committed] \
     cleanly; 1 a real claim was found but could not be safely corrected (its claim line \
     is not exactly once in the file, or the plan does not stage/validate against live \
     content); 2 usage; 5 unreadable tree; 6 nothing was judged";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when the verb is missing or unrecognized, or an argument is
/// not understood.
pub fn Parse(arguments: &[String]) -> Result<CorrectCommand, String>
{
    let Some((verb, rest)) = arguments.split_first()
    else
    {
        return Err(USAGE.to_owned());
    };

    if verb != "phantom-mirrors"
    {
        return Err(format!("unknown verb `{verb}`.\n\n{USAGE}"));
    }

    if let Some(unknown) = rest
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root" && argument.as_str() != "--commit")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(CorrectCommand {
        root: Named_Value_From_String_Arguments(rest, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
        commit: rest.iter().any(|argument| return argument == "--commit"),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_The_Bare_Verb_Defaults_Root_And_Commit()
    {
        let command = Parse(&Arguments("phantom-mirrors")).expect("parses");

        assert_eq!(command.root, PathBuf::from("."));
        assert!(!command.commit);
    }

    #[test]
    fn Test_An_Explicit_Root_And_Commit_Are_Read()
    {
        let command = Parse(&Arguments("phantom-mirrors --root some/tree --commit")).expect("parses");

        assert_eq!(command.root, PathBuf::from("some/tree"));
        assert!(command.commit);
    }

    #[test]
    fn Test_No_Verb_Is_A_Usage_Error()
    {
        let error = Parse(&[]).expect_err("must refuse");

        assert!(error.contains("usage"));
    }

    #[test]
    fn Test_An_Unknown_Verb_Is_A_Usage_Error()
    {
        let error = Parse(&Arguments("dance")).expect_err("must refuse");

        assert!(error.contains("dance"));
    }

    #[test]
    fn Test_An_Unknown_Flag_Is_A_Usage_Error()
    {
        let error = Parse(&Arguments("phantom-mirrors --wat")).expect_err("must refuse");

        assert!(error.contains("--wat"));
    }

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }
}
