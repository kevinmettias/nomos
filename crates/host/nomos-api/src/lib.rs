//! Band 90 — host. A second real caller of orchestration seams `nomos-cli` otherwise has
//! to itself.
//!
//! `nomos-gate-orchestration`'s own module doc, since `P13-GATE-RUN-SEAM-CRATE`, states the
//! point directly: `Run_Gate` is generic over `nomos-platform`'s traits "so a second adapter
//! can call it without depending on `nomos-cli`." Until this crate, nothing did --
//! `nomos-cli`'s own `gate.rs` was the only caller outside the orchestration crate itself,
//! apart from `nomos-ledger`'s own unrelated `Run_Gate_Step` (a lint-argv runner that merely
//! shares a name fragment, `OD-WORKFLOW-001`'s own correction). `ARC-ROADMAP-001` names
//! `CLI / API / MCP projections` as a near-term-tier item next to the Gate object itself;
//! this crate's first increment was `Handle_Gate_Run`. Its second, [`work::Handle_Work_List`],
//! proves the same is true of `nomos_work_orchestration::Run`: grepped directly, before that
//! increment its only real caller anywhere in this workspace was `nomos-cli`'s own `work.rs`.
//! Its third, [`spec::Handle_Spec_Profiles`], does the same for `nomos_spec_orchestration::
//! Profiles` -- the simplest of that crate's nine `SpecCommand` verbs plus `Submit`, all ten
//! of which had exactly one real caller, `nomos-cli`, before this increment. Its fourth,
//! [`work::Handle_Work_Show`], does the same for `WorkCommand::Show` -- the lookup by id
//! stays at the caller, exactly where `nomos-cli`'s own `Render_Show` already puts it, rather
//! than being invented inside `nomos_work_orchestration` for this increment's convenience.
//! Its fifth, [`work::Handle_Work_Validate`], does the same for `WorkCommand::Validate` -- a
//! real internal-consistency check, not a second view onto data `List` already exposes. Its
//! sixth, [`work::Handle_Work_Audit`], does the same for `WorkCommand::Audit` -- whose own
//! `Run` arm returns the identical `Result<BoardView, LedgerError>` `List`'s arm does, so what
//! makes this seam an audit and not a second listing is a caller-side filter over the same
//! canonical `nomos_ledger::Claim_Refusal` `crates/host/nomos-cli/src/work/report.rs`'s own
//! `Blocking_Refusal` already calls, rather than a second implementation of that one rule.
//! Its seventh, [`Handle_Gate_Plan`], does the same for `nomos_gate_orchestration::Run` (the
//! `Plan` verb's own body, not this crate's `Run_Gate`) -- a function that takes a full
//! `GateCommand` but, by its own doc, reads none of it: `Plan` reports what this gate's rule
//! registry holds, not a walk over `root`. `Handle_Gate_Plan` takes no argument for that
//! reason, unlike [`Handle_Gate_Run`]. Its eighth, [`Handle_Gate_Explain`], does the same for
//! `Explain_Gate` -- the same walk-and-judge composition `Handle_Gate_Run` already uses, plus
//! a `FindingQuery` naming one finding to answer for. Its ninth, [`work::Handle_Work_Claim`]
//! (with [`work::Handle_Work_Renew`] and [`work::Handle_Work_TakeOver`] alongside it), does
//! the same for three of `WorkCommand`'s seven remaining mutation verbs -- one shared
//! response type across all three, because `nomos_work_orchestration::ClaimRequest` is
//! already one shared request type for them, by that type's own doc. Its tenth,
//! [`work::Handle_Work_Abandon`] (with [`work::Handle_Work_Decline`] alongside it), does the
//! same for two of `WorkCommand`'s four still-remaining verbs, `Abandon` and `Decline` --
//! kept to two response types rather than one, because `EndingRequest`'s own doc says the
//! opposite of `ClaimRequest`'s: `abandon` and `decline` share only their argument shape, not
//! what they mean. Its eleventh, [`work::Handle_Work_Finish`], does the same for `Finish` --
//! `nomos_work_orchestration::Run`'s own `Finish` arm always passes `working_directory: None`
//! to `nomos_ledger::Finish`, so the gate's own lint step resolves relative to the calling
//! process's own directory, not anything this crate's caller supplies. Its twelfth,
//! [`work::Handle_Work_Add`], does the same for `Add`, closing `WorkCommand` entirely -- the
//! one verb needing a real, walked `published` `Territory`, computed by a private
//! `Published_Records` this crate keeps to itself, a deliberate twin of `nomos-cli`'s own
//! function of the same name rather than a shared dependency of it, the same division
//! `sources.rs` already draws for Gate's own walk. Its thirteenth,
//! [`spec::Handle_Spec_Sources`], moves on to Spec's own remaining verbs: `Sources` is a
//! unit `SpecCommand` variant, the next-simplest of that crate's nine after `Profiles`, but
//! the first here to go through `nomos_spec_orchestration::corpus::Assemble` at all. Its
//! fourteenth, [`spec::Handle_Spec_Record`], is the first Spec seam carrying a request
//! payload of its own rather than a unit variant -- `RecordRequest{id, revision}` -- and the
//! first whose own outcome needed twin types of its own,
//! [`spec::DocumentSourceResponse`]/[`spec::NodeSummaryResponse`], because
//! `nomos_spec_store::DocumentSource`/`NodeSummary` do not derive `Serialize` and are not
//! this crate's own types to change. Its fifteenth, [`spec::Handle_Spec_Table`], does the
//! same for `Table`, reusing [`spec::DocumentSourceResponse`] and adding three more twins
//! for `PathMatch`, `RowCensus` and `TableLine`. Its sixteenth, [`spec::Handle_Spec_Markdown`],
//! does the same for `Markdown`, reusing `RecordRequest` and collapsing `EditError` -- a
//! nine-variant error with no `Serialize` but a real `Display` -- to a single `cause` string,
//! the same shape this crate's `StoreError` handling already uses. Its seventeenth,
//! [`spec::Handle_Spec_Freshness`], does the same for `Freshness` -- generic over
//! `nomos-platform`'s `FileSystem` like `Render`, `Preview` and `Commit`, but, unlike those
//! three, only ever reads through it, so it carries none of their "does a wire call write to
//! this host's disk" question. Its eighteenth, [`spec::Handle_Spec_Preview`], does the same
//! for `Preview` -- also generic over `FileSystem`, and also read-only: its one contact with
//! a filesystem is reading the staged `--from` file the caller already named, never a write.
//! Its nineteenth, [`spec::Handle_Spec_Render`], is the first Spec seam here that writes:
//! `run::render::Render` places a built projection's body and its sidecar under a
//! caller-named `into` root through `Replace_Atomically`, unconditionally overwriting what
//! was there. This is not a new category of risk for this crate -- [`Handle_Work_Claim`]
//! already writes a real ledger file at a caller-named directory over an unauthenticated
//! wire call -- and unlike `Commit`'s vacate step, a rendered projection is a derived,
//! regenerable artifact, not the governing record itself. Its twentieth,
//! [`spec::Handle_Spec_Commit`], closes `SpecCommand` entirely: it writes the committed
//! record's own bytes the same way `Render` writes a projection's, but unlike `Render` it
//! replaces the governing record itself, and a rename's old path is permanently unlinked via
//! a raw `std::fs::remove_file` outside `nomos-platform`'s own port -- `run::commit`'s own
//! documentation names the failure mode directly: "two files now declare this record."
//! Building both `Render` and `Commit` was an explicit choice, asked of and confirmed by a
//! person rather than decided here, once the research showed `Freshness` and `Preview` (both
//! generic over `FileSystem`) never actually write, narrowing the real decision to these two.
//!
//! [`Handle_Gate_Run`] is a deliberate twin of `nomos-cli`'s `gate.rs`
//! `GateInvocation::Run` arm: it walks `root` for `.rs` sources
//! ([`sources::Walked`] -- a twin of `crates/host/nomos-cli/src/gate/sources.rs`, since a
//! walk is a composition-root concern `OD-HOST-002` does not seam), reads this crate's own
//! build variant ([`composition::Host_Variant`] -- `env!` resolves against the crate that
//! calls it, so this cannot be shared either), and calls `Run_Gate` with the default
//! `GateCommand` every verb had before `OD-GATE-014`'s scope/rule selectors gave one a real
//! caller -- narrowing `root`'s own tree is this crate's only input today. What differs from
//! `nomos-cli` is the return: [`response::GateRunResponse`] instead of rendered text, because
//! this crate's reason to exist is a JSON-serializable answer a wire transport can hand back,
//! not a terminal one.
//!
//! # What this crate deliberately does not do
//!
//! It does not read argv, listen on a socket, or speak MCP's JSON-RPC framing. Building a
//! transport ahead of a real second caller for it is exactly the premature-surface pattern
//! this workspace has repeatedly declined to build ahead of evidence (`OD-PACKAGE-006`,
//! `OD-PACKAGE-008`, and `nomos-cli`'s own `gate.rs` doc on why `compare` is not stubbed). A
//! follow-up increment wires a real transport over [`Handle_Gate_Run`] once one exists to
//! design it against; this increment's job is only to prove the seam is reachable and
//! projectable from a second composition root at all.
//!
//! It does not select scope or rules either -- `GateCommand::scope` and `GateCommand::rules`
//! stay at their select-everything default, the same starting point every construction site
//! that predates `OD-GATE-014`'s selectors has. A caller that needs to narrow either is a
//! later increment to this crate's request shape, not a gap this one leaves silently.

