//! [`Handle_Gate_Explain`] and its own [`GateExplainResponse`], paired in one file the same
//! way [`super::gate_run_response`] pairs [`super::gate_run_response::Handle_Gate_Run`] with
//! [`super::gate_run_response::GateRunResponse`].

use crate::{composition, sources};
use nomos_contracts::Finding;
use nomos_gate_orchestration::{Explanation, FindingQuery, GateCommand};
use serde::Serialize;
use std::path::Path;

use super::{BaselineDebtResponse, RuleCalibrationResponse, SuppressionResponse};

/// Walks `root`, judges it exactly as `nomos gate run` would, and answers `query` against
/// what was judged -- exactly as `nomos gate explain` would -- and hands back a
/// JSON-serializable [`GateExplainResponse`].
///
/// The same walk-and-judge composition [`crate::response::gate_run_response::Handle_Gate_Run`] already
/// uses, over the default [`GateCommand`] narrowed only by `root`: `Explain_Gate` itself does
/// not consult `command.scope` or `command.rules`, by its own doc.
#[must_use]
pub fn Handle_Gate_Explain(root: &Path, query: &FindingQuery) -> GateExplainResponse
{
    use nomos_platform_std::StdProcessLauncher;

    let command = GateCommand { root: root.to_path_buf(), ..Default::default() };
    let walked = sources::Walked_Sources(root);
    let result = nomos_gate_orchestration::Explain_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment { variant: composition::Host_Variant(), launcher: &StdProcessLauncher },
        &command,
        query,
    );

    return GateExplainResponse::From(result.explanation);
}

/// What a real `nomos gate explain` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// Carries no `root` and no `check_outcome`: this crate's caller already supplied `root`,
/// and `GateRunResponse` already sets the precedent of dropping `check_outcome` from its own
/// wire shape entirely, rather than re-exposing `CheckOutcome` for a caller to reconstruct a
/// distinction `Explain_Gate` itself does not draw -- see [`Self::NotFound`]'s own doc for
/// the one it collapses.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum GateExplainResponse
{
    /// No finding from the query's rule at the query's location exists in this judgment.
    ///
    /// Two different causes collapse into this one value, by `Explain_Gate`'s own design
    /// (`crates/orchestration/nomos-gate-orchestration/src/explain.rs`'s `Explained`): a
    /// tree that was never judged at all, and a judged tree whose real findings simply do
    /// not include this rule and location. This type preserves that collapse rather than
    /// inventing a finer distinction the orchestration crate does not draw.
    NotFound,
    /// The finding the query names, and what it would do to a real run's disposition.
    Found
    {
        /// The finding itself, in full. `Finding` already derives `Serialize`. Boxed, the
        /// same reason `WorkShowResponse::Found::item` is: `NotFound` carries nothing, and
        /// an unboxed `Finding` here would size every `GateExplainResponse` to `Found`'s own
        /// width regardless of which variant it holds.
        finding: Box<Finding>,
        /// Whether this finding, on its own, could fail a build a real `run` reduces it
        /// into.
        would_block: bool,
        /// The calibration that kept it from blocking, when `would_block` is `false`
        /// because of one.
        calibrated_by: Option<RuleCalibrationResponse>,
        /// The suppression that kept it from blocking, when `would_block` is `false`,
        /// `calibrated_by` is `None`, and a suppression matched. Boxed: `SuppressionResponse`
        /// is the largest of the three calibration/suppression/baseline fields (it alone
        /// carries two owned `String`s beside `rule` and `subject`), so it is the one
        /// `clippy::large_enum_variant` names to shrink `Found`'s own width by.
        suppressed_by: Box<Option<SuppressionResponse>>,
        /// The baseline debt entry that kept it from blocking, when `would_block` is
        /// `false` and both `calibrated_by` and `suppressed_by` are `None`.
        baselined_by: Option<BaselineDebtResponse>,
        /// The governing record `finding.rule`'s implementation cites, and the version
        /// of that record it was written against -- `AGT-008`'s "rule version" clause.
        /// `None` only for a rule this build's registry does not hold; see
        /// [`nomos_gate_orchestration::Explanation::Found`]'s own doc for why the
        /// ordinary "no `CONTRACT_RECORD`" case is a real citation, not this.
        contract_record: Option<String>,
        /// The version of `contract_record` this rule's implementation was written
        /// against. Always `Some` exactly when `contract_record` is.
        contract_record_version: Option<u32>,
    },
}

