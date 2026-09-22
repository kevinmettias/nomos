//! [`Handle_Gate_Run`] and its own [`GateRunResponse`], paired in one file: the response type
//! exists only for this one handler, the same "handler beside its own response" locality every
//! other endpoint file in this crate keeps.

use crate::{composition, sources};
use nomos_contracts::RunId;
use nomos_gate_orchestration::{Effective_Policy_Report, GateCommand, GateRunResult};
use nomos_platform::Clock;
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use serde::Serialize;
use std::path::PathBuf;

use super::{CheckOutcomeResponse, Disposition, GateFindings, NoVerdictResponse};

/// Walks `command.root` and judges it exactly as `nomos gate run` would, and hands back a
/// JSON-serializable [`GateRunResponse`].
///
/// `command` travels straight through to [`nomos_gate_orchestration::Run_Gate`], so its
/// `scope` and `rules` selectors reach the same materialization-skipping `OD-GATE-017` gives
/// `nomos-cli`'s own `run` verb -- a caller that narrows either pays for only what it asked
/// to judge, not for every registered rule on every call.
#[must_use]
pub fn Handle_Gate_Run(command: &GateCommand) -> GateRunResponse
{
    let walked = sources::Walked_Sources(&command.root);
    let now = CLOCK.Now();
    let run = nomos_gate_orchestration::Fresh_Run_Id(now);
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        nomos_gate_orchestration::GateEnvironment {
            variant: composition::Host_Variant(),
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            now,
        },
        command,
        run,
    );

    return GateRunResponse::From(result);
}

/// What a real gate run over `root` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
pub struct GateRunResponse
{
    /// The identity of this execution. `RunId` already derives `Serialize` -- unlike
    /// [`nomos_gate_orchestration::GateRunOutcome`], it needs no local twin.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// The reduced verdict.
    pub disposition: Disposition,
    /// Every finding this run reduced, grouped by why it does or does not block.
    pub findings: GateFindings,
    /// The check behind this run, and what it managed to look at.
    ///
    /// A `disposition` of `indeterminate` is reached from four different non-judged outcomes
    /// and from a judged run whose gate policy could not be read, and until this field
    /// existed they all arrived here identically: one word, and four empty finding buckets.
    /// `nomos-cli` has never had that problem, because it matches on the check outcome
    /// rather than on the disposition -- its own `Render_Run` says so in its doc -- and the
    /// wire had no way to.
    pub check_outcome: CheckOutcomeResponse,
    /// Why a run that judged its tree still reached no verdict, when one did.
    ///
    /// `None` on every run that reached a real verdict, and `None` as well when nothing was
    /// judged at all -- `check_outcome` is the reason in that case, and this does not restate
    /// it. The split is `GateRunResult`'s own and is kept rather than flattened here: folding
    /// the two into one wire vocabulary would be a second encoding of a distinction the
    /// domain already draws.
    pub no_verdict: Option<NoVerdictResponse>,
    /// Every declared policy entry that matched no finding in this run, described as a reader
    /// would need to find it in the file that declares it.
    ///
    /// Not a failure, and it does not move the disposition: a policy legitimately outlives
    /// the finding it was written for. But never silent either, which is the whole of
    /// `OD-GATE-024` -- an author who writes an entry that matches nothing otherwise gets no
    /// error, no warning and no effect, and cannot tell a mis-spelling from a debt that has
    /// since been paid. `nomos-cli` has reported this under a heading of its own since that
    /// record; a headless caller was given nothing, which is the state the record was filed
    /// to end, left standing on the surface this product is meant to be run on.
    ///
    /// Empty whenever every declared entry matched, and empty when none was declared.
    pub unmatched_policy: Vec<String>,
    /// Which layer and artifact decided each field of the policy this run judged under, what
    /// that statement outranked, and any statement a lock refused with the reason.
    ///
    /// `OD-POLICY-001`'s provenance, reaching a headless caller. Every line here is
    /// [`nomos_gate_orchestration::Effective_Policy_Report`]'s own, which is the same
    /// function `nomos-cli`'s `gate policy` verb prints: this is a projection of that
    /// rendering and not a second assembly of it, so the two surfaces cannot come to disagree
    /// about what decided a field while each looks right on its own.
    ///
    /// Text rather than a typed entry, for the reason [`Self::unmatched_policy`] already
    /// gives: these are report lines. A caller that wants to match on a layer has the typed
    /// resolution one crate down; duplicating it on the wire would be a second addressing
    /// scheme for one question.
    ///
    /// Empty for a run whose resolution refused, which is the case
    /// [`Self::no_verdict`] already carries the reason for.
    pub effective_policy: Vec<String>,
}

