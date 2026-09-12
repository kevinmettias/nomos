

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A named enforcer of a rule.
///
/// The name must be a **tool identity**, not a path and not a CI job name. xvpe records
/// the failure directly: one of its rules names
/// `[fast-tier-benchmarks, .github/workflows/pr-fast.yml, tools/perf/fast-tier-bench]`,
/// and because the resolver matches tool-module leaf names, none of the three resolves.
/// The rule silently classifies as unreachable while its own prose says it is advisory.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EnforcerRef
{
    /// A human judges this. The honest declaration when no tool can.
    Review,
    /// A registered check, named by its identity.
    Check
    {
        /// The check's registered name.
        name: String,
    },
    /// A setting in a tool this repository does not write, such as an analyzer
    /// configuration entry. The setting must exist in a template the repository ships,
    /// or the delegation is a claim with nothing behind it.
    External
    {
        /// The delegated-to tool, such as `editorconfig`.
        tool: String,
        /// The setting within it, such as `CA1822`.
        setting: String,
    },
}

impl core::fmt::Display for EnforcerRef
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Review => formatter.write_str("review"),
            Self::Check { name } => formatter.write_str(name),
            Self::External { tool, setting } => write!(formatter, "{tool}:{setting}"),
        };
    }
}

#[cfg(test)]
mod tests
{
    use alloc::string::ToString;
    use alloc::borrow::ToOwned;
    use super::*;

    #[test]
    fn Test_External_Enforcer_Should_Render_With_Its_Colon_Form()
    {
        let external = EnforcerRef::External {
            tool: "editorconfig".to_owned(),
            setting: "CA1822".to_owned(),
        };

        assert_eq!(external.to_string(), "editorconfig:CA1822");
    }
}
