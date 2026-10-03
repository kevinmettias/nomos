//! What a finish found for a gate step the workflow is free not to declare.

use super::GateOutcome;
use serde::Deserialize;
use serde::Serialize;

/// The gate's `Rules` step as a finish met it: run, or never declared.
///
/// Two values and not one, because "the workflow declares no such step" and "the step ran and
/// passed" are opposite answers that a bare `Option` would let a reader confuse. `OD-GATE-036`
/// lets a workflow decline the step -- the same ledger serves KWB, whose gate declares `Lint`,
/// `Test` and `Contract` -- and requires that a finish in such a repository record the step as
/// not declared, never as passed.
///
/// There is no failing value. A step that ran and exited nonzero refused the finish, so no
/// verification record exists to carry it; one whose command could not be derived refused
/// before anything ran.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OptionalStepOutcome
{
    /// The workflow declares the step, and this is what ran and what it exited with.
    Ran(GateOutcome),
    /// The workflow declares no step by that name, so nothing ran in its place.
    NotDeclared,
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The two values serialize apart, so a reader of the ledger file can tell a step that
    /// passed from a step that was never declared without knowing this type.
    #[test]
    fn Test_A_Step_That_Ran_And_A_Step_Never_Declared_Should_Serialize_Apart()
    {
        let ran = OptionalStepOutcome::Ran(GateOutcome { argv: vec!["cargo".to_owned()], exit_code: 0 });

        let ran_text = serde_json::to_string(&ran).expect("a step outcome serializes");
        let absent_text = serde_json::to_string(&OptionalStepOutcome::NotDeclared).expect("a step outcome serializes");

        assert_eq!(ran_text, r#"{"Ran":{"argv":["cargo"],"exit_code":0}}"#);
        assert_eq!(absent_text, r#""NotDeclared""#);
        assert_eq!(serde_json::from_str::<OptionalStepOutcome>(&ran_text).expect("it reads back"), ran);
    }
}
