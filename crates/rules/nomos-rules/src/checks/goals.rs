//! `goals-and-parts-line-up` — the one rule that reads `nomos.cap.goals.policy`.
//!
//! # The one rule in this crate whose subject is not source
//!
//! Every other rule here is handed [`SourceFile`]s and judges what they say. This one is
//! handed nothing but a [`FactReader`], because code-standards' own `check-goal-traceability`
//! reads nothing but the declaration: which purposes a system says it has, and which parts
//! say they serve them. Neither fact is in any file's text, and its own header says why —
//! "it is a fact about which parts are FOR what, and the only place that intent exists is
//! the declaration."
//!
//! That is why the signature takes no `sources`, and it is not an oversight. `OD-RULES-001`
//! requires a rule to take its subject as an argument rather than reading the tree; this
//! rule's subject is the declared policy, and it arrives as an argument through `facts`.
//! Nothing in this crate types rules uniformly — `registry::RuleOffer` carries a `RuleId`
//! and a record citation, not a function pointer — so a rule whose subject is not source is
//! free to say so in its own signature rather than accept a parameter it would not read.
//!
//! # Four disagreements, one rule id
//!
//! `AuditTraceability` reports four things and `goals-and-parts-line-up.md` is one rule
//! document, so this is one function emitting four kinds of finding rather than four
//! functions — the opposite of the split `facade.rs` makes, and for the opposite reason:
//! there, one tool carried three published rule documents; here, one rule document carries
//! four judgments. The unit of export stays the rule.
//!
//! An UNDECLARED goal is a part serving a purpose the system never agreed it has. An
//! ORPHANED goal is a declared purpose nothing was built for. A PURPOSELESS part serves no
//! declared purpose at all. A SMEARED goal is spread across more parts than the declared
//! ceiling allows.
//!
//! # Two opt-outs, both faithful
//!
//! A repository declaring no goals is judged nothing, and that is the whole rule's own
//! gate rather than a convenience: a goal cannot be inferred from code, so a repository
//! that has not written one down has not opted in. A ceiling of zero drops only the spread
//! bound and leaves the two-way audit standing, matching the Go implementation's own
//! `maxPerGoal > 0` guard.
//!
//! # Reading the capability is optional, the same way naming's is
//!
//! `OD-CAPABILITY-004` and `OD-RULES-011` settle this: a failed `Require` means "no policy
//! to judge against", never a `Finding` and never this capability's `Applicability`
//! surfacing. Here that collapses into the same answer as the opt-out above — no policy and
//! an empty policy both judge nothing — which is why this rule needs no fallback constant
//! of its own, unlike naming's and limits' prior hardcoded defaults.

use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_goals_policy::{GoalsPolicyPayload, SubsystemDeclaration};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards goal-traceability rule id.
pub const GOALS_AND_PARTS_LINE_UP: &str = "goals-and-parts-line-up";

/// Where a finding about the declaration points, since the declaration is its whole subject.
const DECLARATION_FILE: &str = "standards.json";

/// Reports every way a repository's declared purposes and its declared parts fail to line
/// up: a part serving an undeclared purpose, a declared purpose nothing serves, a part
/// serving no purpose, and a purpose spread past its declared ceiling.
///
/// Judges nothing when no policy is readable or none is declared. Both are the same answer
/// and neither is a `Finding`: goal traceability is a discipline a system takes on.
#[must_use]
pub fn Check_Goals_And_Parts_Line_Up(facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(payload) = Declared_Policy(facts)
    else
    {
        return Vec::new();
    };

    return Findings_For(&payload);
}

fn Declared_Policy(facts: &mut dyn FactReader) -> Option<GoalsPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_goals_policy::Capability(), &subject, InputDigest::Of(&[]), &Goals_Policy_Requirement())
        .ok()?;

    return nomos_cap_goals_policy::Parse_Payload(&fact.payload.bytes).ok();
}

