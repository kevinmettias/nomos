//! What resolving layer-labelled contributions must mean: which layer decides a field, what a
//! provenance says about it, which statements are refused, and which layer had no source at
//! all.

use super::{
    Effective_Gate_Policy, EffectivePolicy, PHASE_POLICY_UNIT, PolicyContribution, PolicyField, PolicyRefusal, Resolved_Gate_Policy,
};
use crate::policy::{AdoptionPolicy, BaselinePolicy, CoveragePolicy, GatePolicyFile, RuleCalibration, SuppressionPolicy};
use crate::{GateCommand, GatePhase, PhaseApproval, PhaseThreshold};
use nomos_contracts::{ConfigurationLayer, RuleId};

/// The artifact name a fixture's repository-level source is reported under.
const A_REPOSITORY_FILE: &str = "nomos-gate.json";

/// A second artifact of the repository layer, for the cases about two sources of one layer.
const ANOTHER_REPOSITORY_FILE: &str = "nomos-gate.local.json";

/// The artifact name a fixture's workflow step is reported under.
const A_WORKFLOW_STEP: &str = "the workflow step's own gate body";

/// The artifact name a fixture's invocation is reported under.
const AN_INVOCATION: &str = "the policy the caller built";

/// A contribution from `layer`'s `artifact` that states only `coverage`.
fn Stating_Coverage(layer: ConfigurationLayer, artifact: &str, coverage: CoveragePolicy) -> PolicyContribution
{
    return PolicyContribution { coverage: Some(coverage), ..PolicyContribution::Silent(layer, artifact) };
}

/// One stage judging one rule, named `name`.
fn A_Phase(name: &str, rule: &str) -> GatePhase
{
    return GatePhase { name: name.to_owned(), rules: vec![RuleId::New(rule)], threshold: PhaseThreshold::AnyBlockingFinding };
}

/// One approval of the stage `phase`.
fn An_Approval(phase: &str) -> PhaseApproval
{
    return PhaseApproval { phase: phase.to_owned(), rationale: "reviewed".to_owned() };
}

/// One calibration of `rule`, with `rationale` deciding whether two of them are the same entry.
fn A_Calibration(rule: &str, rationale: &str) -> AdoptionPolicy
{
    return AdoptionPolicy { calibrated: vec![RuleCalibration { rule: RuleId::New(rule), rationale: rationale.to_owned() }] };
}

/// The effective policy `contributions` resolve to, for a fixture that must not be refused.
fn Resolved(contributions: &[PolicyContribution]) -> EffectivePolicy
{
    return Effective_Gate_Policy(contributions).expect("this fixture states nothing the resolver refuses");
}

/// `OD-POLICY-001`'s central claim, asked of one field: the highest layer that states it
/// decides it, the effective policy names what decided it, and every statement it outranked
/// stays visible with its own layer and artifact.
///
/// Three layers rather than two, because two is the case `Resolved_Over` already handled and a
/// two-level rule that happens to work is indistinguishable from a layered one over two
/// layers. The middle layer states the *stronger* floor and still loses, so the assertion
/// cannot be satisfied by a resolver that picked the most restrictive value rather than the
/// highest layer.
#[test]
fn Test_A_Field_Stated_At_Three_Layers_Should_Take_The_Highest_And_Name_What_It_Overrode()
{
    let contributions = [
        Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::Unset),
        Stating_Coverage(ConfigurationLayer::Workflow, A_WORKFLOW_STEP, CoveragePolicy::RequireCompleteness),
        Stating_Coverage(ConfigurationLayer::CommandLine, AN_INVOCATION, CoveragePolicy::Unset),
    ];

    let effective = Resolved(&contributions);
    let coverage = effective.Deciding(PolicyField::Coverage).expect("every field is resolved");

    assert_eq!(effective.values.coverage, CoveragePolicy::Unset, "the highest layer that states the field decides it");
    assert_eq!(coverage.decided_by.layer, ConfigurationLayer::CommandLine);
    assert_eq!(coverage.decided_by.artifact, AN_INVOCATION);
    assert_eq!(
        coverage.overrode.iter().map(|overridden| return overridden.layer).collect::<Vec<ConfigurationLayer>>(),
        vec![ConfigurationLayer::Repository, ConfigurationLayer::Workflow],
        "every contribution the winner outranked is named, lowest layer first"
    );
    assert_eq!(
        coverage.overrode.iter().map(|overridden| return overridden.artifact.clone()).collect::<Vec<String>>(),
        vec![A_REPOSITORY_FILE.to_owned(), A_WORKFLOW_STEP.to_owned()],
        "a layer alone does not say which source spoke, so the artifact is carried beside it"
    );
}

