//! [`UndeclaredValueResponse`], one value a run's rule read that the repository never declared, as
//! a judged check outcome carries it.

use super::undeclared_outcome_response::UndeclaredOutcomeResponse;
use nomos_check_orchestration::UndeclaredValues;
use nomos_contracts::RuleId;
use nomos_rules::UndeclaredValue;
use serde::Serialize;

/// A serializable twin of one [`nomos_rules::UndeclaredValue`], with the rule that read it.
///
/// `OD-RULES-011` version 3: a rule that judged against a value this workspace substituted reaches
/// exactly the findings it would have reached had the repository declared that value, and a rule
/// with no norm to judge by reaches none, so a caller that is not told which values nobody declared
/// reads every such verdict as one the repository asked for. One entry per value rather than one
/// per rule, at the grain a repository declares a value, so a caller acts on an entry -- writes the
/// declaration, or does not -- without unnesting it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UndeclaredValueResponse
{
    /// The rule that read the value.
    pub rule: RuleId,
    /// The family: `limits`, `naming`, `scripting`, `goals`, `standards-corpus` or
    /// `requirement-trace`.
    pub family: String,
    /// Where a repository declares it: the file, or for the requirement trace the directory.
    pub declared_in: String,
    /// The key it is declared under.
    pub key: String,
    /// The language the rule read it for, or `None` for a value read repository-wide.
    pub language: Option<String>,
    /// What the rule did with it.
    #[serde(flatten)]
    pub outcome: UndeclaredOutcomeResponse,
}

impl UndeclaredValueResponse
{
    /// Every value `undeclared` names, rule by rule in the order the run judged them and each
    /// rule's values in the order it named them -- `nomos_check_orchestration::UndeclaredValues::
    /// Named`, read and not asked again. Empty when no rule read a value nobody declared.
    pub(crate) fn Every(undeclared: &UndeclaredValues) -> Vec<Self>
    {
        return undeclared
            .Named()
            .iter()
            .flat_map(|(rule, values)| return values.iter().map(move |value| return Self::Of(rule, value)))
            .collect();
    }

    /// `value`, which `rule` read and the repository never declared.
    fn Of(rule: &RuleId, value: &UndeclaredValue) -> Self
    {
        return Self {
            rule: rule.clone(),
            family: value.family.to_owned(),
            declared_in: value.declared_in.to_owned(),
            key: value.key.to_owned(),
            language: value.language.clone(),
            outcome: UndeclaredOutcomeResponse::From(&value.outcome),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_rules::UndeclaredOutcome;

    /// Each value becomes one entry beside its rule, the outcome flattened into it: a value judged
    /// against carries that value, an outcome that judged nothing carries none, and a value read
    /// repository-wide carries a null language.
    #[test]
    fn Test_Every_Should_Name_Each_Value_Beside_Its_Rule_With_What_The_Rule_Did()
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(
            RuleId::New("types-use-upper-camel-case-lower-camel-case"),
            vec![UndeclaredValue {
                family: "naming",
                declared_in: "standards.json",
                key: "type.exported",
                language: Some("go".to_owned()),
                outcome: UndeclaredOutcome::JudgedAgainst { value: "upper-camel".to_owned() },
            }],
        );
        undeclared.Note(
            RuleId::New("goals-and-parts-line-up"),
            vec![UndeclaredValue { family: "goals", declared_in: "standards.json", key: "goals", language: None, outcome: UndeclaredOutcome::JudgedNothing }],
        );

        let rendered = serde_json::to_value(UndeclaredValueResponse::Every(&undeclared)).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(
            rendered,
            serde_json::json!([
                {
                    "rule": "types-use-upper-camel-case-lower-camel-case",
                    "family": "naming",
                    "declared_in": "standards.json",
                    "key": "type.exported",
                    "language": "go",
                    "outcome": "judged_against",
                    "value": "upper-camel",
                },
                {
                    "rule": "goals-and-parts-line-up",
                    "family": "goals",
                    "declared_in": "standards.json",
                    "key": "goals",
                    "language": null,
                    "outcome": "judged_nothing",
                },
            ])
        );
    }
}