/// This crate's own floor for `nomos.cap.goals.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this rule could
/// honestly still act on.
fn Goals_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_goals_policy::Capability(),
        nomos_cap_goals_policy::CONTRACT_VERSION,
        nomos_cap_goals_policy::Ceiling(),
    );
}

/// The audit itself, over a payload already in hand.
///
/// Ported from code-standards' `AuditTraceability`, including its ordering: the subsystem
/// pass first, then the goal pass, and the whole list sorted by kind, then subject, then the
/// related goal. Sorting by kind first is what groups a reader's attention on one sort of
/// disagreement at a time, and it is why this does not use the `subject_name` sort the
/// source-judging rules in this crate share — those order by file and line, which this rule
/// has none of.
fn Findings_For(payload: &GoalsPolicyPayload) -> Vec<Finding>
{
    if payload.goals.is_empty()
    {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let mut served_by: Vec<(&str, Vec<&str>)> = payload.goals.iter().map(|goal| return (goal.as_str(), Vec::new())).collect();

    Extend_With_Subsystem_Findings(&payload.subsystems, &payload.goals, &mut served_by, &mut findings);
    Extend_With_Goal_Findings(&served_by, payload.max_subsystems_per_goal, &mut findings);

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Judges every declared subsystem, appending to `findings` and recording, in `served_by`,
/// which declared goals each one serves.
fn Extend_With_Subsystem_Findings<'policy>(
    subsystems: &'policy [SubsystemDeclaration],
    declared: &[String],
    served_by: &mut [(&'policy str, Vec<&'policy str>)],
    findings: &mut Vec<Finding>,
)
{
    for subsystem in subsystems
    {
        let subsystem_findings = Subsystem_Findings(subsystem, declared, served_by);
        findings.extend(subsystem_findings);
    }
}

/// Judges one part — purposeless when it names no goal, undeclared for any goal outside the
/// declared set — and records, for the declared goals, that this part serves them.
fn Subsystem_Findings<'policy>(
    subsystem: &'policy SubsystemDeclaration,
    declared: &[String],
    served_by: &mut [(&'policy str, Vec<&'policy str>)],
) -> Vec<Finding>
{
    if subsystem.goals.is_empty()
    {
        return vec![Finding_For(Kind::PurposelessSubsystem, Subject(&subsystem.name), Detail(""))];
    }

    let mut findings = Vec::new();
    for goal in &subsystem.goals
    {
        if !declared.contains(goal)
        {
            let finding = Finding_For(Kind::UndeclaredGoal, Subject(&subsystem.name), Detail(goal));
            findings.push(finding);
            continue;
        }

        if let Some((_goal, servers)) = served_by.iter_mut().find(|(declared_goal, _servers)| return declared_goal == goal)
        {
            servers.push(subsystem.name.as_str());
        }
    }

    return findings;
}

/// Judges every declared goal against who serves it, appending an orphaned- or smeared-goal
/// finding to `findings` as each disagreement is found.
fn Extend_With_Goal_Findings(served_by: &[(&str, Vec<&str>)], ceiling: u32, findings: &mut Vec<Finding>)
{
    for (goal, servers) in served_by
    {
        if servers.is_empty()
        {
            let finding = Finding_For(Kind::OrphanedGoal, Subject(goal), Detail(""));
            findings.push(finding);
            continue;
        }

        if ceiling > 0 && servers.len() > ceiling as usize
        {
            let finding = Finding_For(Kind::SmearedGoal, Subject(goal), Detail(&format!("{} {}", servers.len(), ceiling)));
            findings.push(finding);
        }
    }
}

/// The four disagreements, in the order they sort — the same order code-standards' own
/// `kind` enum declares, so a reader comparing the two lists finds them in one order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind
{
    UndeclaredGoal,
    OrphanedGoal,
    PurposelessSubsystem,
    SmearedGoal,
}

impl Kind
{
    const UNDECLARED_GOAL_RANK: u8 = 0;
    const ORPHANED_GOAL_RANK: u8 = 1;
    const PURPOSELESS_SUBSYSTEM_RANK: u8 = 2;
    const SMEARED_GOAL_RANK: u8 = 3;

