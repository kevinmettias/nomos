//! Zone: Host. A second real caller of orchestration seams `nomos-cli` otherwise has
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
//! to `nomos_ledger::Finish_Item`, so the gate's own lint step resolves relative to the calling
//! process's own directory, not anything this crate's caller supplies. Its twelfth,
//! [`work::Handle_Work_Add`], does the same for `Add`, closing `WorkCommand` entirely -- the
//! one verb needing a real, walked `published` `Territory`, computed by a private
//! `Published_Records` this crate keeps to itself, a deliberate twin of `nomos-cli`'s own
//! function of the same name rather than a shared dependency of it, the same division
//! `sources.rs` already draws for Gate's own walk. Its thirteenth,
//! [`spec::Handle_Spec_Sources`], moves on to Spec's own remaining verbs: `Sources` is a
//! unit `SpecCommand` variant, the next-simplest of that crate's nine after `Profiles`, but
//! the first here to go through `nomos_spec_orchestration::corpus::Assemble_Corpus` at all. Its
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
//! `run::render::Rendered_Projection` places a built projection's body and its sidecar under a
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
//! With `Commit`, all nine `SpecCommand` verbs are seamed. Its twenty-first,
//! [`spec::Handle_Spec_Submit`], seams the one verb `SpecCommand` does not carry:
//! `nomos_spec_orchestration::Submit_Corpus_Request` is a sibling of `Run`, not one of its cases
//! (`OD-HOST-005`'s own resolution: a `nomos request submit` invocation is not a `nomos spec`
//! verb by the CLI's own naming), and takes an already-assembled `&mut Assembly` directly, so
//! this is the first `Handle_Spec_*` function here that calls `corpus::Assemble_Corpus` itself
//! rather than getting it from `Run`. Its twenty-second, [`correction::Handle_Correction_Run`],
//! is this crate's first correction verb: `P40-CORRECTIONS-CANONICAL-SEAM` moved correction
//! planning and lifecycle out of `nomos-cli`'s own `correct.rs` into
//! `nomos_correction_orchestration::Run_Correction`, a seam both hosts now call -- this
//! crate's own first real caller outside that orchestration crate's unit tests and
//! `nomos-cli`. Its twenty-third, [`check::Handle_Check_Run`], gives this crate its first
//! bare Check verb: before it, the only path from here into `nomos_check_orchestration::Run`
//! was transitively, through `Handle_Gate_Run`'s own call to `Run_Gate`, which always applies
//! `GateCommand`'s suppression, baseline and coverage policy on top of it -- there was no way
//! for a caller of this crate to run a policy-free Check the way `nomos-cli`'s own, separate
//! `nomos check` command already lets a person do. `P62-API-CHECK-SEAM` closes that gap with
//! the same twinned-response shape `correction.rs` already established. Its twenty-fourth,
//! [`workflow::Handle_Workflow_Run`], gives this crate its first Workflow verb:
//! `nomos_workflow_orchestration::Run` had exactly one real caller anywhere in this
//! workspace before it, `nomos-cli`'s own `workflow.rs`, kept to a single step per
//! invocation -- this dispatches the identical single-step shape, reusing
//! [`check::CheckResponse`], [`correction::CorrectionResponse`] and
//! [`response::GateRunResponse`] for the three step kinds this crate already twins, rather
//! than a fourth, divergent projection of the same outcomes. Its twenty-fifth,
//! [`agent::Handle_Agent_Execute`] (with [`agent::Handle_Agent_Judge_Role`] alongside it),
//! gives this crate its first Agent verbs: `P43-AGENT-CANONICAL-SEAM-2` moved `nomos agent`
//! `execute`/`judge-role`'s own dispatch out of `nomos-cli`'s own `agent.rs` and its
//! `agent/` directory into `nomos_agent_orchestration::Run_Agent_Execute`/
//! `Run_Agent_Judgment`, the only major verb family in this workspace that had reached this
//! point with no orchestration crate of its own -- this crate's own first real caller of
//! that seam outside its own unit tests and `nomos-cli`. [`agent::AgentDispatchResponse`]
//! is its own response type, not a reuse of [`workflow::AgentExecutionOutcomeResponse`]/
//! [`workflow::OllamaExecutionOutcomeResponse`]: see `agent.rs`'s own module doc for why.
//! Its twenty-sixth, [`Handle_Gate_Compare`], gives the Gate its fourth verb here and closes
//! the set `ARC-ROADMAP-001`'s constraint 5 names -- plan, run, explain and compare -- which
//! `nomos-cli` had served in full while this crate stopped at three, leaving the only way to
//! ask what changed between two runs a terminal. It re-derives both runs in one process
//! rather than looking either up, which is not a shortcut but `OD-GATE-022-A`'s own decision:
//! a compare caller builds no run-history store and serializes no `GateRunResult`, because
//! both cases the verb answers -- two trees under one policy, one tree under two -- are
//! same-process cases. Comparing against a run an earlier invocation produced is real,
//! deferred, and would need a persisted history keyed by `RunId` or a round-trippable twin.
//!
//! [`Handle_Gate_Run`] is a deliberate twin of `nomos-cli`'s `gate.rs`
//! `Invocation::Run` arm: it walks `command.root` for `.rs` sources
//! ([`sources::Walked_Sources`] -- a twin of `crates/host/nomos-cli/src/gate/sources.rs`, since a
//! walk is a composition-root concern `OD-HOST-002` does not seam), reads this crate's own
//! build variant ([`composition::Host_Variant`] -- `env!` resolves against the crate that
//! calls it, so this cannot be shared either), and calls `Run_Gate` with the caller's own
//! `GateCommand` whole -- `P40-API-GATE-SELECTION-SURFACE-2` closed the "select-everything
//! only" gap this doc used to name here, so a caller's `scope` and `rules` reach `Run_Gate`
//! exactly as `nomos-cli`'s own `run` verb already lets them. What differs from `nomos-cli`
//! is the return: [`response::GateRunResponse`] instead of rendered text, because this
//! crate's reason to exist is a JSON-serializable answer a wire transport can hand back, not
//! a terminal one.
//!
//! # What this crate deliberately does not do
//!
//! It does not read argv, listen on a socket, or speak MCP's JSON-RPC framing. Building a
//! transport ahead of a real second caller for it is exactly the premature-surface pattern
//! this workspace has repeatedly declined to build ahead of evidence (`OD-PACKAGE-006`,
//! `OD-PACKAGE-008`, and `nomos-cli`'s own `gate.rs`, where `compare` was refused as usage
//! under that same discipline until a real body arrived, and released when one did --
//! that file's own closing lines say the discipline is unchanged and the body is what
//! arrived, which makes it the stronger form of the precedent: one carried to its end
//! rather than one still waiting). That trigger has since fired here too.
//! `nomos-api-transport` is the real second caller this section once said a follow-up
//! increment would wire "once one exists to design it against": it serves
//! [`Handle_Gate_Run`] over a wire, dispatching to it directly from its own
//! `nomos_api_service.rs`. The boundary above is unmoved by that, which is why it still
//! stands word for word -- that crate's own module doc quotes this section's
//! argv/socket/framing sentence as the hole it fills, and says `nomos-api` gains no socket,
//! no protocol dependency and no new handler from it. Proving this seam reachable and
//! projectable from a second composition root at all was this crate's own first job, and is
//! what gave that transport something real to be designed against.

