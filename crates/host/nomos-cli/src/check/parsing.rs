//! What `nomos check` was asked for.

use super::{CheckCommand, Named_Value_From_String_Arguments, PathBuf};

pub(super) const USAGE: &str = "usage: nomos check [--root <path>]\n\n\
     Runs every rule over the tree and reports what they find. Which rules those are is \
     deliberately not named here: `nomos gate plan --root <path>` names every one of them, \
     read out of the rule set this binary composes, so the answer cannot be a rule behind \
     the binary printing it. See `OD-AGENT-004`.\n\n\
     exit codes: 0 nothing blocking, 1 findings that can fail a build, 2 usage,\n\
     \x20           5 unreadable tree, 6 nothing was judged";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when an argument is not understood.
pub fn Check_Command_From_String_Arguments(arguments: &[String]) -> Result<CheckCommand, String>
{
    if let Some(unknown) = arguments
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(CheckCommand {
        root: Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Check_Command_From_String_Arguments_Should_Default_Root_To_The_Current_Directory()
    {
        let command =
            Check_Command_From_String_Arguments(&[]).expect("no argument is present for the parser to refuse");

        assert_eq!(command.root, PathBuf::from("."));
    }

    #[test]
    fn Test_Check_Command_From_String_Arguments_Should_Read_An_Explicit_Root()
    {
        let arguments = vec!["--root".to_owned(), "some/tree".to_owned()];

        let command = Check_Command_From_String_Arguments(&arguments)
            .expect("`--root` is the one flag the check parser accepts, and it carries a value");

        assert_eq!(command.root, PathBuf::from("some/tree"));
    }

    #[test]
    fn Test_Check_Command_From_String_Arguments_Should_Refuse_An_Unknown_Flag()
    {
        let arguments = vec!["--not-a-real-flag".to_owned()];

        let error = Check_Command_From_String_Arguments(&arguments).expect_err("unknown flag");

        assert!(error.contains("unknown argument"), "{error}");
    }
}
