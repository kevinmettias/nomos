//! What a real `nomos gate run` produced, including the check facts behind it.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::RunId;
use std::path::PathBuf;

use crate::GateRunOutcome;
use crate::policy::EffectivePolicy;
use super::{GateFindings, GateRunProvenance, NoVerdict};

/// What a real `nomos gate run` produced.
///
/// `check_outcome` is carried in full -- including `Claim`, for information only, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code -- so a
/// caller that wants the finer detail behind `disposition` does not have to re-walk or
/// re-judge anything to get it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateRunResult
{
    /// The identity of this execution -- `OD-WORKFLOW-001`'s first real consumer for
    /// `RunId`. Supplied by the caller to [`crate::Run_Gate`], not derived from anything
    /// else in this struct: two runs over the same `root` with the same findings are still
    /// two different executions.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// Every finding this run reduced, grouped by why it does or does not block.
    pub findings: GateFindings,
    /// The reduced verdict. When [`crate::GateCommand::phases`] declares a phase policy,
    /// this is [`crate::Phased_Disposition`]'s own answer rather than the flat
    /// [`crate::Disposition_Of_Findings`] every earlier increment computed alone -- a phase
    /// policy can turn a run that would otherwise fail into one that passes, when every
    /// blocking finding is named by some phase and no phase failed unapproved. `findings`
    /// still carries every finding this run reduced, in full, regardless of what any phase
    /// decided about them.
    pub disposition: GateRunOutcome,
    /// Every declared policy entry that matched no finding in this run, described as a
    /// reader would need to find it in the file that declares it.
    ///
    /// Not a failure. A policy legitimately outlives the finding it was written for, and a
    /// repository whose debt was paid must not fail its own gate for having paid it. But it
    /// is never silent either: `OD-GATE-024` was filed because an author who writes an entry
    /// that matches nothing gets no error, no warning and no effect, and cannot tell a
    /// mis-spelling from a finding that has since been fixed.
    ///
    /// Text rather than a typed entry, deliberately. This is a report line — the identity a
    /// caller would match on is already in the policy the entry came from, and duplicating
    /// it here would be a second addressing scheme for the same question.
    pub unmatched_policy: Vec<String>,
    /// Why this run judged its tree and still reached no verdict.
    ///
    /// `None` whenever `disposition` is a real verdict. `None` as well when nothing was
    /// judged at all: `check_outcome` is the reason in that case, in more detail than a
    /// second vocabulary here could add, and restating it would be two encodings of one
    /// fact. So this is `Some` for exactly the runs that got all the way through judging and
    /// came out without a verdict, which is the case nothing could previously explain.
    pub no_verdict: Option<NoVerdict>,
    /// What judged this run, so that a comparison against another run can say whether the
    /// two were judged alike.
    ///
    /// `None` means unknown, and `OD-GATE-031` decided what that means: a comparison treats
    /// it as incomparable rather than assuming it matches. [`crate::Run_Gate`] always fills
    /// `Some`, so `None` reaches a comparison only from a result built by hand or read back
    /// from a store that predates this field -- and in both cases the honest answer is that
    /// nobody knows what judged it, which must not read as "the same thing that judged the
    /// other side".
    pub provenance: Option<GateRunProvenance>,
    /// What decided every field of the policy this run judged under.
    ///
    /// `OD-POLICY-001` decides that the effective policy carries, per field, the layer and
    /// artifact that decided it, every contribution that decision outranked, and every
    /// override a lock refused with its reason. The resolver produced all of that from the
    /// moment it existed and nothing carried it past the function that computed it, so a
    /// repository could be judged under a policy and have no way to ask why. This is that
    /// answer, travelling with the run it belongs to.
    ///
    /// Beside [`Self::provenance`] rather than inside it, and for a different question.
    /// [`GateRunProvenance::policy`] is a digest of the effective *values*, so that two runs
    /// judged under identical values compare as the same policy however they were stated;
    /// this says which layer stated them, which is exactly the fact that digest must not
    /// depend on. `OD-POLICY-001`: "provenance rides beside the digest, never inside it".
    ///
    /// `None` for a run whose resolution refused. A locked override does not refuse -- it is
    /// kept visible in the field it was refused for -- but a same-layer contradiction, an
    /// unreadable artifact and an orphaned companion do, and a run that fell back to every
    /// default rather than judging under half a policy has no resolution to report.
    /// [`Self::no_verdict`] is what says so, in the sentence the refusal wrote. `None` as
    /// well for a result built by hand, which is what every fixture that says nothing about
    /// policy passes.
    ///
    /// Boxed, which is a cost this field pays rather than imposes. A resolution is seven
    /// collections and carries about 184 bytes inline, and a `GateRunResult` is itself a
    /// variant of `nomos_workflow_orchestration`'s `DispatchError`, so an unboxed one grows
    /// `StepOutcome` and `WorkflowOutcome` with it: every step of every workflow would move
    /// the size of a policy nothing in that crate reads, which is what `large_enum_variant`
    /// exists to notice and did. One allocation per gate run buys that back, and a gate run
    /// is not a value anything constructs in a loop -- the argument
    /// `crate::policy::Effective_Gate_Policy` makes against boxing its refusal does not
    /// transfer, because that one is a sentence handed straight to a reader and this is a
    /// structure a report walks. The cost is real and lands on the caller: a composition root
    /// writes `Some(Box::new(..))` and a reader takes `as_deref()`.
    pub policy: Option<Box<EffectivePolicy>>,
}
