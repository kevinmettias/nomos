//! Reading named arguments off a command line.
//!
//! Shared by every group rather than reimplemented per group. Two copies of "find the
//! value after this flag" would eventually disagree about a repeated flag or a flag with
//! no value, and the two groups would parse the same command line differently — which is
//! the sort of difference nobody looks for because nobody believes it is possible.

/// The value following `name`, if it is present.
#[must_use]
pub(crate) fn Named_Value_From_String_Arguments(arguments: &[String], name: &str) -> Option<String>
{
    let position = arguments.iter().position(|argument| return argument == name)?;

    return arguments.get(position.saturating_add(1)).cloned();
}

/// Every value given for a repeatable flag.
///
/// Repeating rather than comma-splitting, because a path may contain a comma and a
/// separator character invents a quoting problem the argument vector already solved.
/// A named value spends two arguments: the name, and the value standing after it.
const NAME_AND_VALUE: usize = 2;

#[must_use]
pub(crate) fn Named_Values_From_String_Arguments(arguments: &[String], name: &str) -> Vec<String>
{
    let mut values = Vec::new();
    let mut index = 0_usize;

    while let Some(argument) = arguments.get(index)
    {
        if argument == name
            && let Some(value) = arguments.get(index.saturating_add(1))
        {
            values.push(value.clone());
            index = index.saturating_add(NAME_AND_VALUE);
            continue;
        }
        index = index.saturating_add(1);
    }

    return values;
}

/// The flag's own name, e.g. `--item` -- echoed back into the refusal message so a
/// caller learns what was missing. A distinct type from [`Usage`] only so the two
/// adjacent `&str` positions in [`Required`] cannot be swapped without the compiler
/// noticing.
pub(crate) struct Name<'a>(pub(crate) &'a str);

/// The usage text appended after [`Name`] in a refusal, so a caller sees not just what
/// was missing but how to supply it.
pub(crate) struct Usage<'a>(pub(crate) &'a str);

/// A required value, or a message naming what is missing and what the group accepts.
///
/// # Errors
///
/// Returns the message when the value is absent.
pub(crate) fn Required_Value(value: Option<&String>, name: Name<'_>, usage: Usage<'_>) -> Result<String, String>
{
    return value
        .cloned()
        .ok_or_else(|| return format!("{} is required.\n\n{}", name.0, usage.0));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Repeated_Flag_Should_Yield_Every_Value()
    {
        let arguments = Arguments_From_Text("--path a --path b --other c --path d");

        assert_eq!(Named_Values_From_String_Arguments(&arguments, "--path"), vec!["a", "b", "d"]);
        assert_eq!(Named_Value_From_String_Arguments(&arguments, "--path").as_deref(), Some("a"));
    }

    /// A flag at the end of the line has no value. Reading past it would take the next
    /// flag as its value, which is how `--into --profile x` becomes a directory named
    /// `--profile`.
    #[test]
    fn Test_A_Flag_With_No_Value_Should_Be_Absent()
    {
        let arguments = Arguments_From_Text("--profile github-markdown --into");

        assert_eq!(Named_Value_From_String_Arguments(&arguments, "--into"), None);
        assert!(Named_Values_From_String_Arguments(&arguments, "--into").is_empty());
    }

    #[test]
    fn Test_A_Missing_Required_Value_Should_Name_Itself()
    {
        let error = Required_Value(None, Name("--item"), Usage("usage: nomos work")).expect_err("must refuse");

        assert!(error.contains("--item"));
        assert!(error.contains("usage"));
    }

    fn Arguments_From_Text(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }
}
