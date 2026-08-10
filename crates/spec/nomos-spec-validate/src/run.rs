use crate::rule::Rule;
use crate::violation::Violation;
use crate::rule_outcome::RuleOutcome;
use crate::rule_result::RuleResult;
use nomos_spec_model::ContentHash;
use nomos_spec_store::SpecificationStore;

/// Every rule this build claims to enforce.
///
/// Compared against the registered set at run time. A declared rule with no
/// implementation makes the run error, which is what stops "the validator does not exist"
/// from looking like "the validator found nothing".
///
/// Mirrored by `Test_The_Registry_Should_Match_The_Manifest`, which reconciles this list
/// against the identifiers the registered rule objects return, in both directions. Those
/// identifiers are read off the `impl Rule` blocks that run rather than copied from here,
/// so the comparison can fail.
///
/// It reaches the registry and not the implementations: a rule implemented and never
/// registered is in neither list and outside this claim. `OD-COMPLETENESS-002` states that
/// boundary rather than leaving the claim to be read for more than it is.
pub const DECLARED_RULES: &[&str] = &[
    "NSV-PRESERVE-001",
    "NSV-PRESERVE-002",
    "NSV-PRESERVE-003",
    "NSV-PRESERVE-006",
];

#[derive(Debug)]
pub struct ValidationRun
{
    pub results: Vec<RuleResult>,
    /// Declared but not registered. Any entry here fails the run.
    pub unregistered: Vec<String>,
    /// Registered but not declared. Also a failure: an undeclared rule means the
    /// manifest does not describe what ran.
    pub undeclared: Vec<String>,
    pub ruleset_hash: ContentHash,
}

impl ValidationRun
{
    /// Whether the run may be relied on.
    ///
    /// Fails closed. Every declared rule must have run to completion, and an internal
    /// error is a failure rather than a skip: **the absence of a validator has to be
    /// indistinguishable from a failing one.** v15.0 is what the other choice looks
    /// like — the checks existed, none ran, and the publication was clean.
    #[must_use]
    pub fn Passed(&self) -> bool
    {
        return self.unregistered.is_empty()
            && self.undeclared.is_empty()
            && !self.results.is_empty()
            && self
                .results
                .iter()
                .all(|result| matches!(result.outcome, RuleOutcome::Satisfied { .. }));
    }

    /// Rules that concluded nothing was wrong having examined nothing.
    ///
    /// Not automatically a failure — an empty store legitimately has nothing to check —
    /// but it must be visible, because "0 violations over 0 subjects" and "0 violations
    /// over 2533 subjects" print the same and mean opposite things.
    #[must_use]
    pub fn Vacuous_Rules(&self) -> Vec<&str>
    {
        return self
            .results
            .iter()
            .filter(|result| matches!(result.outcome, RuleOutcome::Satisfied { checked: 0 }))
            .map(|result| result.id.as_str())
            .collect();
    }

    #[must_use]
    pub fn Violations(&self) -> Vec<&Violation>
    {
        return self
            .results
            .iter()
            .filter_map(|result| match &result.outcome
            {
                RuleOutcome::Violated(violations) => Some(violations),
                RuleOutcome::Satisfied { .. } | RuleOutcome::Errored(_) => None,
            })
            .flatten()
            .collect();
    }

    #[must_use]
    pub fn Errors(&self) -> Vec<(&str, &str)>
    {
        return self
            .results
            .iter()
            .filter_map(|result| match &result.outcome
            {
                RuleOutcome::Errored(cause) => Some((result.id.as_str(), cause.as_str())),
                RuleOutcome::Satisfied { .. } | RuleOutcome::Violated(_) => None,
            })
            .collect();
    }

    #[must_use]
    pub fn Summary(&self) -> String
    {
        let checked: u32 = self
            .results
            .iter()
            .map(|result| match result.outcome
            {
                RuleOutcome::Satisfied { checked } => checked,
                RuleOutcome::Violated(_) | RuleOutcome::Errored(_) => 0,
            })
            .sum();

        return format!(
            "{} rule(s) ran over {checked} subject(s): {} violation(s), {} error(s), \
             {} unregistered, {} undeclared",
            self.results.len(),
            self.Violations().len(),
            self.Errors().len(),
            self.unregistered.len(),
            self.undeclared.len()
        );
    }
}

/// Runs every registered rule and reconciles the registry against the manifest.
#[must_use]
pub fn Validate(store: &SpecificationStore, rules: &[Box<dyn Rule>]) -> ValidationRun
{
    let registered: Vec<&'static str> = rules.iter().map(|rule| rule.Id()).collect();
    let (unregistered, undeclared) = Reconciled(&registered);

    let results = rules
        .iter()
        .map(|rule| RuleResult {
            id: rule.Id().to_owned(),
            outcome: rule.Evaluate(store),
        })
        .collect();

    return ValidationRun {
        results,
        unregistered,
        undeclared,
        ruleset_hash: Ruleset_Hash(&registered),
    };
}

