//! What the report must say: which statement decided a field, which it outranked, which a
//! lock refused, and which layers had no source at all.

use super::{ABSENT_HEADING, Effective_Policy_Report, OVERRODE, REJECTED};
use crate::policy::{CoveragePolicy, Effective_Gate_Policy, EffectivePolicy, PolicyContribution, PolicyField};
use crate::{GatePhase, PhaseThreshold};
use nomos_contracts::{ConfigurationLayer, RuleId};

/// The artifact a fixture's repository-level source is reported under.
const A_REPOSITORY_FILE: &str = "nomos-gate.json";

/// The artifact a fixture's organization-level source is reported under.
const AN_ORGANIZATION_FILE: &str = "the organization's own policy";

/// The artifact a fixture's invocation is reported under.
const AN_INVOCATION: &str = "the policy the caller built";

/// A contribution from `layer`'s `artifact` that states only `coverage`.
fn Stating_Coverage(layer: ConfigurationLayer, artifact: &str, coverage: CoveragePolicy) -> PolicyContribution
{
    return PolicyContribution { coverage: Some(coverage), ..PolicyContribution::Silent(layer, artifact) };
}

/// The effective policy `contributions` resolve to, for a fixture that must not be refused.
fn Resolved(contributions: &[PolicyContribution]) -> EffectivePolicy
{
    return Effective_Gate_Policy(contributions).expect("this fixture states nothing the resolver refuses");
}

/// `contributions` resolved and rendered as the one block every host prints.
fn Reported(contributions: &[PolicyContribution]) -> String
{
    return Effective_Policy_Report(&Resolved(contributions)).join("\n");
}

/// The ordinary case: the layer and the artifact that decided a field are both on its line.
///
/// Both, not one: `OD-POLICY-001` keeps them apart because the `Repository` layer has four
/// artifacts, so a report naming only the layer cannot say which source spoke.
#[test]
fn Test_An_Ordinary_Field_Should_Name_The_Layer_And_The_Artifact_That_Decided_It()
{
    let report = Reported(&[Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness)]);

    let line = Line_For(&report, PolicyField::Coverage);
    assert!(line.contains(ConfigurationLayer::Repository.Label()), "{line}");
    assert!(line.contains(A_REPOSITORY_FILE), "{line}");
}

/// A field nobody stated is decided by the build's own defaults, and says so rather than
/// leaving a blank where a provenance belongs.
#[test]
fn Test_A_Field_Nobody_Stated_Should_Name_The_Default_Layer()
{
    let report = Reported(&[PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)]);

    let line = Line_For(&report, PolicyField::Coverage);
    assert!(line.contains(ConfigurationLayer::Default.Label()), "{line}");
}

/// `OD-POLICY-001` version 2: every field of a resolved unit reports the unit *and* which
/// field's statement decided it.
///
/// Asked of `approvals`, the companion, because that is the field a provenance naming only
/// itself would misreport: the source that decided it wrote no approvals at all, and a line
/// reading "approvals: `Repository`, `nomos-gate.json`" would say it did.
#[test]
fn Test_A_Companion_Of_A_Unit_Should_Name_The_Unit_And_The_Field_Whose_Statement_Decided_It()
{
    let stated = PolicyContribution {
        phases: Some(vec![A_Phase()]),
        ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
    };

    let report = Reported(&[stated]);

    let line = Line_For(&report, PolicyField::Approvals);
    assert!(line.contains(crate::PHASE_POLICY_UNIT.name), "the unit must be named: {line}");
    assert!(line.contains(PolicyField::Phases.Label()), "the deciding field must be named: {line}");
}

/// `CONFIG-003` through `OD-POLICY-001` version 3: a refused statement reads as rejected,
/// carries the reason, and is never filed among the contributions the winner outranked.
///
/// The negative half is the load-bearing one. A report that listed a rejection under
/// [`OVERRODE`] would tell its author their value lost a precedence contest it was never
/// admitted to, and from a green run that reading is indistinguishable from this one.
#[test]
fn Test_A_Rejected_Override_Should_Read_As_Rejected_Rather_Than_Overridden()
{
    let locking = PolicyContribution {
        locks: vec![PolicyField::Coverage],
        ..Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness)
    };
    let refused = Stating_Coverage(ConfigurationLayer::Organization, AN_ORGANIZATION_FILE, CoveragePolicy::Unset);

    let report = Reported(&[locking, refused]);

    assert!(report.contains(REJECTED), "the refusal must be reported at all: {report}");
    let rejection = Line_Containing(&report, REJECTED);
    assert!(rejection.contains(AN_ORGANIZATION_FILE), "the refused statement's own artifact: {rejection}");
    assert!(rejection.contains(A_REPOSITORY_FILE), "the reason names what locked the field: {rejection}");
    assert!(!report.contains(OVERRODE), "a rejected statement was forbidden, not outranked: {report}");
}

/// `OD-POLICY-001`: "a layer with no source contributes nothing and is reported as absent,
/// never as an empty contribution."
///
/// The heading itself is asserted, not merely the layer name: a layer listed under a heading
/// that did not say which of the two facts it is would leave the reader to guess, which is
/// exactly the confusion the record's sentence exists to prevent.
#[test]
fn Test_A_Layer_With_No_Source_Should_Read_As_Absent_Rather_Than_As_An_Empty_Contribution()
{
    let report = Reported(&[Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness)]);

    assert!(report.contains(ABSENT_HEADING), "{report}");
    let named = Line_Containing(&report, ConfigurationLayer::Organization.Label());
    assert_eq!(named.trim(), ConfigurationLayer::Organization.Label(), "{report}");
}

/// A host on which every layer had a source names none as absent, so the section is not a
/// heading over nothing on the one run that has nothing to say here.
#[test]
fn Test_A_Resolution_With_Every_Layer_Present_Should_Name_No_Absent_Layer()
{
    let every: Vec<PolicyContribution> =
        ConfigurationLayer::ALL.into_iter().map(|layer| return PolicyContribution::Silent(layer, A_REPOSITORY_FILE)).collect();

    let report = Reported(&every);

    assert!(!report.contains(ABSENT_HEADING), "{report}");
}

/// One stage, enough to make a source a statement of the phase policy.
fn A_Phase() -> GatePhase
{
    return GatePhase {
        name: "review".to_owned(),
        rules: vec![RuleId::New("a-rule")],
        threshold: PhaseThreshold::AnyBlockingFinding,
    };
}

/// The one line of `report` that reports `field`.
fn Line_For(report: &str, field: PolicyField) -> String
{
    let wanted = format!("{}:", field.Label());

    return Line_Containing(report, &wanted);
}

/// The first line of `report` containing `needle`.
fn Line_Containing(report: &str, needle: &str) -> String
{
    return report
        .lines()
        .find(|line| return line.contains(needle))
        .unwrap_or_else(|| panic!("no line of the report contains '{needle}': {report}"))
        .to_owned();
}
