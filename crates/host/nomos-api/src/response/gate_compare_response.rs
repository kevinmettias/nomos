//! [`Handle_Gate_Compare`] and its own [`GateCompareResponse`], paired in one file: the
//! response type exists only for this one handler, the same "handler beside its own response"
//! locality every other endpoint file in this crate keeps.
//!
//! # What this answers that the other three do not
//!
//! `Handle_Gate_Run` answers what a tree looks like now. This answers what changed, which is
//! the question a gate exists to answer over time rather than once: which findings are new,
//! which are gone, and which changed bucket because a policy moved rather than because the
//! code did. `ARC-ROADMAP-001`'s constraint 5 names `compare` as one of the product Gate's
//! four verbs, and `IF-001` requires every user-visible action to have a canonical
//! application service; until this handler existed, `compare` had one only in a terminal.
//!
//! # Why it re-derives both runs rather than looking either up
//!
//! `OD-GATE-022-A` decided exactly that: a compare caller re-derives both runs inside one
//! process, builds no run-history store, and serializes no `GateRunResult`. Both cases the
//! comparison is for are same-process cases -- two trees under one policy, or one tree under
//! two policies -- and neither needs a value to outlive the process that produced it.
//! Comparing against a run some earlier invocation produced is real and deferred, and would
//! need either a persisted history keyed by `RunId` or a round-trippable twin; that record
//! says which, and says it is not today's case.

use crate::{composition, sources};
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_contracts::{Finding, RunId};
use nomos_gate_orchestration::{GateCommand, GateCompareResult, GateRunResult};
use nomos_platform::Clock;
use serde::Serialize;

use super::bucket_change::BucketChange;
use super::ComparabilityResponse;

/// Judges `baseline.root` and `candidate.root` exactly as `nomos gate run` would, and hands
/// back what changed between them as a JSON-serializable [`GateCompareResponse`].
///
/// Each side is a whole [`GateCommand`], not a shared one over two roots, because the
/// policy comparison `OD-GATE-022-A` names -- one tree under two policies -- varies
/// `suppressions`, `baseline`, `rules` or `adoption` between the two sides rather than the
/// tree. A caller wanting the code comparison instead passes the same policy twice and two
/// roots. Both are the same call.
#[must_use]
pub fn Handle_Gate_Compare(baseline: &GateCommand, candidate: &GateCommand) -> GateCompareResponse
{
    let before = Judged(baseline);
    let after = Judged(candidate);

    return match nomos_gate_orchestration::Compare_Gate_Runs(&before, &after)
    {
        Ok(result) => GateCompareResponse::From(result),
        Err(refused) => GateCompareResponse::Refused(before.run, after.run, refused.run),
    };
}