mod composition;
mod response;
mod sources;
mod spec;
mod work;

pub use response::{
    BaselineDebtResponse, Disposition, GateExplainResponse, GatePlanResponse, GateRunResponse, RuleCalibrationResponse,
    RuleOfferResponse, SuppressionDispositionResponse, SuppressionResponse,
};
pub use spec::{
    AbsenceResponse, BlockChangeResponse, CommitReportResponse, CommittedPreviewResponse, DocumentSourceResponse,
    Handle_Spec_Commit, Handle_Spec_Freshness, Handle_Spec_Markdown, Handle_Spec_Preview, Handle_Spec_Profiles,
    Handle_Spec_Record, Handle_Spec_Render, Handle_Spec_Sources, Handle_Spec_Table, IdentityChangeResponse,
    NodeSummaryResponse, NormativeMovementResponse, NormativeOutcomeResponse, PathMatchResponse,
    ProfileOutcomeResponse, ProfilesResponse, RecordRelationResponse, ReproductionResponse, RowCensusResponse,
    SpecCommitResponse, SpecFreshnessResponse, SpecMarkdownResponse, SpecPreviewResponse, SpecRecordResponse,
    SpecRenderResponse, SpecSourcesResponse, SpecTableResponse, TableLineResponse, VacateOutcomeResponse,
    VacatedResponse, VerdictResponse,
};
pub use work::{
    BlockedItem, Handle_Work_Abandon, Handle_Work_Add, Handle_Work_Audit, Handle_Work_Claim, Handle_Work_Decline,
    Handle_Work_Finish, Handle_Work_List, Handle_Work_Renew, Handle_Work_Show, Handle_Work_TakeOver,
    Handle_Work_Validate, ReservationResponse, WorkAbandonResponse, WorkAddResponse, WorkAuditResponse,
    WorkDeclineResponse, WorkFinishResponse, WorkListResponse, WorkReservationResponse, WorkShowResponse,
    WorkValidateResponse,
};

