//! `nomos-gate.json`'s two phase families, as written: the ordered stages a run judges and
//! the approvals that let one of them pass anyway.
//!
//! `crate::GatePhase`, `crate::PhaseThreshold` and `crate::PhaseApproval` had execution
//! semantics and no author -- `OD-ROADMAP-003` measured that gap directly ("a phase can be
//! constructed and cannot be authored") once it found `OD-ROADMAP-002`'s pause on new gate
//! policy increments lapsed. This module is the authoring side, in the one reader
//! `OD-GATE-011` permits rather than in a second encoding beside it.
//!
//! # Why a threshold is a key on its phase and not a family of its own
//!
//! `crate::GatePhase` owns its own `threshold`, so a phase and the number it tolerates are
//! one declaration. A top-level `thresholds` table keyed by phase name would be a second
//! encoding of a decision this file already carries -- two places to write it, two ways for
//! them to disagree, and no rule in the types for which one wins. That is the defect class
//! `OD-GATE-011` names, reached from inside one file rather than across two readers.
//!
//! # Why an absent threshold is the strict reading
//!
//! `OD-GATE-029` decided that an absent key still means unset, and the meaning of unset is
//! settled per field rather than globally: an absent `phases` is "no phase policy," exactly
//! the state every existing caller and CI's own `gate run --root .` are already in. Inside a
//! phase, absence resolves to [`crate::PhaseThreshold::AnyBlockingFinding`] -- the strictest
//! of the two variants, and identical to what the run already does with those rules when no
//! phase is declared at all.
//!
//! Deliberately the opposite direction from `DeclaredDebt`'s own `accepted_occurrence_count`,
//! whose absence is the *lenient* reading. `OD-GATE-030` chose that for a migration: entries
//! authored before the count existed would otherwise start blocking builds over debt a
//! repository did adopt. Nothing predates `threshold`, so no entry is owed that compatibility,
//! and `OD-GATE-029`'s own principle applies instead -- a tolerance should be a decision
//! somebody wrote, never a consequence of a field nobody wrote.
//!
//! # A declared table, never a condition
//!
//! `OD-ROADMAP-003`'s surviving constraint: a new gate policy family is a declared constant
//! the gate policy reader reads off the file, and never a condition that consults store
//! state, cost or prior materialization. A phase here applies because the file names its
//! rules. It may not apply *because the last run failed*, *because the tolerated count is
//! cheaper to accept than to fix*, or because of anything else no row of this table can
//! state. The moment an entry needs an input the row does not carry, it has stopped being a
//! declared fact, and `OD-RULES-009` is where that has to be argued.
//!
//! # What a phase reaches that a suppression does not
//!
//! A phase addresses by `rule`, the coarse addressing `DeclaredCalibration` already uses, so
//! unlike `DeclaredSuppression` and `DeclaredDebt` it reaches every rule a file can name.
//! The path-to-subject limit the parent module documents -- `OD-GATE-032`, and the measured
//! fact that `completeness-mirror` addresses the mirrored item rather than the file -- is a
//! limit on path-authored entries and not on these two.

use nomos_contracts::RuleId;
use serde::Deserialize;
use std::collections::BTreeSet;

use crate::{GatePhase, PhaseApproval, PhaseThreshold};

/// One ordered stage, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclaredPhase
{
    name: String,
    rules: Vec<String>,
    /// How many blocking findings this stage tolerates before it fails.
    ///
    /// Optional, and absence is the strict reading rather than a tolerance -- see this
    /// module's own doc for why that is the opposite direction from `DeclaredDebt`'s count.
    #[serde(default)]
    threshold: Option<DeclaredThreshold>,
}

impl DeclaredPhase
{
    /// What is wrong with this entry, if anything -- the phase counterpart to
    /// `DeclaredSuppression::Problem`, refused at the same point for the same reason.
    fn Problem(&self) -> Option<String>
    {
        if self.rules.is_empty()
        {
            return Some(format!(
                "the phase '{}' names no rules. A phase judges only the findings its own rules admit, so one naming none judges nothing and can only report a stage that was never run: give it the rules this stage covers, or remove it.",
                self.name
            ));
        }

        if matches!(self.threshold, Some(DeclaredThreshold::MaxBlockingFindings(0)))
        {
            return Some(format!(
                "the phase '{}' declares a threshold of zero blocking findings, which is 'any-blocking-finding' under another name and is already what a phase naming no threshold at all means: write that spelling, or the number of findings this stage tolerates.",
                self.name
            ));
        }

        return None;
    }

    /// This entry as the domain type, with an absent threshold resolved strictly.
    fn Resolved(self) -> GatePhase
    {
        return GatePhase {
            name: self.name,
            rules: self.rules.into_iter().map(|rule| return RuleId::New(&rule)).collect(),
            threshold: self.threshold.map_or(PhaseThreshold::AnyBlockingFinding, DeclaredThreshold::Resolved),
        };
    }
}

