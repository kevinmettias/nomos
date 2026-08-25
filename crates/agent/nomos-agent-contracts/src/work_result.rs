//! What an agent-assisted operation hands back when it is done.

use nomos_contracts::Finding;
use nomos_corrections::CorrectionPlan;
use nomos_ledger::VerificationPredicate;

/// `AGT-002`: "Agents shall return structured plans, changes, claims, tests, requested
/// verification, assumptions, and unresolved questions."
///
/// Six fields cover the corpus's seven named elements because "plans" and "changes" are
/// already one artifact: a [`CorrectionPlan`]'s candidates each carry their own
/// `ChangeSet`, so a second, separate `changes` field would duplicate data already
/// reachable through `plan`, not transcribe a distinct one. `tests` is the individual
/// commands an agent ran or proposes; `requested_verification` is the single predicate
/// it asks be run to accept the whole submission -- the same shape
/// [`nomos_ledger::LedgerItem::verification`] already uses at `Option` cardinality.
/// `assumptions`/`unresolved_questions` are the only genuinely new pieces: no existing
/// type in this workspace covers either concept.
///
/// `plan` is `Option`, not required, for the same reason `requested_verification` already
/// is: an agent's response may legitimately have neither. `CorrectionPlan::New` refuses an
/// empty candidate list (`CorrectionError::Vacuous`) on purpose -- a *proposed* plan with
/// nothing in it is a defect, not a real answer -- but a judgment-only task (assess this,
/// propose nothing) was never proposing a plan at all, and forcing one to exist to satisfy
/// this field's type would fabricate a correction nobody put forward. `OD-CONTRACTS-003`
/// found this directly: `nomos-agent-executor`, this crate's own first real caller, could
/// not construct a `WorkResult` for exactly this reason.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkResult
{
    pub plan: Option<CorrectionPlan>,
    pub claims: Vec<Finding>,
    pub tests: Vec<VerificationPredicate>,
    pub requested_verification: Option<VerificationPredicate>,
    pub assumptions: Vec<String>,
    pub unresolved_questions: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};
    use nomos_corrections::{ChangeSet, CorrectionCandidate, CorrectionClass, Edit};

    fn Example_Plan() -> CorrectionPlan
    {
        let edit = Edit::New("src/lib.rs", None, Some("pub mod work_result;".to_owned()));
        let change = ChangeSet::Empty().With(edit);
        let candidate = CorrectionCandidate::New("wire the new module", change, CorrectionClass::Mechanical, vec![]);

        return CorrectionPlan::New(vec![candidate]).expect("one candidate never conflicts with itself");
    }

    fn Example_Finding() -> Finding
    {
        return Finding {
            rule: RuleId::New("check-naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([9; Digest128::BYTE_LENGTH])),
            subject_name: "WorkResult".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::AgentJudged,
            gate: GateCategory::Advisory,
            summary: "the new type has no doc example yet".to_owned(),
            locations: vec!["crates/agent/nomos-agent-contracts/src/work_result.rs".to_owned()],
        };
    }

    #[test]
    fn Test_A_Result_Carries_Exactly_What_It_Was_Given()
    {
        let result = WorkResult {
            plan: Some(Example_Plan()),
            claims: vec![Example_Finding()],
            tests: vec![VerificationPredicate::New(vec!["cargo".to_owned(), "test".to_owned(), "-p".to_owned(), "nomos-agent-contracts".to_owned()])],
            requested_verification: Some(VerificationPredicate::New(vec!["cargo".to_owned(), "test".to_owned(), "--no-fail-fast".to_owned(), "-p".to_owned(), "nomos-contract-tests".to_owned()])),
            assumptions: vec!["the module is wired into lib.rs before this runs".to_owned()],
            unresolved_questions: vec![],
        };

        assert!(result.plan.is_some());
        assert_eq!(result.claims.len(), 1);
        assert_eq!(result.tests.len(), 1);
        assert!(result.requested_verification.is_some());
        assert_eq!(result.assumptions.len(), 1);
        assert!(result.unresolved_questions.is_empty());
    }

    /// The case `OD-CONTRACTS-003` exists for: a judgment-only response, proposing no
    /// change, must be constructible without fabricating a plan to satisfy the type.
    #[test]
    fn Test_A_Judgment_Only_Result_Has_No_Plan()
    {
        let result = WorkResult {
            plan: None,
            claims: vec![Example_Finding()],
            tests: vec![],
            requested_verification: None,
            assumptions: vec![],
            unresolved_questions: vec!["does this warrant a follow-up correction?".to_owned()],
        };

        assert!(result.plan.is_none());
        assert_eq!(result.claims.len(), 1);
    }
}