impl GateExplainResponse
{
    pub(crate) fn From(explanation: Explanation) -> Self
    {
        return match explanation
        {
            Explanation::NotFound => Self::NotFound,
            Explanation::Found { finding, would_block, calibrated_by, suppressed_by, baselined_by, contract } =>
            {
                let (contract_record, contract_record_version) = match contract
                {
                    Some((record, version)) => (Some(record), Some(version)),
                    None => (None, None),
                };

                Self::Found {
                    finding,
                    would_block,
                    calibrated_by: calibrated_by.map(RuleCalibrationResponse::From),
                    suppressed_by: Box::new(suppressed_by.map(SuppressionResponse::From)),
                    baselined_by: baselined_by.map(BaselineDebtResponse::From),
                    contract_record,
                    contract_record_version,
                }
            }
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::RuleId;
    use nomos_rules::COMPLETENESS_MIRROR;

    /// The `COMPLETENESS_MIRROR` rule's contract record version, cited in
    /// `Test_Explaining_A_Real_Trigger_Should_Find_A_Real_Blocking_Finding`'s own assertion
    /// against `D-134`'s current version -- named so that assertion reads as a citation of a
    /// specific record version, not an unexplained number.
    const EXPECTED_CONTRACT_RECORD_VERSION: u32 = 2;

    /// A query naming a rule and location no finding carries is [`GateExplainResponse::
    /// NotFound`], not a panic or a default -- over a real walked directory, the same "an
    /// absent answer is a typed state, not a shorter one" discipline `crates/orchestration/
    /// nomos-gate-orchestration/src/tests.rs`'s own `Test_Explain_Should_Report_Not_Found_
    /// For_A_Query_Nothing_Answers` already proves at the orchestration layer.
    #[test]
    fn Test_Explaining_A_Query_Nothing_Answers_Should_Be_Not_Found()
    {
        let directory = Scratch_Source_Tree("not-found", "a.rs", "pub fn Ok() {}\n");
        let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "nowhere.rs".to_owned() };

        let response = Handle_Gate_Explain(&directory, &query);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert!(matches!(response, GateExplainResponse::NotFound), "{response:?}");
    }

    /// A real query naming the one blocking finding a real trigger produces answers `Found`,
    /// with `would_block` true and no calibration, suppression or baseline -- the same
    /// trigger content `crates/orchestration/nomos-gate-orchestration/src/tests.rs`'s own
    /// `Test_Explain_Should_Find_A_Real_Blocking_Finding` fixture uses, walked from a real
    /// directory by this crate's own `sources::Walked_Sources` rather than handed to `Explain_Gate`
    /// as a synthetic `SourceFile` list.
    #[test]
    fn Test_Explaining_A_Real_Trigger_Should_Find_A_Real_Blocking_Finding()
    {
        let directory =
            Scratch_Source_Tree("found", "a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
        let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };

        let response = Handle_Gate_Explain(&directory, &query);

        let _ignored = std::fs::remove_dir_all(&directory);

        let GateExplainResponse::Found {
            would_block,
            calibrated_by,
            suppressed_by,
            baselined_by,
            contract_record,
            contract_record_version,
            ..
        } = response
        else
        {
            // This fixture's trigger content is the same one the orchestration layer's own
            // test proves produces a real blocking finding, so landing on `NotFound` here
            // means the fixture itself regressed, not a condition this test should recover
            // from silently.
            panic!("this fixture must produce the finding the query names");
        };
        assert!(would_block);
        assert!(calibrated_by.is_none());
        assert!(suppressed_by.is_none());
        assert!(baselined_by.is_none());
        assert_eq!(contract_record.as_deref(), Some("D-134"));
        assert_eq!(contract_record_version, Some(EXPECTED_CONTRACT_RECORD_VERSION));
    }

    /// The response a real `Found` explanation produces is valid JSON, and its outcome
    /// round-trips through `serde_json` under the field name a wire caller would actually
    /// read.
    #[test]
    fn Test_A_Real_Found_Explanation_Should_Round_Trip_As_Json()
    {
        let directory =
            Scratch_Source_Tree("json", "a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
        let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };

        let response = Handle_Gate_Explain(&directory, &query);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a GateExplainResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized GateExplainResponse always has this field");

        assert_eq!(outcome, "found", "{json}");
    }

    /// A real, freshly walkable scratch tree of this test's own -- never the real repository
    /// tree, which live sessions write to concurrently. A call-local counter, the same
    /// `crates/host/nomos-api/src/work.rs::tests::Unique_Scratch_Directory` fix: several
    /// tests above build a tree holding the same trigger content, and the default test
    /// runner's threads would otherwise race on one directory a bare pid gave them.
    fn Scratch_Source_Tree(label: &str, file_name: &str, content: &str) -> std::path::PathBuf
    {
        // scope: allow this test-only counter has no owner beyond disambiguating calls within
        // one process; a bare pid does not distinguish two calls in the same test run.
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let directory =
            std::env::temp_dir().join(format!("nomos-api-gate-explain-{label}-{}-{unique}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        std::fs::write(directory.join(file_name), content).expect("writes a real source file");

        return directory;
    }
}