impl GateRunResponse
{
    pub(crate) fn From(result: GateRunResult) -> Self
    {
        return Self {
            run: result.run,
            root: result.root,
            disposition: Disposition::From(result.disposition),
            check_outcome: CheckOutcomeResponse::From(&result.check_outcome),
            no_verdict: result.no_verdict.as_ref().map(NoVerdictResponse::From),
            unmatched_policy: result.unmatched_policy,
            effective_policy: result.policy.as_deref().map(Effective_Policy_Report).unwrap_or_default(),
            findings: GateFindings::From(result.findings),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Area, Probe_Tree, TreeName};
    use nomos_contracts::{ConfigurationLayer, RuleId};
    use nomos_gate_orchestration::{CoveragePolicy, Effective_Gate_Policy, PolicyContribution, RuleSelector};
    use nomos_rules::NAMING_CONVENTION;

    /// The subsystem this module's own probe trees are named under.
    const GATE_RUN_AREA: Area = Area("gate-run");

    /// A real run over this crate's own tree reaches a real judgment -- not
    /// [`Disposition::Indeterminate`], the state a walk that never became a judged check
    /// outcome carries -- proving this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Gate_Run_And_Walked_Sources_Should_Reach_A_Judgment_Over_A_Real_Tree()
    {
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));