/// The order the contributions arrive in decides nothing.
///
/// `MODEL-ROUTE-023` forbids declaration order as a way of settling precedence, and a slice is
/// exactly the shape that would make it available. Reversing the slice must not move a single
/// answer.
#[test]
fn Test_Declaration_Order_Should_Decide_Nothing()
{
    let mut contributions = vec![
        Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness),
        Stating_Coverage(ConfigurationLayer::CommandLine, AN_INVOCATION, CoveragePolicy::Unset),
    ];

    let as_written = Resolved(&contributions);
    contributions.reverse();
    let reversed = Resolved(&contributions);

    assert_eq!(as_written, reversed, "the slice's order reached the answer, which is the one way of settling precedence the corpus forbids");
}

/// A value nobody stated is decided by the build, at the `Default` layer.
///
/// `OD-POLICY-001` names "the build for a default" among the artifacts, so a field the ten
/// layers all left alone still has a provenance rather than a blank.
#[test]
fn Test_A_Field_Nobody_States_Should_Be_Decided_By_The_Build()
{
    let effective = Resolved(&[PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)]);
    let coverage = effective.Deciding(PolicyField::Coverage).expect("every field is resolved");

    assert_eq!(coverage.decided_by.layer, ConfigurationLayer::Default);
    assert!(coverage.overrode.is_empty(), "a field nobody stated outranked nobody");
    assert_eq!(effective.values.coverage, CoveragePolicy::default());
}

/// A layer with no source is reported absent, never as a contribution that stated nothing.
///
/// "Organization: no source on this host" and "Organization: declared nothing" are different
/// facts, and the assertion is in both directions so a resolver that listed every layer as
/// absent would fail it too.
#[test]
fn Test_A_Layer_With_No_Source_Should_Be_Reported_Absent()
{
    let effective = Resolved(&[PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)]);

    assert!(effective.absent_layers.contains(&ConfigurationLayer::Organization), "no source named an organization");
    assert!(!effective.absent_layers.contains(&ConfigurationLayer::Repository), "a source that stated nothing is present, not absent");
    assert!(!effective.absent_layers.contains(&ConfigurationLayer::Default), "the build always states the defaults");
}

/// The same key stated at two layers takes the higher layer's entry and records the lower one.
///
/// The keyed-union shape: the layers' other entries survive beside it, because no layer removes
/// a lower layer's entry.
#[test]
fn Test_A_Key_Stated_At_Two_Layers_Should_Take_The_Higher_And_Record_The_Lower()
{
    let contributions = [
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "adopting incrementally")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "the invocation says otherwise")),
            ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)
        },
    ];

    let effective = Resolved(&contributions);
    let adoption = effective.Deciding(PolicyField::Adoption).expect("every field is resolved");

    assert_eq!(effective.values.adoption.calibrated.len(), 1, "one key stated twice is one entry, not two");
    assert_eq!(
        effective.values.adoption.calibrated.first().expect("one entry").rationale,
        "the invocation says otherwise",
        "the higher layer's entry is the one that survives"
    );
    assert_eq!(
        adoption.overrode.iter().map(|overridden| return overridden.layer).collect::<Vec<ConfigurationLayer>>(),
        vec![ConfigurationLayer::Repository],
        "the entry that lost stays visible with its own layer"
    );
}

/// Two layers' *different* keys are unioned rather than replaced, which is what makes the
/// merged shape different from the override one.
#[test]
fn Test_Two_Layers_Stating_Different_Keys_Should_Both_Survive()
{
    let contributions = [
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "adopting incrementally")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            adoption: Some(A_Calibration("dependency-direction", "adopting incrementally")),
            ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)
        },
    ];

    let effective = Resolved(&contributions);

    assert_eq!(effective.values.adoption.calibrated.len(), 2, "no layer removes a lower layer's entry");
}

