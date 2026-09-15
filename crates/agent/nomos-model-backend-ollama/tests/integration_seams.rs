//! `nomos-model-backend-ollama`'s public surface is exactly one function, [`Execute_Task`]
//! -- everything else (`Execute_In`, `Command_For`, this crate's own
//! `Isolated_Working_Directory` wrapper) is `pub(crate)` or private, by design, so a test
//! compiled outside `src/` can only reach this crate's real seams the way a real consumer
//! does: build a real [`TaskEnvelope`], hand it a real [`ProcessLauncher`], and read back a
//! real [`AgentExecutionOutcome`] or [`AgentExecutionError`].

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_agent_contracts::{Isolated_Working_Directory, TaskEnvelope};
use nomos_contracts::{CapabilityId, KnowledgeReferenceId, RuleId, SchemaId};
use nomos_ledger::{LedgerItem, Territory};
use nomos_model_backend_ollama::{AgentExecutionError, Execute_Task};
use nomos_model_package::EffortLevel;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::time::Duration;

/// The claim lease the ledger seam test below asks for: comfortably longer than the test
/// takes, so a claim that lapsed mid-test could not be mistaken for the seam failing.
const CLAIM_LEASE: Duration = Duration::from_secs(60);

/// A launcher whose one answer was written down by the test that built it, exercising
/// `nomos_platform::ProcessLauncher` -- the trait boundary `Execute_Task` is generic
/// over -- from outside this crate.
struct Scripted
{
    outcome: ExitOutcome,
    stdout: String,
    stderr: String,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for Scripted
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        return Ok(ProcessOutput { outcome: self.outcome, stdout: self.stdout.clone(), stderr: self.stderr.clone() });
    }
}

/// A `TaskEnvelope` built the way a real caller would: every field a real value from the
/// crate that owns it, not a placeholder -- `nomos_contracts` for the identifiers,
/// `nomos_ledger`'s own re-exported `Territory` for scope (the same type
/// `nomos-agent-contracts` itself declares `TaskEnvelope.scope` as), `nomos_model_package`
/// for effort.
/// `available_tools` is empty, and that is a real value rather than an omission:
/// `OD-EXECUTOR-007` has this backend refuse a capability it cannot grant, so a non-empty
/// list here would test the refusal rather than the dispatch. It has its own case below.
fn Real_Task(goal: &str) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(["crates/agent/nomos-model-backend-ollama"]),
        knowledge_context: vec![KnowledgeReferenceId::New("kwb:decision:integration-seam-test")],
        applicable_rules: vec![RuleId::New("check-naming-convention")],
        prohibited_changes: Territory::Of_Files(["work/ledger.json"]),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.v1"),
        effort: EffortLevel::BackendDefault,
    };
}

/// The happy path across the whole boundary: a real `TaskEnvelope` (`nomos_agent_contracts`
/// and, through its `scope`/`prohibited_changes` fields, `nomos_ledger`'s re-exported
/// `Territory`), driven through `Execute_Task` by a launcher satisfying the real
/// `nomos_platform::ProcessLauncher` contract.
#[test]
fn Test_Execute_Task_Should_Read_A_Clean_Response_From_A_Real_Task_Envelope()
{
    let launcher = Scripted { outcome: ExitOutcome::Exited { code: 0 }, stdout: "PONG\n".to_owned(), stderr: String::new() };

    let outcome = Execute_Task(&Real_Task("say PONG"), &launcher).expect("a well-formed scripted response");

    assert_eq!(outcome.response, "PONG");
}

/// The error `nomos_platform`'s own `ProcessLauncher::Run` reports (`Result::Err`) must
/// cross the boundary as `AgentExecutionError::Unavailable`, never as a panic or a
/// silently swallowed failure.
#[test]
fn Test_Execute_Task_Should_Report_A_Launcher_Failure_As_Unavailable()
{
    struct Unavailable;
    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for Unavailable
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProcessLauncher for Unavailable
    {
        fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
        {
            return Err("nomos-platform could not start the process at all".to_owned());
        }
    }

    let error = Execute_Task(&Real_Task("say hello"), &Unavailable).expect_err("the launcher never even started");

    assert!(matches!(error, AgentExecutionError::Unavailable(_)));
}

