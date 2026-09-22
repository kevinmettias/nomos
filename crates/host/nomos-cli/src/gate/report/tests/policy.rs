//! What the `policy` verb reports: the winner, what it outranked, and the lines themselves.

use super::super::{ExitCode, Render_Policy};
use super::Judged_With;
use nomos_contracts::ConfigurationLayer;
use nomos_gate_orchestration::{
    CoveragePolicy, Effective_Gate_Policy, Effective_Policy_Report, GateRunResult, NoVerdict, PolicyContribution,
};

/// The byte the run id of every fixture here is derived from. Any byte would do; these tests
/// differ in their policy and in nothing else.
const POLICY_FIXTURE_FILL: u8 = 4;

/// The artifact an organization-level statement is reported under.
const AN_ORGANIZATION_FILE: &str = "the organization's own policy";

/// The artifact a repository-level statement is reported under.
const A_REPOSITORY_FILE: &str = "nomos-gate.json";

/// The artifact a statement the caller built is reported under.
const AN_INVOCATION: &str = "the policy the caller built";

/// A contribution from `layer`'s `artifact` stating only the coverage floor.
fn Stating_Coverage(layer: ConfigurationLayer, artifact: &str) -> PolicyContribution
{
    return PolicyContribution {
        coverage: Some(CoveragePolicy::RequireCompleteness),
        ..PolicyContribution::Silent(layer, artifact)
    };
}

/// What the `policy` verb writes for a run carrying `contributions`' own resolution.
///
/// The contributions are declared and resolved rather than read off three real sources,
/// because five of the ten layers have no source on this host at all -- `OD-POLICY-001`'s own
/// table -- so three real statements of one field cannot be produced by any composition root
/// that exists today. What is real is everything after: the resolver is the shipped one, the
/// result is the shape a run hands a host, and the rendering is the verb's.
fn Reported(contributions: &[PolicyContribution]) -> (ExitCode, String)
{
    let policy = Effective_Gate_Policy(contributions).expect("these fixtures state nothing the resolver refuses");
    let result = GateRunResult { policy: Some(Box::new(policy)), ..Judged_With(POLICY_FIXTURE_FILL, None) };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Policy(&result, &mut stdout, &mut stderr);

    return (code, String::from_utf8_lossy(&stdout).into_owned());
}

/// The claim this whole increment makes, end to end: a field three layers stated reports the
/// statement that decided it, with its layer and its artifact, and both statements it
/// outranked.
///
/// Both losers, not one. A report that named only the nearest loser would look right on every
/// two-layer resolution -- which is every resolution a real composition root produces today --
/// and would be wrong the first time a third source existed.
#[test]
fn Test_A_Field_Stated_At_Three_Layers_Should_Name_Its_Winner_And_Both_Losers()
{
    let (code, report) = Reported(&[
        Stating_Coverage(ConfigurationLayer::Organization, AN_ORGANIZATION_FILE),
        Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE),
        Stating_Coverage(ConfigurationLayer::CommandLine, AN_INVOCATION),
    ]);

    assert_eq!(code, ExitCode::Ok, "{report}");
    let deciding = Line_Containing(&report, "coverage:");
    assert!(deciding.contains(ConfigurationLayer::CommandLine.Label()), "the winner's layer: {deciding}");
    assert!(deciding.contains(AN_INVOCATION), "the winner's artifact: {deciding}");
    assert!(report.contains(AN_ORGANIZATION_FILE), "the lower loser must stay visible: {report}");
    assert!(report.contains(A_REPOSITORY_FILE), "the nearer loser must stay visible: {report}");
}

/// The verb prints the orchestration crate's own report and composes no sentence of its own.
///
/// This is what makes the command line and the canonical service agree: `nomos-api` projects
/// the same function's lines, so a second assembly here would let the two surfaces answer one
/// question differently while each looked right alone.
#[test]
fn Test_The_Policy_Verb_Should_Print_The_Report_Rather_Than_Assemble_One()
{
    let contributions = [Stating_Coverage(ConfigurationLayer::Repository, A_REPOSITORY_FILE)];
    let policy = Effective_Gate_Policy(&contributions).expect("one statement of one field is not refused");

    let (_, report) = Reported(&contributions);

    assert_eq!(report.lines().collect::<Vec<&str>>(), Effective_Policy_Report(&policy), "{report}");
}

/// A resolution that refused reports the refusal rather than an empty policy.
///
/// An empty report at exit zero would say the repository states nothing, which is a different
/// answer from "what it stated could not be resolved" -- and the second is the one an author
/// with a broken `nomos-gate.json` needs.
#[test]
fn Test_A_Refused_Resolution_Should_Report_The_Refusal_Rather_Than_An_Empty_Policy()
{
    let refusal = "'baseline' is stated twice at the Repository layer".to_owned();
    let result = GateRunResult {
        policy: None,
        no_verdict: Some(NoVerdict::MalformedPolicy(refusal.clone())),
        ..Judged_With(POLICY_FIXTURE_FILL, None)
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Policy(&result, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Contradictory);
    assert!(stdout.is_empty(), "{}", String::from_utf8_lossy(&stdout));
    assert!(String::from_utf8_lossy(&stderr).contains(&refusal), "{}", String::from_utf8_lossy(&stderr));
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