/// A phase's own threshold, as written: `"any-blocking-finding"`, or
/// `{ "max-blocking-findings": <count> }`.
///
/// `kebab-case` for the reason `DeclaredDisposition` is spelled that way -- this is a
/// hand-authored file and not a serialization of the enum, and the mapping is stated once,
/// here. The count rides in the variant directly rather than under a second `max` key, so an
/// author writes the number where the name already says what it counts.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclaredThreshold
{
    AnyBlockingFinding,
    MaxBlockingFindings(usize),
}

impl DeclaredThreshold
{
    /// This spelling as the domain variant.
    const fn Resolved(self) -> PhaseThreshold
    {
        return match self
        {
            Self::AnyBlockingFinding => PhaseThreshold::AnyBlockingFinding,
            Self::MaxBlockingFindings(max) => PhaseThreshold::MaxBlockingFindings { max },
        };
    }
}

/// One approval of a phase that would otherwise fail, as written.
///
/// Names no rule and no path: `crate::PhaseApproval` addresses a whole phase by name, and
/// giving this entry either would suggest a narrowing it does not perform.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclaredApproval
{
    phase: String,
    rationale: String,
}

impl DeclaredApproval
{
    /// What is wrong with this entry against the `phases` declared beside it, if anything.
    fn Problem(&self, phases: &[DeclaredPhase]) -> Option<String>
    {
        if phases.iter().any(|phase| return phase.name == self.phase)
        {
            return None;
        }

        return Some(format!(
            "the approval naming phase '{}' matches no phase this file declares, and this file declares {}. An approval is matched to the phase it covers by name, so one naming a phase that is not there is read by nothing at all: name a declared phase, or remove the approval.",
            self.phase,
            Declared_Names(phases)
        ));
    }

    /// This entry as the domain type.
    fn Resolved(self) -> PhaseApproval
    {
        return PhaseApproval { phase: self.phase, rationale: self.rationale };
    }
}

/// What is wrong with the phase families in one file, if anything.
///
/// One entry point rather than a `Problem` per list at the call site, because the third
/// refusal below is about the two lists together and has nowhere else to live: an approval is
/// only answerable against the phases declared beside it.
pub(super) fn Phase_Problem(phases: &[DeclaredPhase], approvals: &[DeclaredApproval]) -> Option<String>
{
    return phases
        .iter()
        .find_map(DeclaredPhase::Problem)
        .or_else(|| return Repeated_Phase_Name(phases))
        .or_else(|| return approvals.iter().find_map(|approval| return approval.Problem(phases)));
}

/// `phases` as the domain types a run judges with, in declared order.
///
/// Order is the declaration: `crate::Evaluated_Phases` judges phases in the order given and
/// stops at the first that fails unapproved, so this must never sort.
pub(super) fn Resolved_Phases(phases: Vec<DeclaredPhase>) -> Vec<GatePhase>
{
    return phases.into_iter().map(DeclaredPhase::Resolved).collect();
}

/// `approvals` as the domain types a run judges with.
pub(super) fn Resolved_Approvals(approvals: Vec<DeclaredApproval>) -> Vec<PhaseApproval>
{
    return approvals.into_iter().map(DeclaredApproval::Resolved).collect();
}

/// The first name two phases share, as a refusal.
///
/// An approval names the phase it covers and `crate::PhaseOutcome::name` reports a stage
/// under that name, so two stages sharing one cannot be told apart by either -- an approval
/// written for the first silently approves the second as well.
fn Repeated_Phase_Name(phases: &[DeclaredPhase]) -> Option<String>
{
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for phase in phases
    {
        if !seen.insert(phase.name.as_str())
        {
            return Some(format!(
                "two phases are named '{}'. An approval names the phase it covers and an outcome is reported under that name, so two stages sharing one cannot be told apart by either: name each phase once.",
                phase.name
            ));
        }
    }

    return None;
}

/// The phase names a file declares, for a refusal that has to say what could have been named.
fn Declared_Names(phases: &[DeclaredPhase]) -> String
{
    if phases.is_empty()
    {
        return "no phases at all".to_owned();
    }

    return phases.iter().map(|phase| return format!("'{}'", phase.name)).collect::<Vec<String>>().join(", ");
}

#[cfg(test)]
mod tests
{
    use super::{DeclaredApproval, DeclaredPhase, Phase_Problem, Resolved_Approvals, Resolved_Phases};
    use crate::PhaseThreshold;
    use nomos_contracts::RuleId;

    /// The count the tolerant fixture below declares, and the one its resolution must carry.
    const TOLERATED_FINDINGS: usize = 3;

    /// One phase naming one rule and the number of findings it tolerates.
    const A_TOLERANT_PHASE: &str = r#"{ "name": "naming", "rules": [ "naming-convention" ], "threshold": { "max-blocking-findings": 3 } }"#;

    /// One phase, as `text` writes it.
    fn Declared_Phase(text: &str) -> DeclaredPhase
    {
        return serde_json::from_str(text).expect("every phase fixture in this module is written in the shape this module parses");
    }

