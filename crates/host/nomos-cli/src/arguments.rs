//! Reading named arguments off a command line.
//!
//! Shared by every group rather than reimplemented per group. Two copies of "find the
//! value after this flag" would eventually disagree about a repeated flag or a flag with
//! no value, and the two groups would parse the same command line differently — which is
//! the sort of difference nobody looks for because nobody believes it is possible.

/// The value following `name`, if it is present.
#[must_use]
pub(crate) fn Named_Value(arguments: &[String], name: &str) -> Option<String>
{
    let position = arguments.iter().position(|argument| return argument == name)?;

    return arguments.get(position.saturating_add(1)).cloned();
}

/// Every value given for a repeatable flag.
///
/// Repeating rather than comma-splitting, because a path may contain a comma and a
/// separator character invents a quoting problem the argument vector already solved.
#[must_use]
pub(crate) fn Named_Values(arguments: &[String], name: &str) -> Vec<String>
{
    let mut values = Vec::new();
    let mut index = 0_usize;

    while let Some(argument) = arguments.get(index)
    {
        if argument == name
            && let Some(value) = arguments.get(index.saturating_add(1))
        {
            values.push(value.clone());
            index = index.saturating_add(2);
            continue;
        }
        index = index.saturating_add(1);
    }

    return values;
}

/// A required value, or a message naming what is missing and what the group accepts.
///
/// # Errors
///
/// Returns the message when the value is absent.
pub(crate) fn Required(value: Option<&String>, name: &str, usage: &str) -> Result<String, String>
{
    return value
        .cloned()
        .ok_or_else(|| return format!("{name} is required.\n\n{usage}"));
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }

    #[test]
    fn Test_A_Repeated_Flag_Should_Yield_Every_Value()
    {
        let arguments = Arguments("--path a --path b --other c --path d");

        assert_eq!(Named_Values(&arguments, "--path"), vec!["a", "b", "d"]);
        assert_eq!(Named_Value(&arguments, "--path").as_deref(), Some("a"));
    }

    /// A flag at the end of the line has no value. Reading past it would take the next
    /// flag as its value, which is how `--into --profile x` becomes a directory named
    /// `--profile`.
    #[test]
    fn Test_A_Flag_With_No_Value_Should_Be_Absent()
    {
        let arguments = Arguments("--profile github-markdown --into");

        assert_eq!(Named_Value(&arguments, "--into"), None);
        assert!(Named_Values(&arguments, "--into").is_empty());
    }

    #[test]
    fn Test_A_Missing_Required_Value_Should_Name_Itself()
    {
        let error = Required(None, "--item", "usage: nomos work").expect_err("must refuse");

        assert!(error.contains("--item"));
        assert!(error.contains("usage"));
    }
}