        assert_ne!(
            response.disposition,
            Disposition::Indeterminate,
            "this crate's own source tree has real .rs files to judge, so the check behind \
             this run must have reached Judged"
        );
    }

    /// An empty tree cannot be judged, the same distinction the check-orchestration layer's
    /// own `CheckOutcome::NoSource` already keeps apart from a clean judged run.
    #[test]
    fn Test_Host_Variant_Should_Compose_Into_A_Working_Environment_For_An_Empty_Tree()
    {
        let empty = std::env::temp_dir().join("nomos-api-gate-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let response = Handle_Gate_Run(&Command_At(empty.clone()));

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_eq!(response.disposition, Disposition::Indeterminate);
        assert!(response.findings.blocking_findings.is_empty());
        // Which indeterminate, not merely that it is one. Before this crate carried the
        // check outcome, this assertion could not be written here at all.
        let rendered = serde_json::to_value(&response).expect("a derived Serialize over owned data has nothing to refuse");
        assert_eq!(Field_At(&rendered, &["check_outcome", "outcome"]), "no_source", "{rendered}");
    }

    /// A tree with real source reports what it looked at, which is the other half of telling
    /// an empty answer apart from an unexamined one.
    #[test]
    fn Test_A_Judged_Run_Should_Report_What_It_Looked_At()
    {
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));

        let rendered = serde_json::to_value(&response).expect("a derived Serialize over owned data has nothing to refuse");
        assert_eq!(Field_At(&rendered, &["check_outcome", "outcome"]), "judged", "{rendered}");
        assert!(Field_At(&rendered, &["check_outcome", "files"]).as_u64().unwrap_or(0) > 0, "{rendered}");
    }

    /// A run that reached a verdict names no reason for not having one.
    #[test]
    fn Test_A_Run_That_Reached_A_Verdict_Should_Carry_No_Reason_For_Not_Having_One()
    {
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));

        assert!(response.no_verdict.is_none(), "{:?}", response.no_verdict);
    }

    /// The defect this item exists for, end to end through the handler a transport calls: a
    /// repository with one mis-spelled key in its `nomos-gate.json` gets back the key.
    ///
    /// Driven through `Handle_Gate_Run` rather than over a constructed `GateRunResult`,
    /// because a conversion that is correct over a value nobody produced proves nothing about
    /// the path a caller actually takes.
    #[test]
    fn Test_A_Malformed_Policy_Should_Reach_A_Headless_Caller_With_The_Key_It_Refused()
    {
        let root = Probe_Tree(GATE_RUN_AREA, TreeName("malformed"), r#"{ "basline": [] }"#)
            .expect("the temp directory is writable and this call's own name is fresh");

        let response = Handle_Gate_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        let rendered = serde_json::to_value(&response).expect("a derived Serialize over owned data has nothing to refuse");
        assert_eq!(Field_At(&rendered, &["disposition"]), "indeterminate", "{rendered}");
        // Judged, and still without a verdict -- the combination that was indistinguishable
        // from an unwalkable tree until both fields were carried.
        assert_eq!(Field_At(&rendered, &["check_outcome", "outcome"]), "judged", "{rendered}");
        assert_eq!(Field_At(&rendered, &["no_verdict", "cause"]), "malformed_policy", "{rendered}");
        let detail = Field_At(&rendered, &["no_verdict", "detail"]).as_str().unwrap_or_default().to_owned();
        assert!(detail.contains("basline"), "{detail}");
    }

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent, so these tests read
    /// through `get` and report a missing key as `Null` rather than as a panic inside an
    /// assertion.
    fn Field_At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    /// A declared entry that matches nothing reaches a headless caller, which is `OD-GATE-024`
    /// arriving on the surface this product is run on.
    #[test]
    fn Test_A_Declared_Entry_That_Matched_Nothing_Should_Reach_A_Headless_Caller()
    {
        let policy = "{ \"baseline\": [ { \"rule\": \"a-rule-no-registry-holds\", \"path\": \"src/lib.rs\", \"rationale\": \"tracked\" } ] }";
        let root = Probe_Tree(GATE_RUN_AREA, TreeName("unmatched"), policy)
            .expect("the temp directory is writable and this call's own name is fresh");

        let response = Handle_Gate_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(
            response.unmatched_policy.iter().any(|entry| return entry.contains("a-rule-no-registry-holds")),
            "{:?}",
            response.unmatched_policy
        );
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its disposition round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = Handle_Gate_Run(&Command_At(PathBuf::from(".")));
        let expected = match response.disposition
        {
            Disposition::Passed => "passed",
            Disposition::Failed => "failed",
            Disposition::Indeterminate => "indeterminate",
        };

        let json = serde_json::to_string(&response).expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let disposition =
            parsed.get("disposition").expect("an internally tagged enum writes its tag under this field name");

        assert_eq!(disposition, expected, "{json}");
    }

    /// A [`GateCommand::rules`] selection reaches `Run_Gate` through this crate's real entry
    /// point, not only the response built from it: excluding the rule behind this fixture's
    /// one real finding turns a Failed run into a Passed one with no blocking findings at
    /// all, the same "the deselected rule was never asked to run" property
    /// `nomos_gate_orchestration`'s own `Test_A_Deselected_Rules_Finding_Should_Not_Exist`
    /// proves one layer down.
    #[test]
    fn Test_A_Rule_Selection_Should_Reach_Run_Gate_Through_This_Crates_Entry_Point()
    {
        let root = std::env::temp_dir().join("nomos-api-gate-run-rule-selection");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(root.join("a.rs"), "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n")
            .expect("writes a fixture whose stale mirror is one real blocking finding");

        let every_rule = Handle_Gate_Run(&Command_At(root.clone()));
        let naming_only = Handle_Gate_Run(&GateCommand {
            rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
            ..Command_At(root.clone())
        });

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(every_rule.disposition, Disposition::Failed, "{every_rule:?}");
        assert_eq!(naming_only.disposition, Disposition::Passed, "{naming_only:?}");
        assert!(
            naming_only.findings.blocking_findings.is_empty(),
            "the deselected rule's finding must not exist at all: {naming_only:?}"
        );
    }

    /// The artifact a policy a caller built in code is reported under, as
    /// `nomos_gate_orchestration`'s own resolver names it.
    ///
    /// Spelled here because a `GateCommand` contributes at `CommandLine` whether or not it
    /// states anything, so the resolution behind the assertion below has two contributions
    /// and not one. A change to that name fails this test, which is the right place to find
    /// out that the wire's wording moved.
    const CALLER_BUILT_POLICY: &str = "the policy the caller built";

    /// `OD-POLICY-001`'s provenance reaching a headless caller, and reaching it as the same
    /// lines `nomos-cli`'s `gate policy` verb prints.
    ///
    /// Compared against `Effective_Policy_Report` over the resolution this tree declares, not
    /// merely searched for the file's name: the claim is that this response *projects* that
    /// rendering rather than assembling one, and a response that built its own sentences
    /// naming the same artifact would pass any weaker assertion.
    #[test]
    fn Test_A_Headless_Caller_Should_Be_Told_What_Decided_Each_Field_In_The_Rendering_Cli_Prints()
    {
        let root = Probe_Tree(GATE_RUN_AREA, TreeName("effective-policy"), r#"{ "coverage": "require-completeness" }"#)
            .expect("the temp directory is writable and this call's own name is fresh");

        let response = Handle_Gate_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(response.effective_policy, Declared_Coverage_Report(), "{:?}", response.effective_policy);
    }

    /// The report a tree declaring only a coverage floor resolves to, rendered by the one
    /// function both hosts read it through.
    fn Declared_Coverage_Report() -> Vec<String>
    {
        let declared = PolicyContribution {
            coverage: Some(CoveragePolicy::RequireCompleteness),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, "nomos-gate.json")
        };
        let stated = PolicyContribution::Silent(ConfigurationLayer::CommandLine, CALLER_BUILT_POLICY);
        let policy = Effective_Gate_Policy(&[declared, stated]).expect("one stated field at one layer is not refused");

        return Effective_Policy_Report(&policy);
    }

    /// [`GateCommand`] over `root`, every selector at its select-everything default -- the
    /// shape every test here builds before narrowing one field of its own.
    fn Command_At(root: PathBuf) -> GateCommand
    {
        return GateCommand { root, ..Default::default() };
    }

}