/// One walk-and-judge of `command.root`, under a freshly minted `RunId`.
///
/// Each side gets its own id from the same clock: two walks in one process are still two
/// executions, which is what `GateRunResult::run` distinguishes. Sequential rather than
/// concurrent -- `Run_Gate` is the expensive part and nothing here waits on I/O it could
/// overlap. This is deliberately the same composition `gate_run_response.rs` performs and
/// `nomos-cli`'s own `gate.rs` performs; unifying the three is a change to that file rather
/// than this one.
fn Judged(command: &GateCommand) -> GateRunResult
{
    let walked = sources::Walked_Sources(&command.root);
    let now = CLOCK.Now();
    let run = nomos_gate_orchestration::Fresh_Run_Id(now);

    return nomos_gate_orchestration::Run_Gate(
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
}

/// What changed between two real gate runs, in a shape `serde_json` can hand across a wire.
///
/// Three distinct lists rather than a count or a boolean. New debt, resolved debt and a
/// finding whose bucket moved are three different things to a caller deciding whether a
/// change is admissible, and collapsing them is the behaviour that makes a gate indist-
/// inguishable from a linter runner.
#[derive(Debug, Serialize)]
pub struct GateCompareResponse
{
    /// The identity of the run compared against.
    pub baseline: RunId,
    /// The identity of the run being compared.
    pub candidate: RunId,
    /// Present in the candidate, absent from the baseline -- by rule and subject identity.
    pub added: Vec<Finding>,
    /// Present in the baseline, absent from the candidate.
    pub removed: Vec<Finding>,
    /// Present in both, in different buckets.
    pub changed: Vec<BucketChange>,
    /// The run that could not be compared, when no comparison happened at all.
    ///
    /// `None` on every real comparison. `Some` means the three lists above are empty because
    /// nothing was compared, **not** because the two runs agreed — and telling those two apart
    /// is the whole reason this field exists. A run whose findings do not yield one occurrence
    /// identity each cannot be indexed without dropping one, so the comparison refuses; the
    /// library-level caller gets `nomos_gate_orchestration::CollidingOccurrences` naming both
    /// findings of every colliding pair.
    ///
    /// # Why it carries a run and not the collisions
    ///
    /// Because no wire consumer reads them yet. The only caller of this handler is
    /// `nomos-api-transport`'s dispatcher, which serializes whatever comes back, and a
    /// serialized failure taxonomy authored ahead of something that reads one is the invented
    /// shape this crate declines elsewhere by name. The run identifies which side to look at,
    /// which is what a consumer needs to act; the pairing is one `nomos gate compare` away and
    /// is already typed for a Rust caller.
    pub uncomparable: Option<RunId>,
    /// What a reader may conclude from the three lists above.
    ///
    /// `OD-GATE-031`: a difference may be attributed to repository state only when the
    /// non-source judgment inputs of the two runs were compatible, or their differences are
    /// explicitly represented. Without this field a wire consumer reading `added` and
    /// `removed` has no way to tell a repository that changed from two runs judged under
    /// different policies, and a continuous enforcement loop is precisely the consumer that
    /// acts on that reading with no person in between.
    ///
    /// `None` only when `uncomparable` is `Some`: no comparison ran, so there is nothing to
    /// conclude from one. That is a different answer from
    /// [`ComparabilityResponse::Incomparable`], which is a comparison that did run and could
    /// not attribute what it found.
    pub comparability: Option<ComparabilityResponse>,
}

impl GateCompareResponse
{
    pub(crate) fn From(result: GateCompareResult) -> Self
    {
        return Self {
            baseline: result.baseline,
            candidate: result.candidate,
            added: result.added,
            removed: result.removed,
            comparability: Some(ComparabilityResponse::From(&result.comparability)),
            changed: result.changed.into_iter().map(BucketChange::From).collect(),
            uncomparable: None,
        };
    }

    /// The response for two runs that were judged but could not be compared.
    ///
    /// Both identities are still reported, because both runs really happened and a consumer
    /// asked about them; what is absent is any claim about the difference between them.
    pub(crate) fn Refused(baseline: RunId, candidate: RunId, uncomparable: RunId) -> Self
    {
        return Self {
            baseline,
            candidate,
            added: Vec::new(),
            removed: Vec::new(),
            changed: Vec::new(),
            uncomparable: Some(uncomparable),
            // No comparison ran, so there is nothing to say a reader may conclude from one.
            // `None` here is that absence, and is not the same answer as `Incomparable`, which
            // is a comparison that happened and could not attribute what it found.
            comparability: None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::response::FindingBucket;
    use crate::response::suppressed_because::SuppressedBecause;
    use crate::response::suppression_standing::SuppressionStanding;
    use nomos_contracts::{RuleId, SubjectId};
    use crate::test_support::{Area, Probe_Tree, TreeName};
    use nomos_gate_orchestration::FindingDisposition;
    use std::path::PathBuf;

    /// The digest length a `RunId` in these tests is built from.
    const DIGEST_BYTES: usize = 16;
    /// The digest bytes the baseline side of a refused comparison is given.
    const BASELINE_BYTES: [u8; DIGEST_BYTES] = [1; DIGEST_BYTES];
    /// The digest bytes the candidate side of a refused comparison is given.
    const CANDIDATE_BYTES: [u8; DIGEST_BYTES] = [2; DIGEST_BYTES];
    /// The subsystem this module's own probe trees are named under.
    const COMPARABILITY_AREA: Area = Area("comparability");

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent.
    fn At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    /// The defect, end to end through the handler a transport calls: two trees under two
    /// declared policies come back saying so, rather than reporting the movement as the
    /// repository's.
    ///
    /// Driven through `Handle_Gate_Compare` over real roots because that is what makes this
    /// reachable at all -- a run resolves its policy from its own root, and this handler takes
    /// two whole commands, so nobody has to do anything unusual to hit it.
    #[test]
    fn Test_Two_Policies_Should_Reach_A_Headless_Caller_As_A_Stated_Difference()
    {
        let lenient = Probe_Tree(COMPARABILITY_AREA, TreeName("lenient"), r#"{ "coverage": "unset" }"#)
            .expect("the temp directory is writable and this call's own name is fresh");
        let strict = Probe_Tree(COMPARABILITY_AREA, TreeName("strict"), r#"{ "coverage": "require-completeness" }"#)
            .expect("the temp directory is writable and this call's own name is fresh");

        let response = Handle_Gate_Compare(&Command_At(&lenient.to_string_lossy()), &Command_At(&strict.to_string_lossy()));

        let _ignored = std::fs::remove_dir_all(&lenient);
        let _ignored = std::fs::remove_dir_all(&strict);
        let rendered = serde_json::to_value(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        assert_eq!(At(&rendered, &["comparability", "comparability"]), "compatible_with", "{rendered}");
        assert_eq!(At(&rendered, &["comparability", "differences"]).to_string(), "[\"policy\"]", "{rendered}");
    }

    /// Two runs over one tree under one policy are compatible, so a caller may read the
    /// difference between them as the repository's.
    ///
    /// The assertion that keeps the one above honest: a comparability that caveated every
    /// comparison would satisfy the record's letter and destroy the verb.
    #[test]
    fn Test_One_Tree_Under_One_Policy_Should_Reach_A_Headless_Caller_As_Compatible()
    {
        let root = Probe_Tree(COMPARABILITY_AREA, TreeName("compatible"), "{}")
            .expect("the temp directory is writable and this call's own name is fresh");

        let response = Handle_Gate_Compare(&Command_At(&root.to_string_lossy()), &Command_At(&root.to_string_lossy()));

        let _ignored = std::fs::remove_dir_all(&root);
        let rendered = serde_json::to_value(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        assert_eq!(At(&rendered, &["comparability", "comparability"]), "compatible", "{rendered}");
        assert!(At(&rendered, &["comparability"]).get("differences").is_none(), "{rendered}");
    }

    /// Comparing a tree with itself under one policy reports nothing changed.
    ///
    /// The identity case, and the one that would catch a comparison matching findings by
    /// something that varies between two runs of the same thing -- a `RunId`, a timestamp, a
    /// vector index. Both sides are real walks of this crate's own tree, not fixtures.
    #[test]
    fn Test_Comparing_A_Tree_With_Itself_Should_Report_No_Change()
    {
        let response = Handle_Gate_Compare(&Command_At("."), &Command_At("."));

        assert!(
            response.added.is_empty() && response.removed.is_empty() && response.changed.is_empty(),
            "comparing a tree with itself reported {} added, {} removed and {} changed. Two \
             runs of the same tree under the same policy answer the same question, so a \
             difference here is the comparison matching on something that is not the \
             finding's identity.",
            response.added.len(),
            response.removed.len(),
            response.changed.len()
        );
    }

    /// A real comparison reports no refusal.
    ///
    /// Half of the pair that makes `uncomparable` mean something. Without it the field could be
    /// `Some` on every response and every other assertion here would still pass, because none of
    /// them reads it.
    #[test]
    fn Test_A_Real_Comparison_Should_Report_No_Refusal()
    {
        let response = Handle_Gate_Compare(&Command_At("."), &Command_At("."));

        assert_eq!(
            response.uncomparable, None,
            "a comparison that happened must not look like one that was refused"
        );
    }

    /// A refused comparison carries no differences a reader could mistake for a result.
    ///
    /// The other half, and the reason the field exists at all. Three empty lists mean *the two
    /// runs agreed* on every other response; on this one they mean *nothing was compared*, and
    /// the only thing that separates those two readings is `uncomparable`. A refusal that also
    /// carried differences would be claiming a difference it never computed.
    #[test]
    fn Test_A_Refused_Comparison_Should_Carry_No_Differences()
    {
        let baseline = RunId::From_Digest(nomos_contracts::Digest128::From_Bytes(BASELINE_BYTES));
        let candidate = RunId::From_Digest(nomos_contracts::Digest128::From_Bytes(CANDIDATE_BYTES));

        let response = GateCompareResponse::Refused(baseline, candidate, baseline);

        assert_eq!(response.uncomparable, Some(baseline), "the refusal must name the side that could not be indexed");
        assert_eq!(response.baseline, baseline, "both runs really happened and are still reported");
        assert_eq!(response.candidate, candidate);
        assert!(
            response.added.is_empty() && response.removed.is_empty() && response.changed.is_empty(),
            "a refused comparison computed no difference, so it must report none"
        );
    }

    /// The two sides are distinct executions.
    ///
    /// `RunId` is what tells a caller which side a reported difference came from, so two
    /// walks sharing one id would make the answer unreadable even when it is correct.
    #[test]
    fn Test_The_Two_Sides_Should_Be_Distinct_Runs()
    {
        let response = Handle_Gate_Compare(&Command_At("."), &Command_At("."));

        assert_ne!(
            response.baseline, response.candidate,
            "both sides reported the same RunId, so nothing in the response says which run a \
             difference belongs to"
        );
    }

    /// Every bucket maps to its own twin variant.
    ///
    /// The twin exists to be exhaustive; this is what makes a reordering or a dropped arm
    /// fail here rather than serialize as the wrong word.
    #[test]
    fn Test_Every_Disposition_Should_Map_To_Its_Own_Bucket()
    {
        assert_eq!(FindingBucket::From(FindingDisposition::Blocking), FindingBucket::Blocking);
        assert_eq!(FindingBucket::From(FindingDisposition::Calibrated), FindingBucket::Calibrated);
        assert_eq!(FindingBucket::From(FindingDisposition::Suppressed), FindingBucket::Suppressed);
        assert_eq!(FindingBucket::From(FindingDisposition::Baselined), FindingBucket::Baselined);
    }

    /// A bucket change serializes with both ends, under the names a wire caller reads.
    #[test]
    fn Test_A_Bucket_Change_Should_Serialize_Both_Ends()
    {
        let change = BucketChange {
            rule: RuleId::New("check-naming-convention"),
            subject: SubjectId::From_Digest(nomos_model::Content_Digest(b"src/lib.rs")),
            subject_name: "src/lib.rs".to_string(),
            before: FindingBucket::Blocking,
            after: FindingBucket::Baselined,
            before_reason: None,
            after_reason: Some(SuppressedBecause::TemporaryWaiver),
            before_standing: None,
            after_standing: Some(SuppressionStanding::Expired),
        };

        let rendered = serde_json::to_string(&change).expect("a BucketChange serializes");

        assert!(
            rendered.contains("\"before\":\"blocking\"")
                && rendered.contains("\"after\":\"baselined\"")
                && rendered.contains("\"after_reason\":\"temporary_waiver\"")
                && rendered.contains("\"after_standing\":\"expired\""),
            "a bucket change must carry both ends in snake_case, and rendered as {rendered}"
        );
    }

    /// [`GateCommand`] over `root`, every selector at its select-everything default.
    fn Command_At(root: &str) -> GateCommand
    {
        return GateCommand { root: PathBuf::from(root), ..Default::default() };
    }
}
