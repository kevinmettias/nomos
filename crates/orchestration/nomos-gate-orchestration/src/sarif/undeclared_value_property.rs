//! [`UndeclaredValueProperty`], one value a run's rule read that the repository never declared, as
//! the run's own property bag carries it.

use nomos_check_orchestration::UndeclaredValues;
use nomos_contracts::RuleId;
use nomos_rules::{UndeclaredOutcome, UndeclaredValue};
use serde::Serialize;

/// One entry of [`super::run_properties::RunProperties`]'s `undeclaredValues`: a value one rule read
/// and the repository never declared, with what the rule did with it.
///
/// `OD-RULES-011` version 3 decision 1, at the grain a repository declares a value: the family,
/// where it is declared, the key, the language the rule read it for, and what the rule did. One
/// object per value rather than one per rule, so a consumer filters by family or key without
/// unnesting, and each carries the rule's id, which `tool.driver.rules` resolves.
///
/// Keys are camel case, as every other key in this log is; the outcome is a word in the snake case
/// every serialized vocabulary of this workspace uses, as a result's `bucket` is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UndeclaredValueProperty
{
    /// The rule that read the value.
    pub(crate) rule_id: String,
    /// The family: `limits`, `naming`, `scripting`, `goals`, `standards-corpus` or
    /// `requirement-trace`.
    pub(crate) family: &'static str,
    /// Where a repository declares it: the file, or for the requirement trace the directory.
    pub(crate) declared_in: &'static str,
    /// The key it is declared under.
    pub(crate) key: &'static str,
    /// The language the rule read it for, absent for a value read repository-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) language: Option<String>,
    /// What the rule did with it, in the word [`Outcome_Word`] spells.
    pub(crate) outcome: &'static str,
    /// The value the rule judged against in place of the one nobody declared, present only when
    /// the outcome is `judged_against`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
}

impl UndeclaredValueProperty
{
    /// Every value `undeclared` names, rule by rule in the order the run judged them, and each
    /// rule's values in the order it named them. Empty when no rule read a value nobody declared.
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
        let judged_against = match &value.outcome
        {
            UndeclaredOutcome::JudgedAgainst { value } => Some(value.clone()),
            UndeclaredOutcome::JudgedNothing | UndeclaredOutcome::ReportedUndeclared | UndeclaredOutcome::SpreadBoundNotJudged => None,
        };

        return Self {
            rule_id: rule.As_Str().to_owned(),
            family: value.family,
            declared_in: value.declared_in,
            key: value.key,
            language: value.language.clone(),
            outcome: Outcome_Word(&value.outcome),
            value: judged_against,
        };
    }
}

