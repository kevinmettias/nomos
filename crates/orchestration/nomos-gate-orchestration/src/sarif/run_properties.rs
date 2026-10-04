//! [`RunProperties`], the rules a run judged over an empty population, carried beside the run's
//! invocation and never among its results.

use nomos_check_orchestration::CheckOutcome;
use serde::Serialize;

/// SARIF 2.1.0 §3.8's property bag, as §3.14's `run` carries it.
///
/// `OD-ANALYSIS-012` version 3: every rendering of a verdict names the rules whose population
/// was empty, beside the claim and not inside it, and the owner ratified that on the condition
/// that it is in the default output. A rule that judged nothing produces no result, exactly as a
/// rule that judged real subjects and found them clean does, so a log that does not say which
/// is which reads every such rule as a clean judgment.
///
/// # Why a run-level property and not the two other places SARIF admits
///
/// - **Not a `result`.** An empty population is not a finding: it has no subject, no location
///   and no level, and a consumer that counts or lists results would count and list it as one.
///   A code-scanning view would raise an alert for a rule that looked at nothing.
/// - **Not one of the invocation's `toolExecutionNotifications`.** Those are what make
///   [`super::sarif_invocation::SarifInvocation`]'s `executionSuccessful` false, which is this
///   log's rendering of the claim, and the record decided that no empty population moves the
///   claim. A notification beside a successful execution would break that invocation's own
///   invariant, and one that flipped it would be the flip the owner ruled out.
///
/// A property of the run is neither. It sits beside `invocations`, it is read by key rather
/// than by scanning prose, and every id in it is a rule id a consumer can resolve against
/// `tool.driver.rules` to reach the record the rule cites.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunProperties
{
    /// Every rule the run selected whose population was empty, by id, in the order the run
    /// judged them -- `nomos_check_orchestration::Populations::Empty`, read and not recounted.
    pub(crate) empty_populations: Vec<String>,
}

impl RunProperties
{
    /// The bag for a run whose check produced `outcome`.
    ///
    /// `None` when every selected rule judged something, so a log over such a run is the log it
    /// was before this bag existed and says nothing new. `None` as well for a run that judged
    /// nothing at all: no rule was handed anything, and the invocation already says why.
    pub(crate) fn Of(outcome: &CheckOutcome) -> Option<Self>
    {
        let CheckOutcome::Judged { populations, .. } = outcome
        else
        {
            return None;
        };

        let empty_populations: Vec<String> = populations.Empty().into_iter().map(|rule| return rule.As_Str().to_owned()).collect();
        if empty_populations.is_empty()
        {
            return None;
        }

        return Some(Self { empty_populations });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At, Judged_Outcome_Of};
    use crate::SarifLog;
    use nomos_check_orchestration::{Claim, Populations};
    use nomos_contracts::RuleId;
    use std::path::Path;

    /// A rule this build registers whose population is Go sources only, so the log's driver
    /// lists it and the name the bag carries can be followed to its descriptor.
    const EMPTY_RULE: &str = nomos_rules::A_SKIPPED_TEST_STATES_WHY;

    /// A rule this build registers that judged a real source and found something in it.
    const JUDGED_RULE: &str = nomos_rules::COMPLETENESS_MIRROR;

    /// How many sources [`JUDGED_RULE`] was judged over.
    const JUDGED_SOURCES: usize = 3;

    /// Where the one finding [`JUDGED_RULE`] reports sits.
    const FINDING_LOCATION: &str = "src/lib.rs:12";

    /// One rule that judged nothing and one that judged [`JUDGED_SOURCES`] sources.
    fn One_Empty_And_One_Judged() -> Populations
    {
        let mut populations = Populations::New();
        populations.Note(RuleId::New(EMPTY_RULE), 0);
        populations.Note(RuleId::New(JUDGED_RULE), JUDGED_SOURCES);

        return populations;
    }

    /// The same run with only [`JUDGED_RULE`] reported, which judged something.
    fn Only_The_Judged_One() -> Populations
    {
        let mut populations = Populations::New();
        populations.Note(RuleId::New(JUDGED_RULE), JUDGED_SOURCES);

        return populations;
    }

    /// A check that judged its tree completely and found one finding from [`JUDGED_RULE`], over
    /// `populations`. The claim is fixed here, so two outcomes built by this differ in their
    /// populations and in nothing else.
    fn Judged_Over(populations: Populations) -> CheckOutcome
    {
        return Judged_Outcome_Of(vec![Finding_At(JUDGED_RULE, FINDING_LOCATION)], populations, Claim::Complete);
    }

