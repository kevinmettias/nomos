use crate::Rule;
use crate::Violation;
use crate::RuleOutcome;
use crate::RuleResult;
// check-dependency-placement reports this crate's `nomos_spec_model` edge as this file's
// alone. ContentHash is the model's own name for what a preserved block hashes to, and a
// validation run that spelled its own would be a second authority on when two blocks are
// the same text.
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
    "NSV-PRESERVE-004",
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
    pub fn Is_Passed(&self) -> bool
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
// The ruleset is a run-time list rather than a fixed set of types, because the reconciliation
// below exists so the registered rules may disagree with `DECLARED_RULES` and be reported
// saying how. A generic parameter would make the ruleset part of this function's signature,
// and then "a declared rule nobody registered" could not be a value it is handed.
pub fn Validate_Rules(store: &SpecificationStore, rules: &[Box<dyn Rule>]) -> ValidationRun
{
    let registered: Vec<&'static str> = rules.iter().map(|rule| rule.Id()).collect();
    let Reconciliation {
        unregistered,
        undeclared,
    } = Reconciled_Registry(&registered);

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
fn Reconciled_Registry(registered: &[&'static str]) -> Reconciliation
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

    return Reconciliation {
        unregistered,
        undeclared,
    };
}

/// The two directions of a registry-against-manifest disagreement.
///
/// Named rather than a pair. Both members are `Vec<String>` and the compiler cannot tell
/// them apart, so a call site that swapped them would report every declared-but-absent
/// rule as registered-but-undeclared and still build.
struct Reconciliation
{
    unregistered: Vec<String>,
    undeclared: Vec<String>,
}

/// Identifies the ruleset that produced a result.
///
/// A snapshot records this. Without it, "validated clean" does not say clean *against
/// what*, and a run with three rules disabled is indistinguishable from a full one.
#[must_use]
pub(crate) fn Ruleset_Hash(ids: &[&str]) -> ContentHash
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

    /// An arbitrary non-zero subject count for a fixture rule that must read as satisfied
    /// but not vacuous. Its exact value is not significant, only that it is not zero.
    const SOME_SUBJECTS_CHECKED: u32 = 10;

    #[test]
    fn Test_Is_Passed_Should_Report_True_For_A_Complete_Satisfied_Run()
    {
        let rules = All_Declared(&RuleOutcome::Satisfied { checked: SOME_SUBJECTS_CHECKED });

        let run = Validate_Rules(&Store(), &rules);

        assert!(run.Is_Passed(), "{}", run.Summary());
        assert!(run.Vacuous_Rules().is_empty());
    }

    /// The structural fix for "no validator ran". A declared rule with no implementation
    /// must fail the run, not be quietly absent from it.
    #[test]
    fn Test_Validate_Rules_Should_Fail_The_Run_When_A_Declared_Rule_Is_Not_Registered()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: SOME_SUBJECTS_CHECKED });
        rules.pop();

        let run = Validate_Rules(&Store(), &rules);

        assert!(!run.Is_Passed(), "a missing rule must not read as a clean run");
        assert_eq!(run.unregistered.len(), 1);
    }

    /// An error is a failure, never a skip.
    #[test]
    fn Test_Errors_Should_Report_One_Entry_For_One_Errored_Rule()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: SOME_SUBJECTS_CHECKED });
        rules.pop();
        let errored = Fake(
            "NSV-PRESERVE-006",
            RuleOutcome::Errored("input did not resolve".to_owned()),
        );
        rules.push(Box::new(errored));

        let run = Validate_Rules(&Store(), &rules);

        assert!(!run.Is_Passed());
        assert_eq!(run.Errors().len(), 1);
        assert!(run.unregistered.is_empty(), "the rule ran; it failed");
    }

    #[test]
    fn Test_A_Rule_Nobody_Declared_Should_Fail_The_Run()
    {
        let mut rules = All_Declared(&RuleOutcome::Satisfied { checked: SOME_SUBJECTS_CHECKED });
        let undeclared = Fake("NSV-INVENTED-001", RuleOutcome::Satisfied { checked: 1 });
        rules.push(Box::new(undeclared));

        let run = Validate_Rules(&Store(), &rules);

        assert!(!run.Is_Passed(), "the manifest must describe what ran");
        assert_eq!(run.undeclared, vec!["NSV-INVENTED-001".to_owned()]);
    }

    /// A run of no rules at all is not a pass.
    #[test]
    fn Test_An_Empty_Ruleset_Should_Not_Pass()
    {
        let run = Validate_Rules(&Store(), &[]);

        assert!(!run.Is_Passed());
    }

    /// "0 violations over 0 subjects" must be distinguishable from a real clean result.
    #[test]
    fn Test_Vacuous_Rules_Should_List_Every_Rule_When_All_Examined_Nothing()
    {
        let rules = All_Declared(&RuleOutcome::Satisfied { checked: 0 });

        let run = Validate_Rules(&Store(), &rules);

        assert!(run.Is_Passed(), "an empty store legitimately has nothing to check");
        assert_eq!(run.Vacuous_Rules().len(), DECLARED_RULES.len());
    }

    /// `Summary` renders the four counts a run is judged by: how many rules ran, over how
    /// many subjects, and how many violations, errors, unregistered and undeclared rules
    /// resulted. Built by hand rather than through `Registered()`'s real rules, so every
    /// count in the expected string is one this test chose and can account for.
    #[test]
    fn Test_Summary_Should_Combine_Rule_Error_And_Reconciliation_Counts_Into_One_Line()
    {
        let rules: Vec<Box<dyn Rule>> = vec![
            Box::new(Fake(
                *DECLARED_RULES.first().expect("DECLARED_RULES lists at least two rules"),
                RuleOutcome::Satisfied { checked: 5 },
            )),
            Box::new(Fake(
                *DECLARED_RULES.get(1).expect("DECLARED_RULES lists at least two rules"),
                RuleOutcome::Violated(vec![Violation {
                    subject: "s".to_owned(),
                    detail: "d".to_owned(),
                }]),
            )),
            Box::new(Fake("NSV-INVENTED-001", RuleOutcome::Errored("boom".to_owned()))),
        ];

        let run = Validate_Rules(&Store(), &rules);

        assert_eq!(
            run.Summary(),
            format!(
                "3 rule(s) ran over 5 subject(s): 1 violation(s), 1 error(s), {} unregistered, 1 undeclared",
                DECLARED_RULES.len() - 2
            )
        );
    }

    #[test]
    fn Test_Violations_Should_Fail_The_Run_And_Be_Listed()
    {
        let rules = All_Declared(&RuleOutcome::Violated(vec![Violation {
            subject: "AGT-001".to_owned(),
            detail: "no preserved lineage".to_owned(),
        }]));

        let run = Validate_Rules(&Store(), &rules);

        assert!(!run.Is_Passed());
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

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory().expect("opens");
    }

    // Erased to the same `Box<dyn Rule>` the production caller passes, so these tests exercise
    // `Validate`'s real parameter. A helper returning `Vec<Fake>` would prove nothing about the
    // signature `Registered()` feeds.
    fn All_Declared(outcome: &RuleOutcome) -> Vec<Box<dyn Rule>>
    {
        return DECLARED_RULES
            .iter()
            .map(|id| {
                let rule = Fake(id, outcome.clone());

                // The cast is load-bearing: without it the closure returns `Box<Fake>` and
                // `collect` builds a `Vec<Box<Fake>>`, which is not the declared return type.
                // Erasure happens here rather than at the `return` because `collect` is what
                // has to be told which element type it is accumulating.
                return Box::new(rule) as Box<dyn Rule>;
            })
            .collect();
    }
}
