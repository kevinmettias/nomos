//! What a check run names, and why that is one field rather than four.

use nomos_check_orchestration::CheckCommand;
use serde::Deserialize;
use std::path::PathBuf;

/// The arguments `nomos.check.run` takes.
///
/// One field, because `nomos check` itself accepts exactly one -- `--root`, the whole of
/// `crates/host/nomos-cli/src/check/parsing.rs`' own usage line -- so a wire caller and a
/// shell caller narrow a run by the same vocabulary rather than by two that have to be kept
/// in step. `nomos_check_orchestration::CheckCommand` carries exactly that one field too,
/// so nothing here is deliberately withheld the way [`crate::GateParameters`] withholds
/// `GateCommand`'s five policy fields.
///
/// The absence of a scope or rule selector is `nomos check`'s own decision rather than this
/// transport's: that verb runs every registered rule over everything it walks, and a caller
/// wanting a narrowed run is asking for `nomos.gate.run`, which is where the four selectors
/// live. `OD-HOST-014` admits this operation for what it causes on the host -- it walks and
/// judges the tree it is given and reads nothing else -- and a selector would not change
/// that.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CheckParameters
{
    /// The tree to judge. Absent, the serving process's own working directory, which is what
    /// `nomos check` does with an absent `--root`.
    pub root: Option<PathBuf>,
}

impl CheckParameters
{
    /// These arguments as the command the orchestration seam takes.
    #[must_use]
    pub fn Command(&self) -> CheckCommand
    {
        return CheckCommand { root: self.root.clone().unwrap_or_else(|| return PathBuf::from(".")) };
    }
}

#[cfg(test)]
mod tests
{
    use super::CheckParameters;
    use std::path::PathBuf;

    /// Absent arguments reach the same command an argument-free `nomos check` builds: the
    /// working directory.
    #[test]
    fn Test_Absent_Arguments_Should_Reach_The_Same_Command_The_Cli_Builds()
    {
        let parameters: CheckParameters = serde_json::from_str("{}").expect("an empty object is every field at its default");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("."));
    }

    /// The one argument reaches the field of `CheckCommand` it names.
    #[test]
    fn Test_Every_Argument_Should_Reach_The_Command_Field_It_Names()
    {
        let parameters: CheckParameters =
            serde_json::from_str(r#"{"root":"some/tree"}"#).expect("every field is one this type declares");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("some/tree"));
    }

    /// A field this type does not declare is refused rather than dropped -- so a caller
    /// sending `nomos.gate.run`'s selectors learns this operation does not carry them,
    /// instead of being told nothing and getting a run that ignored them.
    #[test]
    fn Test_An_Undeclared_Argument_Should_Be_Refused_Rather_Than_Dropped()
    {
        let refused = serde_json::from_str::<CheckParameters>(r#"{"rules":[]}"#);

        assert!(refused.is_err(), "{refused:?}");
    }
}