/// A process that exits non-zero -- the shape a genuinely unreachable `ollama serve`
/// daemon produces, per this crate's own doc comment -- is `Unavailable` too.
#[test]
fn Test_Execute_Task_Should_Report_A_Non_Zero_Exit_As_Unavailable()
{
    let launcher = Scripted {
        outcome: ExitOutcome::Exited { code: 1 },
        stdout: String::new(),
        stderr: "could not connect to the ollama daemon".to_owned(),
    };

    let error = Execute_Task(&Real_Task("say hello"), &launcher).expect_err("a non-zero exit is not a result");

    assert!(matches!(error, AgentExecutionError::Unavailable(_)));
}

/// The `nomos_ledger` seam is more than borrowing its re-exported `Territory` type: a
/// `Territory` this crate hands `TaskEnvelope.scope` must also be the same value
/// `nomos_ledger`'s own exclusion model claims a real item's territory as, or the two
/// crates only agree on a name rather than a shape. This claims a real `FileLedger` item
/// whose territory is the identical `Territory` value `Real_Task` gives `TaskEnvelope`,
/// proving the type is not merely re-exported but load-bearing on both sides.
#[test]
fn Test_The_Ledger_Territory_A_Task_Envelope_Carries_Is_The_Same_Territory_A_Real_Ledger_Claims()
{
    use nomos_ledger::{ExclusionLedger, FileLedger, ItemId, LedgerDocument, SCHEMA_VERSION};
    use nomos_platform_std::{FileLock, StdFileSystem};

    let task = Real_Task("close the nomos_ledger seam");
    let directory = Isolated_Working_Directory("nomos-model-backend-ollama-ledger-seam-test").expect("a real scratch directory");
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        nomos_platform_std::SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![Ready_Item(task.scope.clone())] })
        .expect("a fresh ledger accepts one ready item");
    let reservation = ledger
        .Claim(&ItemId::New("integration-seam-test-item"), "integration-seam-test", CLAIM_LEASE)
        .expect("claiming the item this test just wrote must succeed");

    assert_eq!(reservation.item, ItemId::New("integration-seam-test-item"));
    std::fs::remove_dir_all(&directory).expect("the scratch directory this test just created is there to be removed");
}

/// The one item the ledger seam test above claims: ready to be claimed, and carrying
/// `territory` as its own.
///
/// Every field is written out rather than defaulted, because a real ledger item carries all
/// of them and a fixture that omitted one would prove less about the seam than it claims.
fn Ready_Item(territory: Territory) -> LedgerItem
{
    use nomos_ledger::{ItemId, ItemKind, ItemOrigin, ItemState};

    return LedgerItem {
        id: ItemId::New("integration-seam-test-item"),
        title: "prove the shared Territory type".to_owned(),
        why: "the seam between this crate and nomos_ledger must be real, not coincidental naming".to_owned(),
        done_when: "the claimed territory equals the task envelope's own scope".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory,
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}

/// A `CapabilityId` this backend cannot grant crosses the boundary as a refusal, from
/// outside the crate, the way a real consumer meets it. The same assertion its sibling
/// makes, deliberately: `OD-EXECUTOR-007`'s refusal is the envelope's mechanism, so the two
/// backends owe a caller the same answer rather than each deciding for itself.
#[test]
fn Test_Execute_Task_Should_Refuse_A_Capability_It_Has_No_Way_To_Grant()
{
    let launcher = Scripted {
        outcome: ExitOutcome::Exited { code: 0 },
        stdout: String::new(),
        stderr: String::new(),
    };
    let mut task = Real_Task("say PONG");
    task.available_tools = vec![CapabilityId::New("nomos.cap.example.ollama_seam_refusal_test_only")];

    let error = Execute_Task(&task, &launcher).expect_err("a capability with no grant is refused");

    assert_eq!(error, AgentExecutionError::UnsupportedTools("nomos.cap.example.ollama_seam_refusal_test_only".to_owned()));
}
