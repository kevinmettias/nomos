//! [`SarifLog`], the document a projection produces, with the two functions that produce one.

use super::run_properties::RunProperties;
use super::sarif_invocation::SarifInvocation;
use super::sarif_result::SarifResult;
use super::sarif_run::SarifRun;
use crate::{FindingDisposition, GateFindings, GateRunResult};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::Finding;
use serde::Serialize;
use std::path::Path;

/// SARIF 2.1.0 §3.13.2's `version`, the one value this projection emits. The property is
/// required and the specification admits exactly this string for this revision.
const SARIF_VERSION: &str = "2.1.0";

/// SARIF 2.1.0 §3.13.3's `$schema`: the specification's own published schema URI for this
/// revision, in its errata01 spelling -- the address the OASIS document itself gives, rather
/// than a mirror a consumer may or may not resolve.
const SARIF_SCHEMA: &str = "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

/// How many finding groups a gate run reduces its findings into, one per
/// [`FindingDisposition`] -- the arity of [`Bucketed_Findings`], which emits every one.
const GATE_RUN_BUCKETS: usize = 6;

/// SARIF 2.1.0 §3.13's `sarifLog` object: the root of the document a CI consumer ingests.
///
/// Every field is private and the type carries no reader. A log exists to be written once, by
/// whichever serializer its caller already has, and read by a consumer in another process --
/// nothing in this workspace reads one back, so no accessor and no `Deserialize` is offered
/// for a round trip nobody performs. `Serialize` is the whole of the surface, which is also
/// why the SARIF object types beneath this one stay `pub(crate)`: a caller needs a document,
/// not a SARIF library, and `private_interfaces` is denied in this workspace precisely so a
/// public field cannot leak one of them by accident.
///
/// Exactly one `run` per log. A judgment is one execution of one tool over one tree, and
/// `runs` is a sequence only because SARIF admits a log assembled from several.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SarifLog
{
    /// Always [`SARIF_VERSION`].
    version: &'static str,
    /// Always [`SARIF_SCHEMA`]. Serialized under the specification's own `$schema` spelling,
    /// which is not a Rust identifier.
    #[serde(rename = "$schema")]
    schema: &'static str,
    /// Exactly one, for the reason the type's own doc gives.
    runs: Vec<SarifRun>,
}

impl SarifLog
{
    /// A gate run as a SARIF 2.1.0 log.
    ///
    /// The root every artifact uri is taken relative to is `result.root`, the tree the run
    /// itself recorded having judged, so the projection needs nothing from its caller that the
    /// judgment does not already carry.
    ///
    /// Every finding the run reduced is emitted, in every bucket, and a suppressed, baselined
    /// or calibrated one carries a `suppressions` entry rather than being dropped. A run that
    /// judged nothing, judged incompletely, or judged and reached no verdict is emitted as a
    /// run whose invocation is unsuccessful and says why, never as an empty clean log.
    ///
    /// Every rule whose population was empty is named in the run's own `properties`, under
    /// `emptyPopulations`, and nowhere that would move the verdict: the crate-private
    /// `RunProperties` says why that is the place. Every value a rule read that the repository
    /// never declared is named beside them and apart from them, under `undeclaredValues`, one
    /// object per value carrying its rule's id, family, `declaredIn`, key, language and outcome.
    /// A run in which every selected rule judged something and every value was declared carries
    /// neither property, so its log is what it was before there was one.
    #[must_use]
    pub fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        let results = Gate_Run_Results(&result.root, &result.findings);
        let properties = RunProperties::Of(&result.check_outcome);