use nomos_gate_orchestration::{FindingQuery, GateCommand};
use nomos_platform::Clock;
use nomos_platform_std::{StdProcessLauncher, SystemClock};
use std::path::Path;

/// Walks `root` and judges it exactly as `nomos gate run` would, over the default
/// [`GateCommand`] -- every rule, every file, no baseline, no suppression, no adoption
/// calibration -- and hands back a JSON-serializable [`GateRunResponse`].
#[must_use]
pub fn Handle_Gate_Run(root: &Path) -> GateRunResponse
{
    let command = GateCommand { root: root.to_path_buf(), ..Default::default() };
    let walked = sources::Walked(root);
    let run = nomos_gate_orchestration::Fresh_Run_Id(SystemClock.Now());
    let result = nomos_gate_orchestration::Run_Gate(
        walked,
        composition::Host_Variant(),
        &command,
        &StdProcessLauncher,
        run,
    );

    return GateRunResponse::From(result);
}

/// Composes this gate's rule registry and reports what it holds, exactly as `nomos gate
/// plan` would, and hands back a JSON-serializable [`GatePlanResponse`].
///
/// `nomos_gate_orchestration::Run` -- the `Plan` verb's own real body, not this crate's own
/// `Run_Gate` -- takes a full [`GateCommand`] but, by its own doc, reads none of it: `Plan`
/// reports what [`nomos_gate_orchestration::Registered`] holds, not a walk over `root`, so a
/// default command is built here rather than asking a caller to supply one nothing reads.
#[must_use]
pub fn Handle_Gate_Plan() -> GatePlanResponse
{
    let command = GateCommand::default();
    let outcome = nomos_gate_orchestration::Run(&command);

    return GatePlanResponse::From(outcome);
}