mod agent;
mod check;
mod composition;
mod correction;
mod response;
mod sources;
mod spec;
#[cfg(test)]
mod test_support;
mod work;
mod workflow;

pub use agent::{AgentDispatchResponse, AgentJudgeRoleResponse, Handle_Agent_Execute, Handle_Agent_Judge_Role};
pub use check::{CheckResponse, ClaimResponse, ExaminedResponse, Handle_Check_Run};
pub use correction::{CorrectionResponse, Handle_Correction_Run};
pub use response::{
    BaselineAllowanceResponse, BaselineDebtResponse, BaselinePopulationResponse, BucketChange, CheckOutcomeResponse, ComparabilityResponse,
    Disposition, FindingBucket,
    GateCompareResponse, GateExplainResponse, GateFindings, GatePlanResponse, GateRunResponse, Handle_Gate_Compare,
    Handle_Gate_Explain, Handle_Gate_Plan, Handle_Gate_Run, JudgmentDifferenceResponse, NoVerdictResponse,
    RuleCalibrationResponse, RuleOfferResponse, SuppressionDispositionResponse, SuppressionResponse,
};
pub use spec::{
    AbsenceResponse, BlockChangeResponse, CommitReportResponse, CommitResponse, CommittedPreviewResponse,
    DecisionGapResponse, DocumentSourceResponse, FailureResponse, FieldValueResponse, FreshnessResponse,
    Handle_Spec_Commit, Handle_Spec_Freshness, Handle_Spec_Markdown, Handle_Spec_Preview, Handle_Spec_Profiles,
    Handle_Spec_Record, Handle_Spec_Render, Handle_Spec_Sources, Handle_Spec_Submit, Handle_Spec_Table,
    IdentityChangeResponse, MarkdownResponse, NodeSummaryResponse, NormativeMovementResponse,
    NormativeOutcomeResponse, OriginResponse, PathMatchResponse, PreviewResponse, ProfileOutcomeResponse,
    ProfilesResponse, RecordRelationResponse, RecordResponse, RefusalResponse, RenderResponse,
    RenderedProjectionResponse, ReproductionResponse, RowCensusResponse, SeverityResponse, SourcesResponse,
    SubmissionKindResponse, SubmissionResponse, SubmissionStateResponse, SubmitResponse, TableLineResponse,
    TableResponse, VacateOutcomeResponse, VacatedResponse, VerdictResponse,
};
pub use work::{
    AbandonResponse, AddResponse, AuditResponse, BlockedItem, DeclineResponse, FinishResponse, Handle_Work_Abandon,
    Handle_Work_Add, Handle_Work_Audit, Handle_Work_Claim, Handle_Work_Decline, Handle_Work_Finish, Handle_Work_List,
    Handle_Work_Renew, Handle_Work_Show, Handle_Work_TakeOver, Handle_Work_Validate, ListResponse,
    ReservationOutcomeResponse, ReservationResponse, ShowResponse, ValidateResponse,
};
pub use workflow::{
    AgentExecutionErrorResponse, AgentExecutionOutcomeResponse, DispatchErrorResponse, Handle_Workflow_Run,
    OllamaExecutionErrorResponse, OllamaExecutionOutcomeResponse, StepOutcomeResponse, WorkflowRunResponse,
};

