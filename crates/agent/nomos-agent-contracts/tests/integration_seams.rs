//! Each of `nomos-agent-contracts`'s six `WorkResult` field types is a direct reuse of a
//! type from a sibling crate (`work_result.rs`'s own doc comment says so), so the real
//! contract this crate stands behind is that a value built by that sibling crate is one its
//! own public structs can carry and hand back unchanged. `Substantiation` is the one type
//! that is not a reuse — it is this crate's own vocabulary, added by `OD-EXECUTOR-011` — and
//! it is exercised here too, since a consumer reaching this crate from outside `src/` reads
//! it the same way it reads the others. Compiled outside `src/`, this file can only
//! reach `nomos_contracts`, `nomos_corrections` and `nomos_scope_verification` through
//! `nomos-agent-contracts`'s own public API — the same view a real consumer has, unlike
//! an internal `#[cfg(test)]` module which can also see private items on either side.

use nomos_agent_contracts::{PortionSubstantiation, Substantiation, TaskEnvelope, WorkResult};
use nomos_contracts::{CapabilityId, Finding, KnowledgeReferenceId, RuleId, SchemaId};
use nomos_corrections::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, Edit};
use nomos_model_package::EffortLevel;
use nomos_scope_verification::{Territory, VerificationPredicate};

/// `nomos_scope_verification`, exercised through `TaskEnvelope.scope`/`prohibited_changes`.
///
/// The happy path: a real, non-empty `Territory` naming files, and the real empty one
/// `Territory::Empty()` is defined to be — `Is_Empty` is `nomos_scope_verification`'s own
/// behaviour, not something this crate could fake by holding the value inert.
#[test]
fn Test_A_Task_Envelopes_Territories_Are_Real_Scope_Verification_Values()
{
    let envelope = TaskEnvelope {
        goal: "close the seam between this crate and scope-verification".to_owned(),
        scope: Territory::Of_Files(["crates/agent/nomos-agent-contracts"]),
        knowledge_context: vec![KnowledgeReferenceId::New("kwb:decision:1")],
        applicable_rules: vec![RuleId::New("check-naming-convention")],
        prohibited_changes: Territory::Empty(),
        available_tools: vec![CapabilityId::New("nomos.cap.example.integration_seam_test_only")],
        expected_output_schema: SchemaId::New("nomos.agent.work_result.v1"),
        effort: EffortLevel::BackendDefault,
    };

    assert!(!envelope.scope.Is_Empty(), "a territory naming a real path must not report itself empty");
    assert!(envelope.prohibited_changes.Is_Empty(), "`Territory::Empty()` must still report itself empty once carried by the envelope");
}

/// `nomos_scope_verification`'s `VerificationPredicate`, exercised through
/// `WorkResult.requested_verification` — both the runnable case and the one
/// `Is_Runnable` exists to catch: an argument vector that was never really a command.
#[test]
fn Test_A_Work_Results_Requested_Verification_Is_A_Real_Runnable_Predicate()
{
    let runnable = WorkResult {
        plan: None,
        claims: vec![],
        tests: vec![],
        requested_verification: Some(VerificationPredicate::From_String_Arguments(vec![
            "cargo".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "nomos-agent-contracts".to_owned(),
        ])),
        assumptions: vec![],
        unresolved_questions: vec![],
        substantiation: Substantiation {
            plan: PortionSubstantiation::Substantiated,
            claims: PortionSubstantiation::Substantiated,
            tests: PortionSubstantiation::Substantiated,
            requested_verification: PortionSubstantiation::Substantiated,
            assumptions: PortionSubstantiation::Substantiated,
            unresolved_questions: PortionSubstantiation::Substantiated,
        },
    };
    let unrunnable = WorkResult {
        requested_verification: Some(VerificationPredicate::From_String_Arguments(Vec::new())),
        ..Blank_Judgment_Only_Result()
    };

    assert!(
        runnable
            .requested_verification
            .as_ref()
            .is_some_and(VerificationPredicate::Is_Runnable),
        "a predicate naming a real program must be runnable"
    );
    assert!(
        !unrunnable
            .requested_verification
            .as_ref()
            .is_some_and(VerificationPredicate::Is_Runnable),
        "an empty argument vector is a field filled in to satisfy the schema, not a real predicate"
    );
}

/// `nomos_corrections`, exercised through `WorkResult.plan` — the happy path with a real
/// `CorrectionPlan`, and the error `CorrectionPlan::New` itself refuses: an empty
/// candidate list, which `work_result.rs`'s own doc comment names as exactly the case
/// `OD-CONTRACTS-003` exists for (`plan` stays `None` rather than fabricate one).
#[test]
fn Test_A_Work_Results_Plan_Is_A_Real_Correction_Plan_And_An_Empty_One_Is_Refused()
{
    let edit = Edit::New("src/lib.rs", None, Some("pub mod new_module;".to_owned()));
    let change = ChangeSet::Empty().With(edit);
    let candidate = CorrectionCandidate::New("wire the new module", change, CorrectionClass::Mechanical, vec![]);
    let plan = CorrectionPlan::New(vec![candidate]).expect("one candidate never conflicts with itself");

    let result = WorkResult { plan: Some(plan), ..Blank_Judgment_Only_Result() };

    assert!(result.plan.is_some());
    assert!(
        CorrectionPlan::New(Vec::new()).is_err(),
        "an empty candidate list is a defect in the plan, not a real proposal — `nomos_corrections` refuses it \
         at its own boundary rather than leaving this crate to fabricate one to satisfy `WorkResult`'s shape"
    );
}

/// A judgment-only `WorkResult` (`plan: None`) — the shape `OD-CONTRACTS-003` exists for,
/// reused as the base for the two tests above so each varies only the field it is about.
///
/// It carries the all-grounded substantiation declaration, so the two results derived from
/// it by functional update inherit that rather than restating it. Grounded here means the
/// test itself is the producer and built every one of these values by hand, so the empty
/// ones mean this task produced none — the reading `OD-EXECUTOR-011` requires a
/// `Substantiated` entry to license.
fn Blank_Judgment_Only_Result() -> WorkResult
{
    return WorkResult {
        plan: None,
        claims: Vec::<Finding>::new(),
        tests: vec![],
        requested_verification: None,
        assumptions: vec![],
        unresolved_questions: vec![],
        substantiation: Substantiation {
            plan: PortionSubstantiation::Substantiated,
            claims: PortionSubstantiation::Substantiated,
            tests: PortionSubstantiation::Substantiated,
            requested_verification: PortionSubstantiation::Substantiated,
            assumptions: PortionSubstantiation::Substantiated,
            unresolved_questions: PortionSubstantiation::Substantiated,
        },
    };
}