/// The unit: a caller's phases never pair with a file's approvals, in both directions.
///
/// `OD-POLICY-001` version 2 declares `phases` and `approvals` one unit with `phases`
/// deciding. The second half is what a per-field resolver would get wrong: the file states
/// approvals and the command does not, and a keyed union or an override *per field* would hand
/// the command's stages the file's approvals -- an approval addressing a stage its own source
/// never declared.
#[test]
fn Test_A_Callers_Phases_Should_Never_Pair_With_A_Files_Approvals()
{
    let from_file = PolicyContribution {
        phases: Some(vec![A_Phase("declared", "naming-convention")]),
        approvals: Some(vec![An_Approval("declared")]),
        ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
    };
    let silent = PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION);

    let taken_from_the_file = Resolved(&[from_file.clone(), silent]);

    assert_eq!(taken_from_the_file.values.phases.len(), 1, "a caller stating no phase takes the file's");
    assert_eq!(taken_from_the_file.values.approvals.len(), 1, "and the approvals that came with them");

    let built = PolicyContribution {
        phases: Some(vec![A_Phase("built", "dependency-direction")]),
        ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)
    };

    let taken_from_the_caller = Resolved(&[from_file, built]);

    assert_eq!(taken_from_the_caller.values.phases.first().expect("the caller's own phase").name, "built");
    assert!(
        taken_from_the_caller.values.approvals.is_empty(),
        "a source stating its own stages states its own approvals too, including none, so a file's approval must not reach a caller's stage"
    );
}

/// Every field of a resolved unit carries the deciding field's provenance and says that it did.
///
/// A provenance naming only its own field would report `approvals` as though its source had
/// written approvals when that source wrote none -- the difference between a value and a
/// statement. And the overridden list is recorded for every field of the unit, so a reader of
/// `approvals` alone sees that a file's approvals lost although only `phases` was compared.
#[test]
fn Test_Every_Field_Of_A_Unit_Should_Carry_The_Deciding_Fields_Provenance()
{
    let contributions = [
        PolicyContribution {
            phases: Some(vec![A_Phase("declared", "naming-convention")]),
            approvals: Some(vec![An_Approval("declared")]),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            phases: Some(vec![A_Phase("built", "dependency-direction")]),
            ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)
        },
    ];

    let effective = Resolved(&contributions);
    let approvals = effective.Deciding(PolicyField::Approvals).expect("every field is resolved");

    assert_eq!(approvals.decided_by.layer, ConfigurationLayer::CommandLine, "the deciding field's layer decided this one too");
    assert_eq!(approvals.decided_by.artifact, AN_INVOCATION);
    assert_eq!(
        approvals.decided_by.decided_by_unit,
        Some(PHASE_POLICY_UNIT),
        "a provenance naming only its own field would hide that another field's statement decided it"
    );
    assert_eq!(
        approvals.overrode.iter().map(|overridden| return overridden.layer).collect::<Vec<ConfigurationLayer>>(),
        vec![ConfigurationLayer::Repository],
        "a unit overridden whole records the lower layer's contribution for every field of it"
    );
    assert!(approvals.decided_by.Sentence().contains("phases"), "the sentence a report reads has to name the field that decided it");
}

/// A companion stated without its deciding field is refused, naming the unit, the field and the
/// artifact.
#[test]
fn Test_An_Approval_Stated_Without_Phases_Should_Be_Refused()
{
    let contributions = [PolicyContribution {
        approvals: Some(vec![An_Approval("declared")]),
        ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
    }];

    let refusal = Effective_Gate_Policy(&contributions).expect_err("a companion with no deciding field is refused");

    let PolicyRefusal::CompanionWithoutADecidingField { unit, field, artifact, .. } = refusal
    else
    {
        panic!("an approval with no phases is the unit's own refusal, not one of the other three: {refusal:?}");
    };
    assert_eq!(unit, PHASE_POLICY_UNIT);
    assert_eq!(field, PolicyField::Approvals);
    assert_eq!(artifact, A_REPOSITORY_FILE);
}

/// The unit's refusal is a property of one contribution, so another layer supplying the
/// deciding field does not rescue it.
///
/// `US-CONFIG-002`'s "the resolved policy is reproducible", read at the resolver: a
/// configuration must not become valid because a layer appeared.
#[test]
fn Test_An_Orphaned_Approval_Should_Be_Refused_Even_Where_Another_Layer_States_Phases()
{
    let contributions = [
        PolicyContribution {
            approvals: Some(vec![An_Approval("declared")]),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            phases: Some(vec![A_Phase("declared", "naming-convention")]),
            ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, AN_INVOCATION)
        },
    ];

    let refusal = Effective_Gate_Policy(&contributions).expect_err("a partial unit refuses whoever else spoke");

    assert!(matches!(refusal, PolicyRefusal::CompanionWithoutADecidingField { .. }), "{refusal:?}");
}

