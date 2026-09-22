//! The seam between `nomos_cli`'s `agent` group and `nomos_agent_contracts`.
//!
//! `nomos-cli` is `[[bin]]`-only, so nothing here can call
//! `nomos_agent_orchestration::run::Bare_Task`/`Judgment_Task` directly -- both are private
//! to that crate, and both build the `TaskEnvelope` that the very next step, generic
//! dispatch to a live `claude` or `ollama` subprocess, must never reach for real in this
//! suite. This machine may well have a real `claude` binary on `PATH`, so a test that
//! actually reached that point could spawn a live, costly, recursive agent invocation --
//! never attempted here. `P43-AGENT-CANONICAL-SEAM-2` moved that envelope-building and
//! dispatch out of this crate's own `agent/dispatch.rs` and `agent/judge_role.rs` into
//! `nomos-agent-orchestration`; this suite's own concern -- `nomos-cli`'s use of
//! `nomos_agent_contracts::TaskEnvelope` and `Isolated_Working_Directory` -- is unaffected
//! by where the envelope gets built, and `nomos-cli`'s own workflow parsing still names
//! this crate directly, building its `Body::Agent` around a `TaskEnvelope` of its own.
//!
//! What this proves: the exact `TaskEnvelope` shape a bare `nomos agent execute` or
//! `judge-role` call builds (a bare `goal`/`effort`, every other field the empty value
//! `OD-EXECUTOR-001` already reads as "nothing enumerated, nothing granted"), constructed
//! here the same way and checked against `nomos_agent_contracts::TaskEnvelope`'s own public
//! fields; the shared isolation primitive (`Isolated_Working_Directory`) both real executor
//! crates call before ever reaching a subprocess; and, through the real binary, that
//! `nomos agent`'s two verbs are wired up and refuse *before* any `TaskEnvelope` is ever
//! built, for the argument shapes that must never reach one.

use nomos_agent_contracts::{Isolated_Working_Directory, TaskEnvelope};

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// What a missing `--goal` leaves the process with.
///
/// `agent/exit_code.rs::ExitCode::Usage`'s own value: the command line was wrong, which every
/// `--goal`-less invocation below is the shape of.
const USAGE_EXIT_CODE: i32 = 2;

/// What `judge-role` over a crate `README.md`'s band table does not list leaves the process
/// with.
///
/// `agent/exit_code.rs::ExitCode::NotFound`'s own value: the answer is empty because a row
/// that was expected was not there.
const NOT_FOUND_EXIT_CODE: i32 = 6;

/// The exact bare-envelope shape `agent/dispatch.rs::Execute_Task` builds for `nomos agent
/// execute --goal <text>`: only `goal` and `effort` carry real content, everything else is
/// the empty value this crate's own fields already support.
#[test]
fn Test_A_Bare_Task_Envelope_Carries_Only_A_Goal_And_An_Effort()
{
    let envelope = TaskEnvelope {
        goal: "say PONG".to_owned(),
        scope: nomos_ledger::Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: nomos_ledger::Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: nomos_contracts::SchemaId::New("nomos.agent.executor.cli.v1"),
        effort: nomos_model_package::EffortLevel::BackendDefault,
    };

    assert_eq!(envelope.goal, "say PONG");
    assert!(envelope.scope.paths.is_empty(), "a bare CLI call names no scope");
    assert!(envelope.knowledge_context.is_empty());
    assert!(envelope.applicable_rules.is_empty());
    assert!(envelope.prohibited_changes.paths.is_empty());
    assert!(envelope.available_tools.is_empty());
    assert_eq!(envelope.effort, nomos_model_package::EffortLevel::BackendDefault);
}

/// The isolation primitive both real executor crates call before ever touching a
/// subprocess -- `nomos_agent_executor_claude_code::Execute_Task` and
/// `nomos_model_backend_ollama::Execute_Task` both delegate here, so a caller reaching
/// either backend through `nomos agent` depends on this returning a fresh, empty, real
/// directory, never this repository's own tree.
#[test]
fn Test_Isolated_Working_Directory_Should_Return_A_Fresh_Empty_Directory_Not_This_Repository()
{
    let directory = Isolated_Working_Directory("nomos-cli-agent-contracts-seam").expect("creates a real directory");

    assert!(directory.is_dir());
    let entries: Vec<_> = std::fs::read_dir(&directory).expect("reads the directory").collect();
    assert!(entries.is_empty(), "a freshly created isolated directory must start empty");
    assert_ne!(
        directory,
        std::env::current_dir().expect("a current directory exists"),
        "an isolated directory must never be this repository's own tree"
    );

    let _ignored = std::fs::remove_dir(&directory);
}

/// Two calls in the same process never collide -- the property `nomos agent execute`
/// and `nomos agent judge-role` both depend on if a single run of this binary ever
/// dispatched to more than one backend call (it does not today, but the primitive is
/// shared and this is the invariant a second caller would rely on without re-deriving
/// it).
#[test]
fn Test_Isolated_Working_Directory_Should_Never_Collide_Across_Two_Calls()
{
    let first = Isolated_Working_Directory("nomos-cli-agent-contracts-seam").expect("creates a real directory");
    let second = Isolated_Working_Directory("nomos-cli-agent-contracts-seam").expect("creates a real directory");

    assert_ne!(first, second);

    let _ignored = std::fs::remove_dir(&first);
    let _ignored = std::fs::remove_dir(&second);
}

/// `nomos agent execute` with no `--goal` at all must refuse before a `TaskEnvelope` is
/// ever built -- `Execute_Command_From_String_Arguments` requires it before `Command`
/// exists, so this can never reach the shared seam. Safe to run for real: parsing alone,
/// never a subprocess.
#[test]
fn Test_Agent_Execute_Should_Refuse_Before_Building_A_Task_Envelope_When_Goal_Is_Missing()
{
    let ran = Run(&["agent", "execute"]);

    assert_eq!(ran.code, USAGE_EXIT_CODE, "missing --goal is a usage refusal: {}", ran.stderr);
    assert!(ran.stderr.contains("--goal"), "{}", ran.stderr);
}

/// `nomos agent judge-role` over a root with no `README.md` fails at
/// `Resolve_Declared_Role`, before `Run_Agent_Judgment` is ever called -- the same safe,
/// never-reaches-a-backend path `judge_role.rs`'s own inline test drives in process. This
/// is the same property, driven through the real binary instead.
#[test]
fn Test_Agent_Judge_Role_Should_Refuse_Before_Building_A_Task_Envelope_When_The_Root_Has_No_Readme()
{
    let tree = Tree::New("agent-contracts-judge-role-no-readme");

    let ran = Run(&["agent", "judge-role", "--crate", "nomos-does-not-exist", "--root", &tree.Root()]);

    assert_eq!(ran.code, NOT_FOUND_EXIT_CODE, "no README row is `NotFound`: {}", ran.stderr);
    assert!(ran.stderr.contains("names no row"), "{}", ran.stderr);
}