/// Walks `root`, judges it exactly as `nomos gate run` would, and answers `query` against
/// what was judged -- exactly as `nomos gate explain` would -- and hands back a
/// JSON-serializable [`GateExplainResponse`].
///
/// The same walk-and-judge composition [`Handle_Gate_Run`] already uses, over the default
/// [`GateCommand`] narrowed only by `root`: `Explain_Gate` itself does not consult
/// `command.scope` or `command.rules`, by its own doc.
#[must_use]
pub fn Handle_Gate_Explain(root: &Path, query: &FindingQuery) -> GateExplainResponse
{
    let command = GateCommand { root: root.to_path_buf(), ..Default::default() };
    let walked = sources::Walked(root);
    let result =
        nomos_gate_orchestration::Explain_Gate(walked, composition::Host_Variant(), &command, query, &StdProcessLauncher);

    return GateExplainResponse::From(result.explanation);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::RuleId;
    use nomos_rules::COMPLETENESS_MIRROR;

    /// A real run over this crate's own tree reaches a real judgment -- not
    /// [`Disposition::Indeterminate`], the state a walk that never became a judged check
    /// outcome carries -- proving this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_A_Real_Run_Should_Reach_A_Judgment()
    {
        let response = Handle_Gate_Run(Path::new("."));

        assert_ne!(
            response.disposition,
            Disposition::Indeterminate,
            "this crate's own source tree has real .rs files to judge, so the check behind \
             this run must have reached Judged"
        );
    }

    /// An empty tree cannot be judged, the same distinction `nomos_check_orchestration::
    /// CheckOutcome::NoSource` already keeps apart from a clean judged run.
    #[test]
    fn Test_An_Empty_Tree_Should_Be_Indeterminate()
    {
        let empty = std::env::temp_dir().join("nomos-api-gate-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let response = Handle_Gate_Run(&empty);

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_eq!(response.disposition, Disposition::Indeterminate);
        assert!(response.blocking_findings.is_empty());
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its disposition round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_A_Real_Runs_Response_Should_Round_Trip_As_Json()
    {
        let response = Handle_Gate_Run(Path::new("."));
        let expected = match response.disposition
        {
            Disposition::Passed => "passed",
            Disposition::Failed => "failed",
            Disposition::Indeterminate => "indeterminate",
        };

        let json = serde_json::to_string(&response).expect("a GateRunResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let disposition = parsed.get("disposition").expect("a serialized GateRunResponse always has this field");

        assert_eq!(disposition, expected, "{json}");
    }

    /// A real plan composes this workspace's own real rule registry, not an empty one --
    /// proving this crate, not `nomos-cli`, can produce a real `GatePlanResponse::Planned`.
    #[test]
    fn Test_A_Real_Plan_Should_Compose_This_Workspaces_Own_Registry()
    {
        let response = Handle_Gate_Plan();

        let GatePlanResponse::Planned { rules } = response
        else
        {
            panic!("this crate's own rule registry composes cleanly");
        };
        assert!(!rules.is_empty(), "this workspace ships real rules");
    }

    /// The response a real plan produces is valid JSON, and its outcome round-trips through
    /// `serde_json` under the field name a wire caller would actually read.
    #[test]
    fn Test_A_Real_Planned_Response_Should_Round_Trip_As_Json()
    {
        let response = Handle_Gate_Plan();

        let json = serde_json::to_string(&response).expect("a GatePlanResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized GatePlanResponse always has this field");

        assert_eq!(outcome, "planned", "{json}");
    }

    /// A real, freshly walkable scratch tree of this test's own -- never the real repository
    /// tree, which live sessions write to concurrently. A call-local counter, the same
    /// `crates/host/nomos-api/src/work.rs::tests::Unique_Scratch_Directory` fix: several
    /// tests below build a tree holding the same trigger content, and the default test
    /// runner's threads would otherwise race on one directory a bare pid gave them.
    fn Scratch_Source_Tree(label: &str, file_name: &str, content: &str) -> std::path::PathBuf
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let directory =
            std::env::temp_dir().join(format!("nomos-api-gate-explain-{label}-{}-{unique}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creates a scratch directory");
        std::fs::write(directory.join(file_name), content).expect("writes a real source file");

        return directory;
    }

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
    /// directory by this crate's own `sources::Walked` rather than handed to `Explain_Gate`
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
            panic!("this fixture must produce the finding the query names");
        };
        assert!(would_block);
        assert!(calibrated_by.is_none());
        assert!(suppressed_by.is_none());
        assert!(baselined_by.is_none());
        assert_eq!(contract_record.as_deref(), Some("D-134"));
        assert_eq!(contract_record_version, Some(2));
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
}