    /// A gate run that blocked on [`JUDGED_RULE`]'s finding, over `populations`.
    fn Gate_Run_Over(populations: Populations) -> serde_json::Value
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At(JUDGED_RULE, FINDING_LOCATION));
        let mut result = Complete_Gate_Result(findings);
        result.check_outcome = Judged_Over(populations);

        return Rendered(&SarifLog::Of_Gate_Run(&result));
    }

    /// A bare check run over `populations`.
    fn Check_Run_Over(populations: Populations) -> serde_json::Value
    {
        return Rendered(&SarifLog::Of_Check_Run(Path::new("."), &Judged_Over(populations)));
    }

    /// `log` as the JSON a consumer receives.
    fn Rendered(log: &SarifLog) -> serde_json::Value
    {
        return serde_json::to_value(log).expect("a derived Serialize over owned data has nothing to refuse");
    }

    /// What `reported` must say beside `unreported`, the same run with no population reported:
    /// the empty rule named under the run's own `properties` and nowhere else, as an id the
    /// driver resolves, and every other byte of the log -- the results and the invocation that
    /// is this log's claim among them -- exactly what it was.
    fn Assert_Only_The_Empty_Rule_Is_Named_Beside_The_Same_Log(reported: &serde_json::Value, unreported: &serde_json::Value)
    {
        assert_eq!(reported.pointer("/runs/0/properties"), Some(&serde_json::json!({ "emptyPopulations": [EMPTY_RULE] })), "{reported}");

        let results = reported.pointer("/runs/0/results").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert_eq!(results.len(), 1, "an empty population is not a result: {reported}");
        assert!(results.iter().all(|result| return result.pointer("/ruleId").and_then(serde_json::Value::as_str) != Some(EMPTY_RULE)), "{reported}");

        let listed = reported.pointer("/runs/0/tool/driver/rules").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert!(listed.iter().any(|rule| return rule.pointer("/id").and_then(serde_json::Value::as_str) == Some(EMPTY_RULE)), "{reported}");

        let mut without_the_bag = reported.clone();
        if let Some(run) = without_the_bag.pointer_mut("/runs/0").and_then(serde_json::Value::as_object_mut)
        {
            let _removed = run.remove("properties");
        }
        assert_eq!(&without_the_bag, unreported, "the log apart from the bag is the log with no population reported");
    }

    /// `OD-ANALYSIS-012` version 3, over the log `nomos gate run --sarif` writes: the rule that
    /// judged nothing is named, the rule that judged three sources is not, and the verdict --
    /// one blocking result, a successful invocation -- is the same either way.
    #[test]
    fn Test_A_Gate_Runs_Log_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Verdict()
    {
        let reported = Gate_Run_Over(One_Empty_And_One_Judged());
        let unreported = Gate_Run_Over(Populations::New());

        Assert_Only_The_Empty_Rule_Is_Named_Beside_The_Same_Log(&reported, &unreported);
        assert_eq!(reported.pointer("/runs/0/results/0/level").and_then(serde_json::Value::as_str), Some("error"), "{reported}");
        assert_eq!(reported.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{reported}");
    }

    /// The same, over the log `nomos check --sarif` writes.
    #[test]
    fn Test_A_Check_Runs_Log_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim()
    {
        let reported = Check_Run_Over(One_Empty_And_One_Judged());
        let unreported = Check_Run_Over(Populations::New());

        Assert_Only_The_Empty_Rule_Is_Named_Beside_The_Same_Log(&reported, &unreported);
        assert_eq!(reported.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{reported}");
    }

    /// A run in which every selected rule judged something writes the log it wrote before the
    /// bag existed, byte for byte.
    #[test]
    fn Test_A_Log_With_No_Empty_Population_Should_Say_Nothing_New()
    {
        assert_eq!(Gate_Run_Over(Only_The_Judged_One()), Gate_Run_Over(Populations::New()));
        assert_eq!(Check_Run_Over(Only_The_Judged_One()), Check_Run_Over(Populations::New()));
    }

    /// A run that judged nothing has no population to report, and its invocation is what says
    /// why.
    #[test]
    fn Test_Of_Should_Be_None_For_A_Run_That_Judged_Nothing()
    {
        assert_eq!(RunProperties::Of(&CheckOutcome::NoSource), None);
    }
}
