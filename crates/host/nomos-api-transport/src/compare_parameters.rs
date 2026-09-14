//! The arguments `nomos.gate.compare` carries: two whole sides.
//!
//! # Why two whole sides rather than one command and a second root
//!
//! Because `OD-GATE-022-A` names two cases and they vary different things. A code comparison
//! holds the policy fixed and varies the tree -- two roots. A policy comparison holds the tree
//! fixed and varies `rules`, and whatever a `nomos-gate.json` under that root contributes --
//! one root, two policies. A shape carrying one [`crate::GateParameters`] plus a bare second
//! path serves the first and cannot express the second at all, and would have to grow a second
//! shape the moment somebody asked for it. Two sides serve both, and neither case is a special
//! case of the other.
//!
//! Each side is exactly [`crate::GateParameters`], the shape `nomos.gate.run` already takes, so
//! a caller that can describe one run can describe both and has no second vocabulary to learn.

use crate::GateParameters;
use nomos_gate_orchestration::GateCommand;
use serde::Deserialize;

/// The two runs a compare call wants judged.
///
/// Both sides default the same way a single `nomos.gate.run` call does, so a caller sending
/// `{}` compares this tree against itself under the default policy -- a call that answers
/// "nothing changed" rather than one this transport has to refuse.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompareParameters
{
    /// The run compared against.
    pub baseline: GateParameters,
    /// The run being compared.
    pub candidate: GateParameters,
}

impl CompareParameters
{
    /// The baseline side, as the command a handler takes.
    #[must_use]
    pub fn Baseline(&self) -> GateCommand
    {
        return self.baseline.Command();
    }

    /// The candidate side, as the command a handler takes.
    #[must_use]
    pub fn Candidate(&self) -> GateCommand
    {
        return self.candidate.Command();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    /// Two roots: the code comparison.
    #[test]
    fn Test_Two_Roots_Should_Reach_Both_Commands()
    {
        let body = r#"{"baseline":{"root":"before"},"candidate":{"root":"after"}}"#;
        let parameters: CompareParameters = serde_json::from_str(body).expect("two sides parse");

        assert_eq!(parameters.Baseline().root, PathBuf::from("before"));
        assert_eq!(parameters.Candidate().root, PathBuf::from("after"));
    }

    /// One root, two rule selections: the policy comparison, which a second-root-only shape
    /// could not have expressed.
    #[test]
    fn Test_One_Root_Under_Two_Policies_Should_Reach_Both_Commands()
    {
        let body = r#"{"baseline":{"root":"."},"candidate":{"root":".","rules":["naming-convention"]}}"#;
        let parameters: CompareParameters = serde_json::from_str(body).expect("two policies parse");

        assert_eq!(parameters.Baseline().root, parameters.Candidate().root);
        assert_ne!(
            parameters.Baseline().rules,
            parameters.Candidate().rules,
            "the two sides must be able to differ by policy alone, which is the case a second \
             bare root could not express"
        );
    }

    /// An empty object is a call, not a refusal.
    #[test]
    fn Test_An_Absent_Body_Should_Default_Both_Sides()
    {
        let parameters: CompareParameters = serde_json::from_str("{}").expect("an empty object parses");

        assert_eq!(parameters.Baseline().root, PathBuf::from("."));
        assert_eq!(parameters.Candidate().root, PathBuf::from("."));
    }

    /// A misspelled side is refused rather than silently defaulted.
    ///
    /// `deny_unknown_fields` is what makes `{"baselines":...}` an error instead of a
    /// comparison of this tree with itself, which would answer "nothing changed" to a caller
    /// who asked something else entirely.
    #[test]
    fn Test_A_Misspelled_Side_Should_Be_Refused()
    {
        let body = r#"{"baselines":{"root":"before"},"candidate":{"root":"after"}}"#;

        assert!(
            serde_json::from_str::<CompareParameters>(body).is_err(),
            "a misspelled side parsed, so a caller's typo would silently compare a default \
             against their candidate and report a difference that is an artifact of the typo"
        );
    }
}