/// One field stated twice within one layer with different values is refused rather than
/// resolved, and the refusal names the key and both artifacts.
#[test]
fn Test_One_Field_Stated_Twice_Within_A_Layer_Should_Be_Refused()
{
    let contributions = [
        Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness),
        Stating_Coverage(ConfigurationLayer::Repository, ANOTHER_REPOSITORY_FILE, CoveragePolicy::Unset),
    ];

    let refusal = Effective_Gate_Policy(&contributions).expect_err("two sources of one layer disagreeing is refused");

    let PolicyRefusal::ContradictionWithinALayer { layer, key, artifacts, .. } = refusal
    else
    {
        panic!("two artifacts of one layer disagreeing is the contradiction case: {refusal:?}");
    };
    assert_eq!(layer, ConfigurationLayer::Repository);
    assert_eq!(key, "coverage");
    assert_eq!(artifacts, vec![A_REPOSITORY_FILE.to_owned(), ANOTHER_REPOSITORY_FILE.to_owned()]);
}

/// The same rule calibrated twice within one layer with different rationales is refused, and
/// the refusal names the rule rather than the field alone.
#[test]
fn Test_One_Key_Of_A_Merged_Field_Stated_Twice_Within_A_Layer_Should_Be_Refused()
{
    let contributions = [
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "one reason")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "another reason")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, ANOTHER_REPOSITORY_FILE)
        },
    ];

    let refusal = Effective_Gate_Policy(&contributions).expect_err("one key stated twice within one layer is refused");

    let PolicyRefusal::ContradictionWithinALayer { key, field, .. } = refusal
    else
    {
        panic!("one key stated twice within one layer is the contradiction case: {refusal:?}");
    };
    assert_eq!(field, PolicyField::Adoption);
    assert!(key.contains("naming-convention"), "the refusal has to name the offending key: {key}");
}

/// The same key stated twice within one layer with the *same* value is not a contradiction.
///
/// The falsifier for the assertion above: a resolver that refused on key collision alone would
/// refuse two sources that agree, which is a configuration nobody has to repair.
#[test]
fn Test_One_Key_Stated_Twice_Within_A_Layer_With_One_Value_Should_Resolve()
{
    let contributions = [
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "one reason")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
        },
        PolicyContribution {
            adoption: Some(A_Calibration("naming-convention", "one reason")),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, ANOTHER_REPOSITORY_FILE)
        },
    ];

    let effective = Resolved(&contributions);

    assert_eq!(effective.values.adoption.calibrated.len(), 1, "two sources saying one thing state one entry");
}

/// An artifact present and unreadable refuses rather than reading as empty.
#[test]
fn Test_An_Unreadable_Artifact_Should_Be_Refused()
{
    let contributions = [PolicyContribution {
        unreadable: Some("expected value at line 1 column 3".to_owned()),
        ..PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)
    }];

    let refusal = Effective_Gate_Policy(&contributions).expect_err("an artifact that cannot be read is refused");

    let PolicyRefusal::UnreadableArtifact { artifact, detail, .. } = refusal
    else
    {
        panic!("a present and unreadable artifact is the third refusal case: {refusal:?}");
    };
    assert_eq!(artifact, A_REPOSITORY_FILE);
    assert!(detail.contains("line 1"), "the refusal carries what the reader said was wrong: {detail}");
}