        return Self::Over(SarifRun::Of(results, SarifInvocation::Of_Gate_Run(result), properties));
    }

    /// A check run as a SARIF 2.1.0 log.
    ///
    /// `root` is a parameter here and not for [`Self::Of_Gate_Run`] because [`CheckOutcome`]
    /// carries no root of its own: a bare check applies no policy and records no tree, so the
    /// only honest source for what a location is relative to is the caller that named the root
    /// the check ran over.
    ///
    /// Every result's `properties` carries no `bucket`, since no policy was applied and there
    /// is no bucket to report; the absence is the answer, and `OD-HOST-002`'s reconstructibility
    /// is why it is absent rather than defaulted to `blocking`.
    ///
    /// The rules whose population was empty, and the values nobody declared, are named exactly
    /// as [`Self::Of_Gate_Run`] names them, since both read the one check outcome that carries
    /// them.
    #[must_use]
    pub fn Of_Check_Run(root: &Path, outcome: &CheckOutcome) -> Self
    {
        let results = Check_Run_Results(root, outcome);

        return Self::Over(SarifRun::Of(results, SarifInvocation::Of_Check_Run(outcome), RunProperties::Of(outcome)));
    }

    /// This log as the JSON text a consumer ingests, indented for a person who opens the file.
    ///
    /// Serialized here rather than by each host, because this crate already carries
    /// `serde_json` in production and a host that must add a serializer to emit the log is the
    /// cost `OD-HOST-017` decision 4 names and this placement removes: `nomos-cli` writes what
    /// this returns and declares no serializer of its own.
    ///
    /// # Errors
    ///
    /// Whatever `serde_json` refuses. A derived `Serialize` over owned strings, integers and
    /// maps keyed by string has nothing to refuse, so this does not err on any log the two
    /// projections above build; it returns the error rather than assuming that.
    pub fn Serialized(&self) -> Result<String, serde_json::Error>
    {
        return serde_json::to_string_pretty(self);
    }

    /// A log carrying `run` and the two required properties of the document itself.
    fn Over(run: SarifRun) -> Self
    {
        return Self { version: SARIF_VERSION, schema: SARIF_SCHEMA, runs: vec![run] };
    }
}

/// Every finding a gate run reduced, each in the bucket the run placed it in, in canonical
/// order.
fn Gate_Run_Results(root: &Path, findings: &GateFindings) -> Vec<SarifResult>
{
    let mut results = Vec::new();

    for (bucket, group) in Bucketed_Findings(findings)
    {
        for finding in group
        {
            results.push(SarifResult::Of(root, finding, Some(bucket)));
        }
    }

    return Canonically_Ordered(results);
}

/// Every one of a gate run's finding groups, each beside the bucket it is.
///
/// The groups are read by destructuring [`GateFindings`] whole, with no `..`, and that is the
/// guard: a group added to the struct is a compile error here until it is paired with its
/// bucket, so a new bucket's findings cannot silently stop reaching a consumer. The fixed
/// length alone was never that guard -- it checks this list against its own constant, which
/// is how `below_evidence_floor_findings` was once left out with nothing noticing. The two
/// fields that are not finding groups are named and set aside for the same reason, so adding a
/// third is a question this function is made to answer rather than one it skips.
pub(crate) fn Bucketed_Findings(findings: &GateFindings) -> [(FindingDisposition, &[Finding]); GATE_RUN_BUCKETS]
{
    let GateFindings {
        blocking_findings,
        calibrated_findings,
        suppressed_findings,
        baselined_findings,
        baseline_exceeded_findings,
        below_evidence_floor_findings,
        baseline_populations: _,
        suppression_reasons: _,
    } = findings;

    return [
        (FindingDisposition::Blocking, blocking_findings),
        (FindingDisposition::Calibrated, calibrated_findings),
        (FindingDisposition::Suppressed, suppressed_findings),
        (FindingDisposition::Baselined, baselined_findings),
        (FindingDisposition::BaselineExceeded, baseline_exceeded_findings),
        (FindingDisposition::BelowEvidenceFloor, below_evidence_floor_findings),
    ];
}

/// Every finding a check run judged, in canonical order, and nothing at all for a run that
/// judged nothing -- what stopped it short is [`SarifInvocation::Of_Check_Run`]'s answer, and
/// a second empty list here would not be a second statement of it.
fn Check_Run_Results(root: &Path, outcome: &CheckOutcome) -> Vec<SarifResult>
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Vec::new();
    };

    let projected = findings.iter().map(|finding| return SarifResult::Of(root, finding, None)).collect();

    return Canonically_Ordered(projected);
}

