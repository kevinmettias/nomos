//! Which of a lint tool's own severities one diagnostic was reported at.

/// Which of a lint tool's own severities one diagnostic was reported at.
///
/// Two values, not the fuller set a compiler's own diagnostic renderer carries (`note`,
/// `help`, `failure-note`): this capability answers for one diagnostic's own verdict, and
/// a tool's `note`/`help` messages are attached explanation for a `warning`/`error`, never
/// a diagnostic of their own — `nomos-lang-rust-clippy`'s own reader keeps only the two a
/// real `cargo clippy --message-format=json` run ever reports at its top level, verified
/// directly against this workspace's own output before this type was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintLevel
{
    Warning,
    Error,
}

impl LintLevel
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Warning => "warning",
            Self::Error => "error",
        };
    }

    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            _ => None,
        };
    }
}

impl core::fmt::Display for LintLevel
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every declared level, named so a second test could point at the same list rather than
    /// writing its own.
    fn All_Lint_Levels() -> Vec<LintLevel>
    {
        return vec![LintLevel::Warning, LintLevel::Error];
    }

    #[test]
    fn Test_From_Label_Should_Round_Trip_Every_Level_Through_Its_Label()
    {
        for level in All_Lint_Levels()
        {
            assert_eq!(LintLevel::From_Label(level.Label()), Some(level));
        }
    }

    #[test]
    fn Test_An_Unrecognized_Label_Should_Resolve_To_Nothing()
    {
        assert_eq!(LintLevel::From_Label("note"), None);
        assert_eq!(LintLevel::From_Label(""), None);
    }
}
