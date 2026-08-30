//! Which of a policy tool's own severities one violation was reported at.

/// Which of a policy tool's own severities one violation was reported at.
///
/// Two values, not the fuller set a diagnostic renderer carries (`note`, `help`): the
/// same restriction `nomos_cap_lint::LintLevel` already applies for the identical reason —
/// `cargo deny`'s own JSON stream reports every top-level `bans`/`licenses`/`sources`
/// diagnostic at one of these two, verified directly against this workspace's own `cargo
/// deny --format json check bans licenses sources` output before this type was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicySeverity
{
    Warning,
    Error,
}

impl PolicySeverity
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

impl core::fmt::Display for PolicySeverity
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

    #[test]
    fn Test_From_Label_Should_Round_Trip_Every_Severity_Through_Its_Label()
    {
        for severity in All_Policy_Severities()
        {
            assert_eq!(PolicySeverity::From_Label(severity.Label()), Some(severity));
        }
    }

    /// Every declared severity, named so a second test could point at the same list rather
    /// than writing its own.
    fn All_Policy_Severities() -> Vec<PolicySeverity>
    {
        return vec![PolicySeverity::Warning, PolicySeverity::Error];
    }

    #[test]
    fn Test_An_Unrecognized_Label_Should_Resolve_To_Nothing()
    {
        assert_eq!(PolicySeverity::From_Label("note"), None);
        assert_eq!(PolicySeverity::From_Label(""), None);
    }
}