/// `results` in the order [`SarifResult::Canonical_Key`] defines, which is what makes the
/// emitted document a function of what was found rather than of the order it was found in.
fn Canonically_Ordered(mut results: Vec<SarifResult>) -> Vec<SarifResult>
{
    results.sort_by_key(SarifResult::Canonical_Key);

    return results;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings, Finding_At, Judged_Outcome_With};
    use crate::NoVerdict;
    use nomos_check_orchestration::Claim;
    use nomos_contracts::GateCategory;

    /// The root every fixture below projects relative to.
    const FIXTURE_ROOT: &str = ".";

    #[test]
    fn Test_A_Projected_Log_Should_Carry_The_Specifications_Required_Document_Properties()
    {
        let rendered = Rendered(&SarifLog::Of_Gate_Run(&Complete_Gate_Result(Empty_Gate_Findings())));

        assert_eq!(rendered.pointer("/version").and_then(serde_json::Value::as_str), Some("2.1.0"), "{rendered}");
        assert!(rendered.pointer("/$schema").and_then(serde_json::Value::as_str).is_some_and(|schema| return schema.contains("sarif-schema-2.1.0")), "{rendered}");
        assert_eq!(rendered.pointer("/runs").and_then(serde_json::Value::as_array).map(Vec::len), Some(1), "{rendered}");
    }

    /// The whole required shape a consumer navigates, in one assertion per property the item
    /// this file was written for names: the driver and its rules, a result's rule, level,
    /// message and physical location, and the invocation.
    #[test]
    fn Test_A_Projected_Log_Should_Carry_Every_Required_Property_Of_A_Run_And_Its_Results()
    {
        let rendered = Rendered(&SarifLog::Of_Gate_Run(&One_Blocking_Finding_Response()));

        assert_eq!(rendered.pointer("/runs/0/tool/driver/name").and_then(serde_json::Value::as_str), Some("nomos"), "{rendered}");
        assert!(rendered.pointer("/runs/0/tool/driver/version").and_then(serde_json::Value::as_str).is_some(), "{rendered}");
        assert!(rendered.pointer("/runs/0/tool/driver/rules/0/id").and_then(serde_json::Value::as_str).is_some(), "{rendered}");
        assert_eq!(rendered.pointer("/runs/0/results/0/ruleId").and_then(serde_json::Value::as_str), Some("a-rule"), "{rendered}");
        assert_eq!(rendered.pointer("/runs/0/results/0/level").and_then(serde_json::Value::as_str), Some("error"), "{rendered}");
        assert!(rendered.pointer("/runs/0/results/0/message/text").and_then(serde_json::Value::as_str).is_some(), "{rendered}");
        assert_eq!(
            rendered.pointer("/runs/0/results/0/locations/0/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str),
            Some("src/lib.rs"),
            "{rendered}"
        );
        assert_eq!(
            rendered.pointer("/runs/0/results/0/locations/0/physicalLocation/region/startLine").and_then(serde_json::Value::as_u64),
            Some(u64::from(BLOCKING_LINE)),
            "{rendered}"
        );
        assert_eq!(rendered.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{rendered}");
    }

    /// Every rule a result cites is one of the driver's own `rules`, which is the invariant a
    /// consumer resolving a `ruleId` depends on and the only thing that makes the descriptor
    /// list worth emitting.
    #[test]
    fn Test_Every_Results_Rule_Should_Resolve_To_A_Reporting_Descriptor()
    {
        let log = SarifLog::Of_Gate_Run(&One_Blocking_Finding_Response());

        let run = log.runs.first().expect("a log carries exactly one run");
        let listed: Vec<&str> = run.tool.driver.rules.iter().map(|rule| return rule.id.as_str()).collect();
        assert!(!run.results.is_empty(), "{run:?}");
        assert!(run.results.iter().all(|result| return listed.contains(&result.Rule_Id())), "{run:?}");
    }

    /// The item's own prohibition, as a test: a policy that answered a finding must leave that
    /// finding in the log. The falsifier for emitting only what blocks -- a projection that
    /// dropped the five non-blocking buckets would report one result here instead of six.
    #[test]
    fn Test_A_Finding_A_Policy_Answered_Should_Be_Emitted_Rather_Than_Dropped()
    {
        let rendered = Rendered(&SarifLog::Of_Gate_Run(&One_Finding_Per_Bucket_Response()));

        let results = rendered.pointer("/runs/0/results").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert_eq!(results.len(), GATE_RUN_BUCKETS, "{rendered}");
        for (bucket, suppressions) in [
            ("blocking", 0),
            ("calibrated", 1),
            ("suppressed", 1),
            ("baselined", 1),
            ("baseline_exceeded", 0),
            ("below_evidence_floor", 1),
        ]
        {
            let result = results
                .iter()
                .find(|result| return result.pointer("/properties/bucket").and_then(serde_json::Value::as_str) == Some(bucket))
                .unwrap_or_else(|| panic!("{bucket} must reach the log: {rendered}"));

            assert_eq!(result.pointer("/level").and_then(serde_json::Value::as_str), Some("error"), "{bucket}: {result}");
            assert_eq!(result.pointer("/suppressions").and_then(serde_json::Value::as_array).map(Vec::len), Some(suppressions), "{bucket}: {result}");
        }
    }

    /// An advisory finding is a `warning` and a finding behind a gate nothing stands under is a
    /// `note`, beside the `error` the test above pins -- the three levels the item names, over
    /// one projection rather than over `SarifLevel::Of` alone.
    #[test]
    fn Test_A_Projected_Log_Should_Carry_Each_Of_The_Three_Levels_For_The_Finding_That_Deserves_It()
    {
        for (gate, level) in [(GateCategory::Blocking, "error"), (GateCategory::Advisory, "warning"), (GateCategory::Review, "note")]
        {
            let mut finding = Finding_At("a-rule", BLOCKING_LOCATION);
            finding.gate = gate;
            let mut findings = Empty_Gate_Findings();
            findings.blocking_findings.push(finding);

            let rendered = Rendered(&SarifLog::Of_Gate_Run(&Complete_Gate_Result(findings)));

            assert_eq!(rendered.pointer("/runs/0/results/0/level").and_then(serde_json::Value::as_str), Some(level), "{gate}: {rendered}");
        }
    }

    /// The item's other prohibition: a run that reached no verdict is never a clean log. The
    /// falsifier for the invocation half of the projection -- with `executionSuccessful`
    /// hard-coded true, a coverage-floor refusal would reach a consumer as a pass with no
    /// results.
    #[test]
    fn Test_A_Run_That_Reached_No_Verdict_Should_Be_Unsuccessful_And_Say_Why()
    {
        let mut response = Complete_Gate_Result(Empty_Gate_Findings());
        response.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let rendered = Rendered(&SarifLog::Of_Gate_Run(&response));

        assert_eq!(rendered.pointer("/runs/0/results").and_then(serde_json::Value::as_array).map(Vec::len), Some(0), "{rendered}");
        assert_eq!(rendered.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{rendered}");
        assert!(
            rendered
                .pointer("/runs/0/invocations/0/toolExecutionNotifications/0/message/text")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|text| return text.contains("require-completeness")),
            "{rendered}"
        );
    }

    /// A tree nothing could be judged over is the same: unsuccessful, with a reason, and not an
    /// empty clean log.
    #[test]
    fn Test_A_Run_That_Judged_Nothing_Should_Be_Unsuccessful_And_Say_Why()
    {
        let mut response = Complete_Gate_Result(Empty_Gate_Findings());
        response.check_outcome = CheckOutcome::Unreadable;

        let rendered = Rendered(&SarifLog::Of_Gate_Run(&response));

        assert_eq!(rendered.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{rendered}");
        assert_eq!(
            rendered.pointer("/runs/0/invocations/0/toolExecutionNotifications").and_then(serde_json::Value::as_array).map(Vec::len),
            Some(1),
            "{rendered}"
        );
    }

    /// Determinism, proven the way the item asks: the same response serialized twice is the
    /// same bytes. `HashMap` iteration order is the usual way a projection stops being one, so
    /// the fingerprints are a `BTreeMap` and the rules and the results are both sorted.
    #[test]
    fn Test_Projecting_One_Response_Twice_Should_Produce_Byte_Identical_Documents()
    {
        let response = One_Finding_Per_Bucket_Response();

        let first = serde_json::to_string(&SarifLog::Of_Gate_Run(&response)).expect("a derived Serialize over owned data has nothing to refuse");
        let second = serde_json::to_string(&SarifLog::Of_Gate_Run(&response)).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(first, second);
    }

    /// The falsifier for the canonical ordering: two responses carrying the same findings in
    /// different orders project to the same document. Without the sort the two differ, and a
    /// consumer diffing two logs of one tree would read a reordering as a change.
    #[test]
    fn Test_Two_Responses_Differing_Only_In_Finding_Order_Should_Project_Identically()
    {
        let mut one_way = Empty_Gate_Findings();
        one_way.blocking_findings.push(Finding_At("b-rule", BLOCKING_LOCATION));
        one_way.blocking_findings.push(Finding_At("a-rule", BLOCKING_LOCATION));
        let mut other_way = Empty_Gate_Findings();
        other_way.blocking_findings.push(Finding_At("a-rule", BLOCKING_LOCATION));
        other_way.blocking_findings.push(Finding_At("b-rule", BLOCKING_LOCATION));

        let first = serde_json::to_string(&SarifLog::Of_Gate_Run(&Complete_Gate_Result(one_way))).expect("a derived Serialize has nothing to refuse");
        let second = serde_json::to_string(&SarifLog::Of_Gate_Run(&Complete_Gate_Result(other_way))).expect("a derived Serialize has nothing to refuse");

        assert_eq!(first, second);
    }

    /// A check run projects too, and its results carry no bucket, because no policy was applied
    /// and there is nothing to report one from.
    #[test]
    fn Test_A_Check_Run_Should_Project_Its_Findings_With_No_Bucket()
    {
        let judged = Judged_Outcome_With(vec![Finding_At("a-rule", BLOCKING_LOCATION)], Claim::Complete);

        let rendered = Rendered(&SarifLog::Of_Check_Run(Path::new(FIXTURE_ROOT), &judged));

        assert_eq!(rendered.pointer("/runs/0/results/0/ruleId").and_then(serde_json::Value::as_str), Some("a-rule"), "{rendered}");
        assert!(rendered.pointer("/runs/0/results/0/properties/bucket").is_none(), "{rendered}");
        assert_eq!(rendered.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(true), "{rendered}");
    }

    /// A check run that judged nothing carries no results and an unsuccessful invocation --
    /// the same "an empty answer and an unexamined one must never render the same" the gate
    /// half is held to.
    #[test]
    fn Test_A_Check_Run_That_Judged_Nothing_Should_Be_Unsuccessful_With_No_Results()
    {
        let rendered = Rendered(&SarifLog::Of_Check_Run(Path::new(FIXTURE_ROOT), &CheckOutcome::NoSource));

        assert_eq!(rendered.pointer("/runs/0/results").and_then(serde_json::Value::as_array).map(Vec::len), Some(0), "{rendered}");
        assert_eq!(rendered.pointer("/runs/0/invocations/0/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{rendered}");
    }

    /// End to end through the gate a host actually calls, over a real tree with one real
    /// blocking finding -- not a constructed result. A projection that is correct over a value
    /// nobody produces proves nothing about the path `nomos gate run --sarif` takes.
    ///
    /// Scoped to the completeness-mirror rule the fixture is written against, as this crate's
    /// comparison tests are, so the run judges what the assertion is about and loads no
    /// compiler-backed family for a two-line file.
    #[test]
    fn Test_A_Real_Gate_Run_Should_Project_To_A_Log_Naming_The_File_It_Judged()
    {
        let root = std::env::temp_dir().join(format!("nomos-gate-sarif-real-run-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the temp directory is writable");
        let text = "/// A list.\n/// Mirrored by `Test_Sarif_Ghost`.\npub const TABLES: &[&str] = &[];\n";
        std::fs::write(root.join("a.rs"), text).expect("writes a fixture whose stale mirror is one real blocking finding");

        let command = crate::GateCommand {
            root: root.clone(),
            rules: crate::RuleSelector { include: vec![nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] },
            ..crate::GateCommand::default()
        };
        let source = nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), text.to_owned());
        let providers = nomos_composer_providers::Standard_Providers();
        let environment = crate::GateEnvironment {
            variant: nomos_workspace::BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>()),
            launcher: &nomos_platform_std::StdProgramLauncher,
            filesystem: &nomos_platform_std::StdFileSystem,
            environment: &nomos_platform_std::StdEnvironment,
            now: nomos_platform::Timestamp::From_Unix_Seconds(0),
            providers: &providers,
        };
        let result = crate::Run_Gate(Some(vec![source]), environment, &command, crate::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(0)));
        let rendered = Rendered(&SarifLog::Of_Gate_Run(&result));

        let _ignored = std::fs::remove_dir_all(&root);
        let results = rendered.pointer("/runs/0/results").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        assert!(
            results.iter().any(|result| {
                return result.pointer("/locations/0/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str) == Some("a.rs");
            }),
            "{rendered}"
        );
    }

    /// What the log serializes to is the document a consumer reads, and reading it back gives
    /// the value this module's own tests assert on -- so the text a host writes and the value
    /// the tests pin are one document, not two.
    #[test]
    fn Test_Serialized_Should_Be_The_Document_The_Tests_Assert_On()
    {
        let log = SarifLog::Of_Gate_Run(&One_Finding_Per_Bucket_Response());

        let text = log.Serialized().expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(serde_json::from_str::<serde_json::Value>(&text).ok(), Some(Rendered(&log)));
    }

    /// The line the fixture locations below carry.
    const BLOCKING_LINE: u32 = 12;

    /// The location every constructed fixture finding reports, root-relative already.
    const BLOCKING_LOCATION: &str = "src/lib.rs:12";

    /// A complete judged gate run carrying one blocking finding.
    fn One_Blocking_Finding_Response() -> GateRunResult
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At("a-rule", BLOCKING_LOCATION));

        return Complete_Gate_Result(findings);
    }

    /// A complete judged gate run carrying one finding in each of the six buckets, each from a
    /// rule of its own so the six reach the log as six distinct results.
    fn One_Finding_Per_Bucket_Response() -> GateRunResult
    {
        let mut findings = Empty_Gate_Findings();
        findings.blocking_findings.push(Finding_At("a-rule", BLOCKING_LOCATION));
        findings.calibrated_findings.push(Finding_At("b-rule", BLOCKING_LOCATION));
        findings.suppressed_findings.push(Finding_At("c-rule", BLOCKING_LOCATION));
        findings.baselined_findings.push(Finding_At("d-rule", BLOCKING_LOCATION));
        findings.baseline_exceeded_findings.push(Finding_At("e-rule", BLOCKING_LOCATION));
        findings.below_evidence_floor_findings.push(Finding_At("f-rule", BLOCKING_LOCATION));

        return Complete_Gate_Result(findings);
    }

    /// `log` as the JSON a consumer receives.
    fn Rendered(log: &SarifLog) -> serde_json::Value
    {
        return serde_json::to_value(log).expect("a derived Serialize over owned data has nothing to refuse");
    }
}