/// The word an entry's `outcome` carries for `outcome`.
///
/// Spelled here rather than derived onto [`UndeclaredOutcome`], for the reason
/// [`super::result_properties::Bucket_Word`] gives for a bucket: a SARIF property value is this
/// projection's own business. The match is exhaustive, so a fifth outcome does not compile until it
/// has a word.
const fn Outcome_Word(outcome: &UndeclaredOutcome) -> &'static str
{
    return match outcome
    {
        UndeclaredOutcome::JudgedAgainst { .. } => "judged_against",
        UndeclaredOutcome::JudgedNothing => "judged_nothing",
        UndeclaredOutcome::ReportedUndeclared => "reported_undeclared",
        UndeclaredOutcome::SpreadBoundNotJudged => "spread_bound_not_judged",
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At, Judged_Outcome_Beside};
    use crate::SarifLog;
    use nomos_check_orchestration::{CheckOutcome, Claim, Populations};
    use std::path::Path;

    /// A rule this build registers that read a limit no `nomos-limits.json` declared.
    const UNDECLARED_RULE: &str = nomos_rules::NESTING_DEPTH;

    /// A rule this build registers that read a limit the repository declared, and found one
    /// function over it.
    const DECLARED_RULE: &str = nomos_rules::PARAMETER_COUNT;

    /// A rule this build registers whose population was empty, so the log has a population for
    /// the list to sit beside.
    const EMPTY_RULE: &str = nomos_rules::A_SKIPPED_TEST_STATES_WHY;

    /// Where the one finding [`DECLARED_RULE`] reports sits.
    const FINDING_LOCATION: &str = "src/lib.rs:12";

    /// What [`UNDECLARED_RULE`] names: the depth limit nobody declared, judged against 3.
    fn Nesting_Depth_Undeclared() -> UndeclaredValue
    {
        return UndeclaredValue {
            family: "limits",
            declared_in: "nomos-limits.json",
            key: "nesting-depth-max",
            language: None,
            outcome: UndeclaredOutcome::JudgedAgainst { value: "3".to_owned() },
        };
    }

    /// [`UNDECLARED_RULE`] read a value nobody declared and [`DECLARED_RULE`] read one the
    /// repository declared, which is why it names nothing -- exactly what a run records for each.
    fn One_Undeclared_And_One_Declared() -> UndeclaredValues
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(RuleId::New(UNDECLARED_RULE), vec![Nesting_Depth_Undeclared()]);
        undeclared.Note(RuleId::New(DECLARED_RULE), Vec::new());

        return undeclared;
    }

    /// The same run with every value its rules read declared.
    fn Every_Value_Declared() -> UndeclaredValues
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(RuleId::New(DECLARED_RULE), Vec::new());

        return undeclared;
    }

    /// One rule whose population was empty.
    fn One_Empty_Population() -> Populations
    {
        let mut populations = Populations::New();
        populations.Note(RuleId::New(EMPTY_RULE), 0);

        return populations;
    }

    /// A check that judged its tree completely and found one finding from [`DECLARED_RULE`],
    /// over `populations`, naming `undeclared`. The claim is fixed here, so two outcomes built by
    /// this differ in what is reported beside the claim and in nothing else.
    fn Judged_Over(populations: Populations, undeclared: UndeclaredValues) -> CheckOutcome
    {
        return Judged_Outcome_Beside(vec![Finding_At(DECLARED_RULE, FINDING_LOCATION)], populations, undeclared, Claim::Complete);
    }

    /// A gate run that blocked on [`DECLARED_RULE`]'s finding, over `populations`, naming
    /// `undeclared`.
    fn Gate_Run_Over(populations: Populations, undeclared: UndeclaredValues) -> serde_json::Value
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At(DECLARED_RULE, FINDING_LOCATION));
        let mut result = Complete_Gate_Result(findings);
        result.check_outcome = Judged_Over(populations, undeclared);

        return Rendered(&SarifLog::Of_Gate_Run(&result));
    }

    /// A bare check run over `populations`, naming `undeclared`.
    fn Check_Run_Over(populations: Populations, undeclared: UndeclaredValues) -> serde_json::Value
    {
        return Rendered(&SarifLog::Of_Check_Run(Path::new("."), &Judged_Over(populations, undeclared)));
    }

    /// `log` as the JSON a consumer receives.
    fn Rendered(log: &SarifLog) -> serde_json::Value
    {
        return serde_json::to_value(log).expect("a derived Serialize over owned data has nothing to refuse");
    }

    /// What `reported` must say beside `unreported`, the same run naming no value: the undeclared
    /// value under the run's own `properties`, as an object whose rule id the driver resolves, the
    /// declared one nowhere, the population beside it as it was, and every other byte of the log --
    /// the results and the invocation that is this log's claim among them -- exactly what it was.
    fn Assert_Only_The_Undeclared_Value_Is_Named_Beside_The_Same_Log(reported: &serde_json::Value, unreported: &serde_json::Value)
    {
        let named = serde_json::json!([{
            "ruleId": UNDECLARED_RULE,
            "family": "limits",
            "declaredIn": "nomos-limits.json",
            "key": "nesting-depth-max",
            "outcome": "judged_against",
            "value": "3",
        }]);
        assert_eq!(reported.pointer("/runs/0/properties/undeclaredValues"), Some(&named), "{reported}");
        assert_eq!(reported.pointer("/runs/0/properties/emptyPopulations"), Some(&serde_json::json!([EMPTY_RULE])), "{reported}");

        let results = reported.pointer("/runs/0/results").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert_eq!(results.len(), 1, "a value nobody declared is not a result: {reported}");
        assert!(results.iter().all(|result| return result.pointer("/ruleId").and_then(serde_json::Value::as_str) != Some(UNDECLARED_RULE)), "{reported}");

        let listed = reported.pointer("/runs/0/tool/driver/rules").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert!(listed.iter().any(|rule| return rule.pointer("/id").and_then(serde_json::Value::as_str) == Some(UNDECLARED_RULE)), "{reported}");

        let mut without_the_list = reported.clone();
        if let Some(bag) = without_the_list.pointer_mut("/runs/0/properties").and_then(serde_json::Value::as_object_mut)
        {
            let _removed = bag.remove("undeclaredValues");
        }
        assert_eq!(&without_the_list, unreported, "the log apart from the list is the log with no value named");
    }

    /// `OD-RULES-011` version 3 decision 3, over the log `nomos gate run --sarif` writes: the rule
    /// that read a value nobody declared is named with it, the rule that read a declared value is
    /// not, and the verdict -- one blocking result, a successful invocation -- is the same either
    /// way.
    #[test]
    fn Test_A_Gate_Runs_Log_Should_Name_Only_The_Undeclared_Value_Beside_An_Unchanged_Verdict()
    {
        let reported = Gate_Run_Over(One_Empty_Population(), One_Undeclared_And_One_Declared());
        let unreported = Gate_Run_Over(One_Empty_Population(), UndeclaredValues::New());

        Assert_Only_The_Undeclared_Value_Is_Named_Beside_The_Same_Log(&reported, &unreported);
        assert_eq!(reported.pointer("/runs/0/results/0/level").and_then(serde_json::Value::as_str), Some("error"), "{reported}");
        assert_eq!(reported.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{reported}");
    }

    /// The same, over the log `nomos check --sarif` writes.
    #[test]
    fn Test_A_Check_Runs_Log_Should_Name_Only_The_Undeclared_Value_Beside_An_Unchanged_Claim()
    {
        let reported = Check_Run_Over(One_Empty_Population(), One_Undeclared_And_One_Declared());
        let unreported = Check_Run_Over(One_Empty_Population(), UndeclaredValues::New());

        Assert_Only_The_Undeclared_Value_Is_Named_Beside_The_Same_Log(&reported, &unreported);
        assert_eq!(reported.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{reported}");
    }

    /// A run in which every value its rules read was declared writes the log it wrote before the
    /// list existed, byte for byte -- with no bag at all when no population was empty either.
    #[test]
    fn Test_A_Log_With_Every_Value_Declared_Should_Say_Nothing_New()
    {
        assert_eq!(Gate_Run_Over(Populations::New(), Every_Value_Declared()), Gate_Run_Over(Populations::New(), UndeclaredValues::New()));
        assert_eq!(Check_Run_Over(Populations::New(), Every_Value_Declared()), Check_Run_Over(Populations::New(), UndeclaredValues::New()));
        assert!(Check_Run_Over(Populations::New(), Every_Value_Declared()).pointer("/runs/0/properties").is_none());
    }

    /// A value read for one language carries it, and an outcome that judged against nothing
    /// carries no value: the two optional keys are absent exactly when there is nothing to say.
    #[test]
    fn Test_An_Entry_Should_Carry_Its_Language_And_Only_A_Value_It_Was_Judged_Against()
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

        let rendered = serde_json::to_value(UndeclaredValueProperty::Every(&undeclared)).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(
            rendered,
            serde_json::json!([
                {
                    "ruleId": "types-use-upper-camel-case-lower-camel-case",
                    "family": "naming",
                    "declaredIn": "standards.json",
                    "key": "type.exported",
                    "language": "go",
                    "outcome": "judged_against",
                    "value": "upper-camel",
                },
                {
                    "ruleId": "goals-and-parts-line-up",
                    "family": "goals",
                    "declaredIn": "standards.json",
                    "key": "goals",
                    "outcome": "judged_nothing",
                },
            ])
        );
    }
}