/// A locked field rejects a lower layer's statement, keeps it visible with its reason, and does
/// not take its value.
///
/// `CONFIG-003`. The rejection is not an override: nothing outranked the statement, it was
/// forbidden, and a report that said "overridden" would tell its author something untrue.
///
/// The lock is stated by `Workflow` rather than by `Organization`, although `CONFIG-003` names
/// organization policy: `OD-POLICY-001`'s layer order places `Organization` *below*
/// `Repository`, and the rule it decides is that a lower layer may not state a field **a higher
/// layer** has locked. The mechanism is the layer rank, so a fixture locking from below would
/// assert nothing. Which layer should be able to lock is a corpus question this item does not
/// answer -- see `PolicyContribution::locks`.
#[test]
fn Test_A_Locked_Field_Should_Reject_A_Lower_Layers_Statement_And_Keep_It_Visible()
{
    let contributions = [
        Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE, CoveragePolicy::RequireCompleteness),
        PolicyContribution {
            locks: vec![PolicyField::Coverage],
            ..PolicyContribution::Silent(ConfigurationLayer::Workflow, A_WORKFLOW_STEP)
        },
    ];

    let effective = Resolved(&contributions);
    let coverage = effective.Deciding(PolicyField::Coverage).expect("every field is resolved");

    assert_eq!(effective.values.coverage, CoveragePolicy::default(), "a rejected statement does not decide the value");
    assert!(coverage.overrode.is_empty(), "nothing outranked it: it was forbidden");
    let rejected = coverage.rejected.first().expect("the refused statement stays visible");
    assert_eq!(rejected.offered_by.layer, ConfigurationLayer::Repository);
    assert!(rejected.reason.contains("locked"), "a rejected override is kept visible with its reason: {}", rejected.reason);
}

/// The two-level case the gate has always had, reached through the resolver.
///
/// This is `Resolved_Over`'s own behavior stated as one case of the layered resolution: the
/// file contributes at `Repository`, the command at `CommandLine` above it, and what a run
/// judges under is unchanged.
#[test]
fn Test_The_Command_Line_Should_Resolve_Over_The_Repository()
{
    let from_file = GatePolicyFile { coverage: CoveragePolicy::RequireCompleteness, ..GatePolicyFile::default() };
    let silent = GateCommand::default();

    let taken_from_the_file = Resolved_Gate_Policy(Some(&from_file), &silent).expect("neither source states anything refusable");

    assert_eq!(taken_from_the_file.values.coverage, CoveragePolicy::RequireCompleteness);
    assert_eq!(
        taken_from_the_file.Deciding(PolicyField::Coverage).expect("every field is resolved").decided_by.layer,
        ConfigurationLayer::Repository,
        "a field only the file stated is the file's, and the provenance says so"
    );

    let stated = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..GateCommand::default() };
    let silent_file = GatePolicyFile::default();

    let taken_from_the_caller = Resolved_Gate_Policy(Some(&silent_file), &stated).expect("neither source states anything refusable");

    assert_eq!(taken_from_the_caller.values.coverage, CoveragePolicy::RequireCompleteness);
    assert_eq!(
        taken_from_the_caller.Deciding(PolicyField::Coverage).expect("every field is resolved").decided_by.layer,
        ConfigurationLayer::CommandLine
    );
}

/// A sentinel in either source states nothing, so the value a run judges under is the build's.
///
/// `OD-GATE-029` carried into the resolver: a file or a command left at its default has not
/// stated the default, it has said nothing, and the difference is what lets a higher layer's
/// silence leave a lower layer's statement standing.
#[test]
fn Test_A_Sentinel_In_Either_Source_Should_State_Nothing()
{
    let silent_file = GatePolicyFile {
        coverage: CoveragePolicy::Unset,
        suppressions: SuppressionPolicy::default(),
        baseline: BaselinePolicy::default(),
        ..GatePolicyFile::default()
    };

    let effective = Resolved_Gate_Policy(Some(&silent_file), &GateCommand::default()).expect("two silent sources refuse nothing");

    for field in [PolicyField::Coverage, PolicyField::Suppressions, PolicyField::Baseline, PolicyField::Adoption, PolicyField::Phases]
    {
        assert_eq!(
            effective.Deciding(field).expect("every field is resolved").decided_by.layer,
            ConfigurationLayer::Default,
            "{field} was decided by a source that only carried a sentinel"
        );
    }
}

/// Every field of the policy is resolved, so no field can be one a report has no line for.
#[test]
fn Test_Every_Policy_Field_Should_Be_Resolved()
{
    let effective = Resolved(&[PolicyContribution::Silent(ConfigurationLayer::Repository, A_REPOSITORY_FILE)]);

    for field in [
        PolicyField::Suppressions,
        PolicyField::Baseline,
        PolicyField::Adoption,
        PolicyField::Coverage,
        PolicyField::Phases,
        PolicyField::Approvals,
    ]
    {
        assert!(effective.Deciding(field).is_some(), "{field} has no resolved entry, so nothing could report what decided it");
    }
}