    /// The sort rank, kept out of the subject text so two kinds never interleave.
    const fn Rank(self) -> u8
    {
        return match self
        {
            Self::UndeclaredGoal => Self::UNDECLARED_GOAL_RANK,
            Self::OrphanedGoal => Self::ORPHANED_GOAL_RANK,
            Self::PurposelessSubsystem => Self::PURPOSELESS_SUBSYSTEM_RANK,
            Self::SmearedGoal => Self::SMEARED_GOAL_RANK,
        };
    }
}

/// The finding's own subject — a goal or subsystem name. Wrapped so it cannot be transposed
/// with [`Detail`] at a call site: both wrap `&str`, but only one names the thing judged.
struct Subject<'a>(&'a str);

/// The finding's auxiliary text: the related goal for an undeclared one, and the count and
/// ceiling (space-separated) for a smeared one; the other two kinds carry none. Wrapped for
/// the same reason as [`Subject`].
struct Detail<'a>(&'a str);

/// One finding.
fn Finding_For(kind: Kind, subject: Subject<'_>, detail: Detail<'_>) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(GOALS_AND_PARTS_LINE_UP),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: format!("{}:{}:{}", kind.Rank(), DECLARATION_FILE, subject.0),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: Summary_For(kind, Subject(subject.0), Detail(detail.0)),
        locations: vec![DECLARATION_FILE.to_owned()],
    };
}

