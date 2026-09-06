//! What a run names, and the fields deliberately absent from it.

use nomos_contracts::RuleId;
use nomos_gate_orchestration::{GateCommand, RuleSelector, ScopeSelector};
use serde::Deserialize;
use std::path::PathBuf;

/// The arguments `nomos.gate.run` takes.
///
/// These four are exactly what `nomos gate run` itself accepts -- `--root`, `--include`,
/// `--exclude` and `--rule`, per `crates/host/nomos-cli/src/gate/parsing.rs`'s own
/// `KNOWN_ARGUMENTS` -- so a wire caller and a shell caller narrow a run by the same
/// vocabulary rather than by two that have to be kept in step.
///
/// [`GateCommand`]'s five remaining fields are absent rather than accepted and ignored, and
/// since `P40-GATE-POLICY-AUTHORING-3` that absence means two different things.
///
/// `suppressions`, `baseline`, `adoption` and `coverage` are now resolved by `Run_Gate`
/// itself, from the `nomos-gate.json` under the run's own root. Leaving them off the wire is
/// therefore what makes them work rather than what withholds them: a request that left them
/// at their defaults is exactly the request that picks the served tree's declared policy up,
/// and a wire caller overriding a repository's own suppressions per-request is a different
/// and much larger question than this transport should answer by accident.
///
/// `model` is the one field still absent for the original reason: it is read by nothing, and
/// a wire field a caller can set that changes no outcome is a promise this transport cannot
/// keep, worse than its absence because absence is legible.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GateParameters
{
    /// The tree to judge. Absent, the serving process's own working directory, which is what
    /// `nomos gate run` does with an absent `--root`.
    pub root: Option<PathBuf>,
    /// Which files under `root` a run judges. Empty selects everything.
    pub include: Vec<String>,
    /// Which files under `root` a run does not judge, applied after `include`.
    pub exclude: Vec<String>,
    /// Which rules' findings count toward the disposition. Empty selects every registered
    /// rule.
    pub rules: Vec<String>,
}

impl GateParameters
{
    /// These arguments as the command the orchestration seam takes.
    #[must_use]
    pub fn Command(&self) -> GateCommand
    {
        return GateCommand {
            root: self.root.clone().unwrap_or_else(|| return PathBuf::from(".")),
            scope: ScopeSelector { include: self.include.clone(), exclude: self.exclude.clone() },
            rules: RuleSelector { include: self.rules.iter().map(RuleId::New).collect() },
            ..GateCommand::default()
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::GateParameters;
    use std::path::PathBuf;

    /// Absent arguments reach the same command an argument-free `nomos gate run` builds:
    /// the working directory, and every selector selecting everything.
    #[test]
    fn Test_Absent_Arguments_Should_Reach_The_Same_Command_The_Cli_Builds()
    {
        let parameters: GateParameters = serde_json::from_str("{}").expect("an empty object is every field at its default");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("."));
        assert!(command.scope.include.is_empty());
        assert!(command.scope.exclude.is_empty());
        assert!(command.rules.include.is_empty());
    }

    /// Every argument reaches the field of `GateCommand` it names.
    #[test]
    fn Test_Every_Argument_Should_Reach_The_Command_Field_It_Names()
    {
        let body = r#"{"root":"some/tree","include":["a.rs"],"exclude":["b.rs"],"rules":["naming-convention"]}"#;
        let parameters: GateParameters = serde_json::from_str(body).expect("every field is one this type declares");

        let command = parameters.Command();

        assert_eq!(command.root, PathBuf::from("some/tree"));
        assert_eq!(command.scope.include, vec!["a.rs".to_owned()]);
        assert_eq!(command.scope.exclude, vec!["b.rs".to_owned()]);
        assert_eq!(command.rules.include.len(), 1, "{:?}", command.rules.include);
        assert_eq!(command.rules.include.first().map(nomos_contracts::RuleId::As_Str), Some("naming-convention"));
    }

    /// A field this type does not declare is refused rather than dropped -- so a caller
    /// setting `suppressions` learns the transport does not carry it, instead of being told
    /// nothing and getting a run that ignored it.
    #[test]
    fn Test_An_Undeclared_Argument_Should_Be_Refused_Rather_Than_Dropped()
    {
        let refused = serde_json::from_str::<GateParameters>(r#"{"suppressions":[]}"#);

        assert!(refused.is_err(), "{refused:?}");
    }
}