/// The registry against the manifest, in both directions.
///
/// A rule declared and not registered would be reported nowhere, and one registered and not
/// declared would run without the manifest naming it. Checking one direction only leaves
/// half of a disagreement invisible, which is the failure this reconciliation exists for.
fn Reconciled(registered: &[&'static str]) -> (Vec<String>, Vec<String>)
{
    let unregistered: Vec<String> = DECLARED_RULES
        .iter()
        .filter(|declared| !registered.contains(*declared))
        .map(|declared| (*declared).to_owned())
        .collect();

    let undeclared: Vec<String> = registered
        .iter()
        .filter(|id| !DECLARED_RULES.contains(*id))
        .map(|id| (*id).to_owned())
        .collect();

    return (unregistered, undeclared);
}

/// Identifies the ruleset that produced a result.
///
/// A snapshot records this. Without it, "validated clean" does not say clean *against
/// what*, and a run with three rules disabled is indistinguishable from a full one.
#[must_use]
pub fn Ruleset_Hash(ids: &[&str]) -> ContentHash
{
    let mut sorted: Vec<&str> = ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    return ContentHash::Of(&sorted.join("\n"));
}

#[cfg(test)]
mod tests
{
    use super::*;

    struct Fake(&'static str, RuleOutcome);

    impl Rule for Fake
    {
        fn Id(&self) -> &'static str
        {
            return self.0;
        }
        fn Describe(&self) -> &'static str
        {
            return "a test rule";
        }
        fn Evaluate(&self, _store: &SpecificationStore) -> RuleOutcome
        {
            return self.1.clone();
        }
    }

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory().expect("opens");
    }

    fn All_Declared(outcome: &RuleOutcome) -> Vec<Box<dyn Rule>>
    {
        return DECLARED_RULES
            .iter()
            .map(|id| {
                let rule = Fake(id, outcome.clone());

                return Box::new(rule) as Box<dyn Rule>;
            })
            .collect();
    }

    #[test]
    fn Test_A_Complete_Satisfied_Run_Should_Pass()
    {
        let rules = All_Declared(&RuleOutcome::Satisfied { checked: 10 });

        let run = Validate(&Store(), &rules);

        assert!(run.Passed(), "{}", run.Summary());
        assert!(run.Vacuous_Rules().is_empty());
    }

    /// The structural fix for "no validator ran". A declared rule with no implementation
    /// must fail the run, not be quietly absent from it.
    #[test]
    fn Test_A_Declared_But_Unregistered_Rule_Should_Fail_The_Run()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: 10 });
        rules.pop();

        let run = Validate(&Store(), &rules);

        assert!(!run.Passed(), "a missing rule must not read as a clean run");
        assert_eq!(run.unregistered.len(), 1);
    }

    /// An error is a failure, never a skip.
    #[test]
    fn Test_An_Errored_Rule_Should_Fail_The_Run()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: 10 });
        rules.pop();
        let errored = Fake(
            "NSV-PRESERVE-006",
            RuleOutcome::Errored("input did not resolve".to_owned()),
        );
        rules.push(Box::new(errored));

        let run = Validate(&Store(), &rules);

        assert!(!run.Passed());
        assert_eq!(run.Errors().len(), 1);
        assert!(run.unregistered.is_empty(), "the rule ran; it failed");
    }

    #[test]
    fn Test_A_Rule_Nobody_Declared_Should_Fail_The_Run()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: 10 });
        let undeclared = Fake("NSV-INVENTED-001", RuleOutcome::Satisfied { checked: 1 });
        rules.push(Box::new(undeclared));

        let run = Validate(&Store(), &rules);

        assert!(!run.Passed(), "the manifest must describe what ran");
        assert_eq!(run.undeclared, vec!["NSV-INVENTED-001".to_owned()]);
    }

    /// A run of no rules at all is not a pass.
    #[test]
    fn Test_An_Empty_Ruleset_Should_Not_Pass()
    {
        let run = Validate(&Store(), &[]);

        assert!(!run.Passed());
    }

    /// "0 violations over 0 subjects" must be distinguishable from a real clean result.
    #[test]
    fn Test_A_Vacuous_Rule_Should_Be_Visible()
    {
        let rules = All_Declared(&RuleOutcome::Satisfied { checked: 0 });

        let run = Validate(&Store(), &rules);

        assert!(run.Passed(), "an empty store legitimately has nothing to check");
        assert_eq!(run.Vacuous_Rules().len(), DECLARED_RULES.len());
    }

    #[test]
    fn Test_Violations_Should_Fail_The_Run_And_Be_Listed()
    {
        let rules = All_Declared(&RuleOutcome::Violated(vec![Violation {
            subject: "AGT-001".to_owned(),
            detail: "no preserved lineage".to_owned(),
        }]));

        let run = Validate(&Store(), &rules);

        assert!(!run.Passed());
        assert_eq!(run.Violations().len(), DECLARED_RULES.len());
    }

    /// The hash identifies the ruleset, so it must not depend on registration order and
    /// must change when a rule is removed.
    #[test]
    fn Test_The_Ruleset_Hash_Should_Be_Order_Independent_And_Sensitive()
    {
        let forward = Ruleset_Hash(&["NSV-PRESERVE-001", "NSV-PRESERVE-002"]);
        let reverse = Ruleset_Hash(&["NSV-PRESERVE-002", "NSV-PRESERVE-001"]);
        let fewer = Ruleset_Hash(&["NSV-PRESERVE-001"]);

        assert_eq!(forward, reverse);
        assert_ne!(forward, fewer, "a narrowed ruleset must not hash the same");
    }
}