fn Summary_For(kind: Kind, subject: Subject<'_>, detail: Detail<'_>) -> String
{
    let subject = subject.0;
    let detail = detail.0;

    return match kind
    {
        Kind::UndeclaredGoal => format!(
            "{DECLARATION_FILE} declares subsystem `{subject}` as serving goal `{detail}`, which is not in the declared \
             goal set; a purpose one part pursues that the system never agreed it has is either a goal to declare or a \
             part doing work that is not the plan"
        ),
        Kind::OrphanedGoal => format!(
            "{DECLARATION_FILE} declares goal `{subject}` and no subsystem serves it; the system claims a purpose \
             nothing was built for"
        ),
        Kind::PurposelessSubsystem => format!(
            "{DECLARATION_FILE} declares subsystem `{subject}` and it serves no declared goal; a part whose reason to \
             exist is unstated is where dead weight and scope creep both begin"
        ),
        Kind::SmearedGoal =>
        {
            let (count, ceiling) = detail.split_once(' ').unwrap_or((detail, ""));
            format!(
                "{DECLARATION_FILE} declares goal `{subject}` and {count} subsystems serve it, over the ceiling of \
                 {ceiling}; a purpose spread this thin has no part that owns it"
            )
        }
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Report_A_Subsystem_Serving_An_Undeclared_Goal()
    {
        let policy = Goals_Policy(&["render"], 0, &[("experimental", &["teleport"])]);
        let findings = Findings_For(&policy);

        const EXPECTED_FINDINGS: usize = 2;
        assert_eq!(findings.len(), EXPECTED_FINDINGS, "the declared goal is also orphaned: {findings:?}");
        assert!(
            findings.iter().any(|finding| return finding.summary.contains("not in the declared goal set")),
            "{findings:?}"
        );
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Report_A_Goal_Nothing_Serves()
    {
        let policy = Goals_Policy(&["render", "unbuilt"], 0, &[("graphics", &["render"])]);
        let findings = Findings_For(&policy);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above");
        assert_eq!(reported.rule, RuleId::New(GOALS_AND_PARTS_LINE_UP));
        assert!(reported.summary.contains("`unbuilt`"), "{reported:?}");
        assert!(reported.summary.contains("nothing was built for"), "{reported:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Report_A_Part_Serving_No_Goal()
    {
        let policy = Goals_Policy(&["render"], 0, &[("graphics", &["render"]), ("utils", &[])]);
        let findings = Findings_For(&policy);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings.first().expect("asserted len 1 above").summary.contains("serves no declared goal"),
            "{findings:?}"
        );
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Report_A_Goal_Spread_Past_Its_Ceiling()
    {
        let policy = Goals_Policy(&["simulate"], 1, &[("physics", &["simulate"]), ("audio", &["simulate"])]);
        let findings = Findings_For(&policy);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above");
        assert!(reported.summary.contains("2 subsystems serve it, over the ceiling of 1"), "{reported:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Report_All_Four_Disagreements_At_Once()
    {
        let policy = Goals_Policy(
            &["render", "simulate", "unbuilt"],
            1,
            &[
                ("graphics", &["render"]),
                ("physics", &["simulate"]),
                ("audio", &["simulate"]),
                ("utils", &[]),
                ("experimental", &["teleport"]),
            ],
        );
        let findings = Findings_For(&policy);

        const EXPECTED_FINDINGS: usize = 4;
        assert_eq!(findings.len(), EXPECTED_FINDINGS, "one of each, and nothing else: {findings:?}");
        let summaries: Vec<&str> = findings.iter().map(|finding| return finding.summary.as_str()).collect();
        assert!(summaries.first().is_some_and(|summary| return summary.contains("not in the declared goal set")), "{summaries:?}");
        assert!(summaries.get(1).is_some_and(|summary| return summary.contains("nothing was built for")), "{summaries:?}");
        const PURPOSELESS_SUBSYSTEM_INDEX: usize = 2;
        assert!(
            summaries.get(PURPOSELESS_SUBSYSTEM_INDEX).is_some_and(|summary| return summary.contains("serves no declared goal")),
            "{summaries:?}"
        );
        const SMEARED_GOAL_INDEX: usize = 3;
        assert!(summaries.get(SMEARED_GOAL_INDEX).is_some_and(|summary| return summary.contains("over the ceiling of")), "{summaries:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Judge_Nothing_When_No_Goal_Is_Declared()
    {
        let policy = Goals_Policy(&[], 1, &[("utils", &[]), ("experimental", &["teleport"])]);
        let findings = Findings_For(&policy);

        assert!(findings.is_empty(), "a repository that declared no goals has not opted in: {findings:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Not_Judge_Spread_With_No_Ceiling()
    {
        let policy = Goals_Policy(&["render"], 0, &[("a", &["render"]), ("b", &["render"]), ("c", &["render"])]);
        let findings = Findings_For(&policy);

        assert!(findings.is_empty(), "a ceiling of zero drops only the spread bound: {findings:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Accept_A_Policy_That_Lines_Up()
    {
        let policy = Goals_Policy(&["render", "simulate"], 1, &[("graphics", &["render"]), ("physics", &["simulate"])]);
        let findings = Findings_For(&policy);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Count_A_Part_Serving_Two_Goals_Against_Both()
    {
        let policy = Goals_Policy(&["render", "simulate"], 0, &[("engine", &["render", "simulate"])]);
        let findings = Findings_For(&policy);

        assert!(findings.is_empty(), "one part may serve two purposes: {findings:?}");
    }

    #[test]
    fn Test_Check_Goals_And_Parts_Line_Up_Should_Point_Every_Finding_At_The_Declaration()
    {
        let policy = Goals_Policy(&["unbuilt"], 0, &[]);
        let findings = Findings_For(&policy);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").locations, vec![DECLARATION_FILE.to_owned()]);
    }

    fn Goals_Policy(goals: &[&str], ceiling: u32, subsystems: &[(&str, &[&str])]) -> GoalsPolicyPayload
    {
        return GoalsPolicyPayload {
            goals: goals.iter().map(|goal| return (*goal).to_owned()).collect(),
            max_subsystems_per_goal: ceiling,
            subsystems: subsystems
                .iter()
                .map(|(name, served)| {
                    return SubsystemDeclaration {
                        name: (*name).to_owned(),
                        goals: served.iter().map(|goal| return (*goal).to_owned()).collect(),
                    };
                })
                .collect(),
        };
    }
}
