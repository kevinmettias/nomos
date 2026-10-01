//! What a correction run names, and the one field deliberately absent from it.

use nomos_correction_orchestration::CorrectionCommand;
use serde::Deserialize;
use std::path::PathBuf;

/// The arguments `nomos.correction.run` takes.
///
/// These two are exactly what `nomos correct` itself accepts -- `--root` and `--commit` --
/// so a wire caller and a shell caller narrow a run by the same vocabulary rather than by
/// two that have to be kept in step. Unlike [`super::GateParameters`], there is no third
/// field `nomos_correction_orchestration::CorrectionCommand` withholds: that command carries
/// only `root` and `commit`, so nothing here is deliberately left out the way `GateCommand`'s
/// five unauthored policy fields are.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CorrectionParameters
{
    /// The tree to judge. Absent, the serving process's own working directory, which is what
    /// `nomos correct` does with an absent `--root`.
    pub root: Option<PathBuf>,
    /// Whether to actually commit and write the corrected file, or stop after staging and
    /// validating it. Absent, `false` -- a dry run, the same default `nomos correct` gives
    /// an absent `--commit`.
    pub commit: bool,
}

impl CorrectionParameters
{
    /// These arguments as the command the orchestration seam takes.
    #[must_use]
    pub fn Command(&self) -> CorrectionCommand
    {
        return CorrectionCommand { root: self.root.clone().unwrap_or_else(|| return PathBuf::from(".")), commit: self.commit };
    }
}

#[cfg(test)]
mod tests
{
    use super::CorrectionParameters;
    use std::path::PathBuf;

    /// Absent arguments reach the same command an argument-free `nomos correct` builds: the
    /// working directory, and no commit.
    #[test]
    fn Test_Absent_Arguments_Should_Reach_The_Same_Command_The_Cli_Builds()
    {
        let parameters: CorrectionParameters = serde_json::from_str("{}").expect("an empty object is every field at its default");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("."));
        assert!(!command.commit);
    }

    /// Every argument reaches the field of `CorrectionCommand` it names.
    #[test]
    fn Test_Every_Argument_Should_Reach_The_Command_Field_It_Names()
    {
        let body = r#"{"root":"some/tree","commit":true}"#;
        let parameters: CorrectionParameters = serde_json::from_str(body).expect("every field is one this type declares");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("some/tree"));
        assert!(command.commit);
    }

    /// A field this type does not declare is refused rather than dropped -- so a caller
    /// setting one learns the transport does not carry it, instead of being told nothing and
    /// getting a run that ignored it.
    #[test]
    fn Test_An_Undeclared_Argument_Should_Be_Refused_Rather_Than_Dropped()
    {
        let refused = serde_json::from_str::<CorrectionParameters>(r#"{"rules":[]}"#);

        assert!(refused.is_err(), "{refused:?}");
    }
}