    /// One approval, as `text` writes it.
    fn Declared_Approval(text: &str) -> DeclaredApproval
    {
        return serde_json::from_str(text).expect("every approval fixture in this module is written in the shape this module parses");
    }

    #[test]
    fn Test_A_Declared_Phase_Should_Resolve_Into_The_Types_A_Run_Judges_With()
    {
        let declared = vec![Declared_Phase(A_TOLERANT_PHASE)];
        let approved = vec![Declared_Approval(r#"{ "phase": "naming", "rationale": "reviewed with the owning team" }"#)];

        let phases = Resolved_Phases(declared);
        let approvals = Resolved_Approvals(approved);

        let [phase] = phases.as_slice()
        else
        {
            panic!("the fixture declares exactly one phase, got {phases:?}");
        };
        assert_eq!(phase.name, "naming");
        assert_eq!(phase.rules, vec![RuleId::New("naming-convention")]);
        assert_eq!(phase.threshold, PhaseThreshold::MaxBlockingFindings { max: TOLERATED_FINDINGS });
        assert_eq!(approvals.first().expect("the fixture declares one approval").phase, "naming");
    }

    /// The other spelling of a threshold, and the only one a phase states without a number.
    #[test]
    fn Test_A_Phase_Naming_Any_Blocking_Finding_Should_Resolve_To_That_Variant()
    {
        let declared = vec![Declared_Phase(r#"{ "name": "deps", "rules": [ "dependency-direction" ], "threshold": "any-blocking-finding" }"#)];

        let phases = Resolved_Phases(declared);

        assert_eq!(phases.first().expect("one phase").threshold, PhaseThreshold::AnyBlockingFinding);
    }

    /// An absent threshold tolerates nothing, which is the strict reading of absence this
    /// module's own doc argues for from `OD-GATE-029`.
    #[test]
    fn Test_A_Phase_Naming_No_Threshold_Should_Tolerate_No_Finding()
    {
        let declared = vec![Declared_Phase(r#"{ "name": "deps", "rules": [ "dependency-direction" ] }"#)];

        let phases = Resolved_Phases(declared);

        assert_eq!(
            phases.first().expect("one phase").threshold,
            PhaseThreshold::AnyBlockingFinding,
            "an absent threshold must not buy a tolerance nobody wrote"
        );
    }

    #[test]
    fn Test_A_Phase_Naming_No_Rules_Should_Be_Refused()
    {
        let declared = vec![Declared_Phase(r#"{ "name": "empty", "rules": [] }"#)];

        let problem = Phase_Problem(&declared, &[]).expect("a phase judging nothing is refused");

        assert!(problem.contains("names no rules"), "{problem}");
    }

    #[test]
    fn Test_A_Threshold_Tolerating_Zero_Findings_Should_Be_Refused()
    {
        let declared =
            vec![Declared_Phase(r#"{ "name": "naming", "rules": [ "naming-convention" ], "threshold": { "max-blocking-findings": 0 } }"#)];

        let problem = Phase_Problem(&declared, &[]).expect("a threshold tolerating none is refused");

        assert!(problem.contains("zero blocking findings"), "{problem}");
    }

    #[test]
    fn Test_Two_Phases_Sharing_One_Name_Should_Be_Refused()
    {
        let declared = vec![
            Declared_Phase(r#"{ "name": "one", "rules": [ "naming-convention" ] }"#),
            Declared_Phase(r#"{ "name": "one", "rules": [ "dependency-direction" ] }"#),
        ];

        let problem = Phase_Problem(&declared, &[]).expect("two stages under one name are refused");

        assert!(problem.contains("two phases are named 'one'"), "{problem}");
    }

    /// An approval naming a phase the file does not declare is refused rather than stored.
    ///
    /// The failure this catches is silence: the entry parses, resolves and matches no phase
    /// name in `Evaluated_Phases`, so an author who mistyped the phase gets a build that
    /// fails for the reason they thought they had approved away.
    #[test]
    fn Test_An_Approval_Naming_An_Undeclared_Phase_Should_Be_Refused()
    {
        let declared = vec![Declared_Phase(r#"{ "name": "naming", "rules": [ "naming-convention" ] }"#)];
        let approved = vec![Declared_Approval(r#"{ "phase": "nameing", "rationale": "typo" }"#)];

        let problem = Phase_Problem(&declared, &approved).expect("an approval matching no declared phase is refused");

        assert!(problem.contains("matches no phase this file declares"), "{problem}");
        assert!(problem.contains("'naming'"), "the refusal has to say what could have been named: {problem}");
    }

    #[test]
    fn Test_Neither_Family_Declared_Should_Have_Nothing_To_Refuse()
    {
        assert_eq!(Phase_Problem(&[], &[]), None);
        assert!(Resolved_Phases(Vec::new()).is_empty(), "an absent key is no phase policy, not an empty stage");
    }
}
