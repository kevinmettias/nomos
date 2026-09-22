//! Turning what was judged into the buckets a run reports, apart from judging it.
//!
//! Every decision here reads an already-computed [`CheckOutcome`] and the policy a run resolved,
//! and answers one question: which findings still block. It is separate from
//! [`super::Run_Gate`] because a run's *composition* -- what it judged, over what scope, under
//! what policy -- and a run's *reduction* change for different reasons, and a reader asking why
//! one finding stopped blocking should not have to read past the other.

use nomos_check_orchestration::{CheckOutcome, Claim, Claim_Of};
use nomos_contracts::{Finding, RuleId, SubjectId};
use nomos_platform::Timestamp;
use std::collections::BTreeMap;

use crate::policy::{
    AdoptionPolicy, BaselineAllowance, BaselineDebt, BaselinePolicy, EvidenceFloor, RuleCalibration, Suppression, SuppressionPolicy,
};
use crate::{
    BaselinePopulation, CoveragePolicy, Disposition_Of_Findings, GateFindings, GateRunOutcome, NoVerdict, RuleSelector, ScopeSelector,
    SuppressionReason,
};

/// `outcome`'s findings narrowed to what `scope` admits, the judging behind them untouched.
///
/// `OD-GATE-025` decided this is where a scope belongs. Narrowing the *source* set instead
/// handed a rule answering a cross-file question a truncated world, which is how
/// `--include <one file>` came to report `no-orphan-modules` against a file its own `lib.rs`
/// declares: the file was collected and its declaring root was not.
///
/// A finding is admitted when any of its locations is, and a finding carrying no location at
/// all is admitted unchanged -- a path filter has nothing to say about a finding that names
/// no path, which is every `dependency-policy` advisory about the workspace as a whole.
pub(super) fn Scoped_Findings(outcome: CheckOutcome, scope: &ScopeSelector) -> CheckOutcome
{
    let CheckOutcome::Judged { findings, examined, claim, supporting_facts } = outcome
    else
    {
        return outcome;
    };

    let admitted = findings.into_iter().filter(|finding| return Is_Admitted(finding, scope)).collect();

    return CheckOutcome::Judged { findings: admitted, examined, claim, supporting_facts };
}

/// Whether `scope` admits `finding`, by the places it names.
fn Is_Admitted(finding: &Finding, scope: &ScopeSelector) -> bool
{
    if finding.locations.is_empty()
    {
        return true;
    }

    return finding.locations.iter().any(|location| return scope.Is_In_Scope(location));
}

/// [`Reduced_Findings`]'s own result -- named so its caller assigns [`GateFindings`] and the
/// disposition by field rather than by position.
pub(super) struct Reduction
{
    pub(super) findings: GateFindings,
    pub(super) disposition: GateRunOutcome,
    pub(super) unmatched_policy: Vec<String>,
    /// Why this reduction reached no verdict, when it judged findings and reached none.
    /// Only [`Reduced_With_Coverage`] can produce one here; the policy file is read before
    /// any of this runs and [`super::Run_Gate`] carries that cause itself.
    pub(super) no_verdict: Option<NoVerdict>,
}

/// The blocking findings, the findings an `AdoptionPolicy` calibration kept from blocking,
/// the findings a `Suppression` kept from blocking, the findings a `BaselineDebt` kept from
/// blocking, and the disposition they imply, read off a [`CheckOutcome`] this function does
/// not own and must not consume -- `check_outcome` still has to end up in
/// [`crate::GateRunResult`] afterward. `rules` narrows which findings count before any list is
/// computed; a finding whose rule is not selected can be neither blocking, calibrated,
/// suppressed nor baselined, but it still exists in `check_outcome` untouched.
/// `policies.evidence_floor` splits what remains first -- a fact about the finding's own
/// evidence rather than a disposition anybody wrote about it -- then `policies.adoption`
/// splits what cleared the floor, a coarser, rule-wide override rather than a per-finding one,
/// then `policies.suppressions` splits what calibration did not match, then `policies.baseline`
/// splits what neither matched: a finding matched by more than one reports under the earliest
/// of the four, not counted twice. `coverage` is consulted last, over `rules`' own selection rather than
/// any of the four lists it splits into -- calibration, suppression and baseline each answer
/// "does this blocking finding still block," a question about one finding at a time, while
/// `coverage` answers "did this run reach a judgment about everything it selected," a
/// question about the run as a whole.
pub(super) fn Reduced_Findings(
    outcome: &CheckOutcome,
    rules: &RuleSelector,
    policies: DispositionPolicies<'_>,
    coverage: CoveragePolicy,
) -> Reduction
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Unjudged();
    };

    let selected: Vec<Finding> = findings.iter().filter(|finding| return rules.Is_Included(&finding.rule)).cloned().collect();
    let findings = Partitioned_Findings(&selected, policies);
    let unlicensed = Unlicensed_Findings(&findings);
    let disposition = Reduced_With_Coverage(Disposition_Of_Findings(&unlicensed), coverage, &selected);

    let unmatched_policy = Unmatched_Entries(&selected, policies);

    // Disposition_Of_Findings answers Passed or Failed and never Indeterminate, so an
    // Indeterminate here is Reduced_With_Coverage's own downgrade and nothing else. Read off
    // the result rather than recomputing the claim, so the two can never disagree about why.
    let no_verdict = (disposition == GateRunOutcome::Indeterminate).then_some(NoVerdict::IncompleteCoverage);

    return Reduction { findings, disposition, unmatched_policy, no_verdict };
}

/// [`Reduced_Findings`]'s own result when `outcome` was never judged -- nothing was found, so
/// nothing can block, calibrate, suppress or baseline, and [`GateRunOutcome::Indeterminate`]
/// is the only disposition an unjudged run can support.
fn Unjudged() -> Reduction
{
    return Reduction {
        findings: GateFindings {
            blocking_findings: Vec::new(),
            calibrated_findings: Vec::new(),
            suppressed_findings: Vec::new(),
            baselined_findings: Vec::new(),
            baseline_exceeded_findings: Vec::new(),
            below_evidence_floor_findings: Vec::new(),
            baseline_populations: Vec::new(),
            suppression_reasons: BTreeMap::new(),
        },
        disposition: GateRunOutcome::Indeterminate,
        // Nothing was judged, so no entry failed to match -- none was asked.
        unmatched_policy: Vec::new(),
        // Nothing was judged, so check_outcome is already the reason and this does not
        // restate it. See GateRunResult::no_verdict's own doc.
        no_verdict: None,
    };
}

/// Both buckets of `findings` that nothing licensed, as one list.
///
/// Both, because both are findings a policy failed to cover. `OD-GATE-030`: a population above
/// the quantity its entry accepted is a baseline expansion the baseline must not hide, and a
/// run that reported it and still passed would be hiding it in the only way that matters to a
/// build. Composed here rather than by widening [`Disposition_Of_Findings`], which answers
/// about a list of findings and is right as it is.
fn Unlicensed_Findings(findings: &GateFindings) -> Vec<Finding>
{
    return findings.blocking_findings.iter().chain(findings.baseline_exceeded_findings.iter()).cloned().collect();
}

/// `outcome`, downgraded from [`GateRunOutcome::Passed`] to [`GateRunOutcome::Indeterminate`]
/// when `coverage` requires completeness and `Claim_Of(selected)` is [`Claim::Incomplete`] --
/// `OD-GATE-016`'s own decision. Leaves every other `outcome` untouched: unset, this is the
/// identity function, and [`CoveragePolicy::RequireCompleteness`]'s own doc says why a
/// `Failed` outcome is left alone rather than downgraded the same way.
fn Reduced_With_Coverage(outcome: GateRunOutcome, coverage: CoveragePolicy, selected: &[Finding]) -> GateRunOutcome
{
    return match (coverage, outcome)
    {
        (CoveragePolicy::RequireCompleteness, GateRunOutcome::Passed) if Claim_Of(selected) == Claim::Incomplete => GateRunOutcome::Indeterminate,
        (_, outcome) => outcome,
    };
}

/// Every declared entry that no finding in `selected` matched, described for a reader.
///
/// `OD-GATE-024`'s one clause that survived its own retraction: an entry matching nothing is
/// reported rather than silently ignored. Against the findings a run actually *selected*,
/// not every finding it judged, because an entry for a rule the caller deselected did not
/// fail to match -- it was never asked.
fn Unmatched_Entries(selected: &[Finding], policies: DispositionPolicies<'_>) -> Vec<String>
{
    let mut unmatched = Unmatched_Of(selected, &policies.suppressions.suppressions, "suppression");
    let unmatched_baseline = Unmatched_Of(selected, &policies.baseline.debt, "baseline entry");
    let unmatched_calibration = Unmatched_Of(selected, &policies.adoption.calibrated, "calibration");
    unmatched.extend(unmatched_baseline);
    unmatched.extend(unmatched_calibration);

    return unmatched;
}

/// `entries`' own members that no finding in `selected` named, each worded by `noun`.
///
/// One loop over a [`DeclaredEntry`] rather than three copies of one loop over three
/// unrelated types: the question -- did anything this run selected match you -- is the same
/// question, and three copies of it would be three places to change when a fourth entry kind
/// arrives.
fn Unmatched_Of<Entry: DeclaredEntry>(selected: &[Finding], entries: &[Entry], noun: &str) -> Vec<String>
{
    let mut unmatched = Vec::new();

    for entry in entries
    {
        if !selected.iter().any(|finding| return entry.Is_Applicable_To(finding))
        {
            unmatched.push(format!("{} for `{}` matched nothing", noun, entry.Named_Rule()));
        }
    }

    return unmatched;
}

/// `selected`, split into the below-floor, calibrated, suppressed, baselined and
/// still-blocking findings `policies` implies -- [`Reduced_Findings`]'s own middle section,
/// named so that function reads as one decision per line.
///
/// `policies.evidence_floor` is read first, beside `Finding::Can_Fail_A_Build`'s own two
/// conditions rather than inside it -- `OD-GATE-034`: a `Finding` does not know which gate is
/// reading it, so the contract type keeps deciding what *can* fail a build and the gate keeps
/// deciding what does. Calibration, suppression and baseline therefore never see a finding the
/// floor already took, which is what keeps a raised floor from reading as a waiver somebody
/// wrote. Under [`EvidenceFloor::Unset`] the floor admits every class and this partition is the
/// identity it was before the field existed.
pub(super) fn Partitioned_Findings(selected: &[Finding], policies: DispositionPolicies<'_>) -> GateFindings
{
    let blockable: Vec<Finding> = selected.iter().filter(|finding| return finding.Can_Fail_A_Build()).cloned().collect();
    let (admitted, below_evidence_floor_findings): (Vec<Finding>, Vec<Finding>) =
        blockable.into_iter().partition(|finding| return policies.evidence_floor.Admits(finding.evidence));
    let (calibrated_findings, uncalibrated): (Vec<Finding>, Vec<Finding>) =
        admitted.into_iter().partition(|finding| return policies.adoption.Calibrating(finding).is_some());
    let (suppressed_findings, remaining): (Vec<Finding>, Vec<Finding>) =
        uncalibrated.into_iter().partition(|finding| return policies.suppressions.Suppressing(finding, policies.now).is_some());
    let (matched, blocking_findings): (Vec<Finding>, Vec<Finding>) =
        remaining.into_iter().partition(|finding| return policies.baseline.Tolerating(finding).is_some());
    let Tolerated { baselined_findings, baseline_exceeded_findings, baseline_populations } = Tolerated_Within_Allowance(matched, policies.baseline);
    let suppression_reasons = Recorded_Reasons(selected, policies);

    return GateFindings {
        suppression_reasons,
        blocking_findings,
        calibrated_findings,
        suppressed_findings,
        baselined_findings,
        baseline_exceeded_findings,
        below_evidence_floor_findings,
        baseline_populations,
    };
}

/// What [`Tolerated_Within_Allowance`] split one run's baseline-matched findings into.
struct Tolerated
{
    baselined_findings: Vec<Finding>,
    baseline_exceeded_findings: Vec<Finding>,
    baseline_populations: Vec<BaselinePopulation>,
}

/// Splits the findings a baseline entry matched by whether their scope stayed inside the
/// quantity that entry accepted.
///
/// `OD-GATE-030` decides the shape, and two of its clauses are the reason this is a grouping
/// rather than the per-finding filter it replaced.
///
/// **The unit is the scope, not the finding.** An entry accepts a quantity for a
/// `rule`/`subject` scope, so whether it is exceeded is a fact about every occurrence in that
/// scope at once. A filter that asked each finding separately could only ever answer "an entry
/// matches you", which is what tolerated five occurrences against an entry that accepted one.
///
/// **An exceeded scope moves whole.** Where five are observed and one was accepted, four of
/// them provably post-date adoption and *which* four is unknown. Leaving one in
/// `baselined_findings` would pick a historical occurrence out of five candidates on no
/// evidence, and would let the next reformatting commit pick a different one. So the group
/// goes to `baseline_exceeded_findings` entire, and the arithmetic that explains it travels
/// beside it.
fn Tolerated_Within_Allowance(matched: Vec<Finding>, baseline: &BaselinePolicy) -> Tolerated
{
    let mut tolerated = Tolerated { baselined_findings: Vec::new(), baseline_exceeded_findings: Vec::new(), baseline_populations: Vec::new() };

    for ((rule, subject), occurrences) in Grouped_By_Scope(matched)
    {
        let observed = Population_Of(rule, subject, &occurrences, baseline);
        if observed.exceeded
        {
            tolerated.baseline_exceeded_findings.extend(occurrences);
        }
        else
        {
            tolerated.baselined_findings.extend(occurrences);
        }
        tolerated.baseline_populations.push(observed.population);
    }

    return tolerated;
}

/// `matched`, grouped by the `rule`/`subject` scope an entry addresses and in one order on
/// every run over one tree rather than in whatever order the walk happened to produce.
fn Grouped_By_Scope(matched: Vec<Finding>) -> BTreeMap<(RuleId, SubjectId), Vec<Finding>>
{
    let mut scopes: BTreeMap<(RuleId, SubjectId), Vec<Finding>> = BTreeMap::new();

    for finding in matched
    {
        scopes.entry((finding.rule.clone(), finding.subject)).or_default().push(finding);
    }

    return scopes;
}

/// One scope's occurrence population and the verdict its entry's allowance gives it.
///
/// A named result rather than a pair: a bare `true` says nothing about *what* is true, and the
/// two travel together because the verdict is a reading of the population rather than a second
/// fact about the scope.
struct ObservedScope
{
    population: BaselinePopulation,
    exceeded: bool,
}

/// One scope's occurrence population, read against the entry that accepts it.
///
/// Every finding here matched some entry, or the partition above would not have kept it, and
/// each group shares one rule and subject so one lookup answers for all of them. The fallback
/// is unreachable and is written as the permissive reading anyway: an entry nobody could find
/// must not invent a bound nobody declared.
fn Population_Of(rule: RuleId, subject: SubjectId, occurrences: &[Finding], baseline: &BaselinePolicy) -> ObservedScope
{
    let entry = occurrences.first().and_then(|finding| return baseline.Tolerating(finding));
    let allowed = entry.map_or(BaselineAllowance::Unbounded, |entry| return entry.allowance);
    // Cloned rather than borrowed for the reason the whole field exists: this outlives the
    // policy the run resolved, because a caller reads a finished run long after the file
    // it came from may have changed.
    let declared_path = entry.and_then(|entry| return entry.declared_path.clone());
    let population = BaselinePopulation { rule, subject, declared_path, allowed, observed: Occurrence_Count(occurrences.len()) };
    let exceeded = population.Is_Exceeded();

    return ObservedScope { population, exceeded };
}

/// A group's size as the count a [`BaselinePopulation`] reports.
///
/// A checked conversion rather than `as`, which would wrap a population past `u32::MAX` to a
/// small number and report a scope holding four billion occurrences as comfortably inside an
/// allowance of ten. Unreachable on any real tree and one line to make unreachable in
/// principle, which is cheaper than the argument for why it cannot happen.
fn Occurrence_Count(occurrences: usize) -> u32
{
    return u32::try_from(occurrences).unwrap_or(u32::MAX);
}

/// Why a disposition applied to each finding one names, recorded at the moment it was decided.
///
/// Both halves, deliberately. An active disposition explains why a finding is in
/// `suppressed_findings`. A lapsed one explains why a finding is *not*: under `P103` an expired
/// waiver does not suppress, so the finding falls through to baseline or blocking, and without
/// this a reader comparing two runs would see it arrive in `blocking_findings` with nothing
/// saying a tolerance came due rather than a new violation appearing. That distinction is the
/// whole reason expiry was made visible in the run model, and it would be lost again in
/// comparison if only suppressing dispositions were recorded.
fn Recorded_Reasons(selected: &[Finding], policies: DispositionPolicies<'_>) -> BTreeMap<(RuleId, SubjectId), SuppressionReason>
{
    let mut reasons = BTreeMap::new();

    for finding in selected
    {
        let key = (finding.rule.clone(), finding.subject);

        if let Some(suppression) = policies.suppressions.Suppressing(finding, policies.now)
        {
            reasons.insert(key, SuppressionReason { disposition: suppression.disposition, status: suppression.Status_At(policies.now) });
            continue;
        }

        if let Some(lapsed) = policies.suppressions.Lapsed(finding, policies.now).first()
        {
            reasons.insert(key, SuppressionReason { disposition: lapsed.disposition, status: lapsed.Status_At(policies.now) });
        }
    }

    return reasons;
}

/// What [`Reduced_Findings`] checks each still-blockable finding against, grouped into one
/// value so [`Reduced_Findings`] stays within this crate's own parameter-count limit --
/// `evidence_floor` first (a fact about the finding, not a disposition anybody wrote), then
/// `adoption` (a coarser, rule-wide override), then `suppressions`, then `baseline`.
#[derive(Clone, Copy)]
pub(super) struct DispositionPolicies<'a>
{
    pub(super) evidence_floor: EvidenceFloor,
    pub(super) adoption: &'a AdoptionPolicy,
    pub(super) suppressions: &'a SuppressionPolicy,
    pub(super) baseline: &'a BaselinePolicy,
    pub(super) now: Timestamp,
}

/// A declared policy entry that names the `rule`/`subject` scope it applies to.
///
/// [`Suppression`], [`BaselineDebt`] and [`RuleCalibration`] each answer the same question --
/// does this finding fall under me -- and each is reported by its own noun when a run selected
/// nothing it matched. The trait lives here rather than in `policy` because this one report is
/// its only reader, and it exists so [`Unmatched_Of`] is one loop rather than three.
trait DeclaredEntry
{
    /// The rule this entry names, as the unmatched report spells it.
    fn Named_Rule(&self) -> &RuleId;
    /// Whether this entry applies to `finding`, by the scope it names.
    fn Is_Applicable_To(&self, finding: &Finding) -> bool;
}

impl DeclaredEntry for Suppression
{
    fn Named_Rule(&self) -> &RuleId
    {
        return &self.rule;
    }

    fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return Suppression::Is_Applicable_To(self, finding);
    }
}

impl DeclaredEntry for BaselineDebt
{
    fn Named_Rule(&self) -> &RuleId
    {
        return &self.rule;
    }

    fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return BaselineDebt::Is_Applicable_To(self, finding);
    }
}

impl DeclaredEntry for RuleCalibration
{
    fn Named_Rule(&self) -> &RuleId
    {
        return &self.rule;
    }

    fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return RuleCalibration::Is_Applicable_To(self, finding);
    }
}

#[cfg(test)]
mod tests
{
    //! `OD-GATE-034`'s evidence floor, asserted against [`Partitioned_Findings`] directly.
    //!
    //! Beside the code rather than in `super::tests`: the findings a real run over a scratch
    //! tree produces are all at `EvidenceClass::Derived` or above -- the record measured that
    //! the population below `Derived` is zero in this workspace today -- so the case the floor
    //! exists for cannot be produced by walking a tree, and the record names a hand-built
    //! finding as its own falsifier for exactly that reason.

    use super::{DispositionPolicies, Partitioned_Findings};
    use crate::policy::{AdoptionPolicy, BaselinePolicy, EvidenceFloor, RuleCalibration, SuppressionPolicy};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
    use nomos_platform::Timestamp;

    /// The rule every finding below belongs to.
    const RULE: &str = "nesting-depth";

    /// The subject seed every finding below belongs to.
    const SUBJECT_SEED: u8 = 7;

    /// The moment these partitions are judged against. Nothing here declares an expiry, so
    /// the value decides nothing and is fixed rather than read from a clock.
    const AT_THE_EPOCH: i64 = 0;

    /// Every evidence class, weakest first.
    ///
    /// Spelled out because `EvidenceClass` publishes no list of its own, and kept honest by
    /// `crate::policy::evidence_floor`'s own ordering test rather than trusted here.
    const EVERY_CLASS: [EvidenceClass; 8] = [
        EvidenceClass::AgentJudged,
        EvidenceClass::HumanAsserted,
        EvidenceClass::Predicted,
        EvidenceClass::Approximate,
        EvidenceClass::Derived,
        EvidenceClass::Observed,
        EvidenceClass::Verified,
        EvidenceClass::Authoritative,
    ];

    /// `OD-GATE-034`'s own falsifier, written as a test rather than as prose.
    ///
    /// A finding at `Blocking`, `Supported` and `AgentJudged` -- one `Finding::Can_Fail_A_Build`
    /// says can fail a build -- under a gate declaring a floor of `Derived`. It must be out of
    /// `blocking_findings` and it must still be there to read, in a bucket of its own.
    ///
    /// Both halves, because either alone is satisfiable by something wrong. A floor that
    /// dropped the finding would pass the first assertion, and a floor that did nothing at all
    /// would pass the second.
    #[test]
    fn Test_A_Finding_Below_The_Floor_Should_Not_Block_And_Should_Not_Disappear()
    {
        let weak = Finding_With(EvidenceClass::AgentJudged);

        let partitioned = Partitioned_Findings(&[weak.clone()], Under(EvidenceFloor::AtLeast(EvidenceClass::Derived)));

        assert!(partitioned.blocking_findings.is_empty(), "a finding under the floor must not block: {:?}", partitioned.blocking_findings);
        assert_eq!(partitioned.below_evidence_floor_findings, vec![weak], "and it must still be reported, in a bucket of its own");
    }

    /// The floor is not a way to relabel a finding as somebody's disposition.
    ///
    /// `OD-GATE-034`: a calibration, a suppression and a baseline entry each say a person
    /// authored something about this finding, and the floor says nobody did. A reader who
    /// found it filed as calibrated would go looking for a calibration nobody wrote -- so this
    /// asserts the other four buckets are empty, not merely that the right one is not.
    #[test]
    fn Test_A_Below_Floor_Finding_Should_Not_Be_Filed_As_Anybody_Else_Disposition()
    {
        let weak = Finding_With(EvidenceClass::Predicted);
        let adoption =
            AdoptionPolicy { calibrated: vec![RuleCalibration { rule: RuleId::New(RULE), rationale: "adopting incrementally".to_owned() }] };
        let policies = DispositionPolicies { adoption: &adoption, ..Under(EvidenceFloor::AtLeast(EvidenceClass::Derived)) };

        let partitioned = Partitioned_Findings(&[weak], policies);

        assert_eq!(partitioned.below_evidence_floor_findings.len(), 1, "the floor took it");
        assert!(partitioned.calibrated_findings.is_empty(), "and a calibration that also matched must not claim it");
        assert!(partitioned.suppressed_findings.is_empty());
        assert!(partitioned.baselined_findings.is_empty());
        assert!(partitioned.baseline_exceeded_findings.is_empty());
    }

    /// A finding whose evidence is exactly the floor clears it.
    ///
    /// The comparison `OD-GATE-034` states is "at least the floor", so the boundary belongs on
    /// the blocking side; a strict comparison would silently raise every declared floor by one
    /// class.
    #[test]
    fn Test_A_Finding_At_The_Floor_Should_Still_Block()
    {
        let exactly = Finding_With(EvidenceClass::Derived);

        let partitioned = Partitioned_Findings(&[exactly.clone()], Under(EvidenceFloor::AtLeast(EvidenceClass::Derived)));

        assert_eq!(partitioned.blocking_findings, vec![exactly]);
        assert!(partitioned.below_evidence_floor_findings.is_empty());
    }

    /// `Unset` migrates nobody: every class still blocks, the weakest included.
    ///
    /// The whole vocabulary rather than one class, because a floor that read `Unset` as some
    /// particular class would agree with this for every class at or above it.
    #[test]
    fn Test_An_Unset_Floor_Should_Leave_Every_Class_Blocking()
    {
        for class in EVERY_CLASS
        {
            let partitioned = Partitioned_Findings(&[Finding_With(class)], Under(EvidenceFloor::Unset));

            assert_eq!(partitioned.blocking_findings.len(), 1, "{} stopped blocking under no floor at all", class.Label());
            assert!(partitioned.below_evidence_floor_findings.is_empty(), "{}", class.Label());
        }
    }

    /// An advisory finding is not reported as floored.
    ///
    /// `Finding::gate` is the rule's own wiring truth and the floor does not rewrite it: a
    /// finding that could never have blocked was not kept from blocking by the floor, and
    /// filing it here would attribute to policy what the rule already decided.
    #[test]
    fn Test_An_Advisory_Finding_Should_Not_Be_Reported_As_Floored()
    {
        let advisory = Finding { gate: GateCategory::Advisory, ..Finding_With(EvidenceClass::AgentJudged) };

        let partitioned = Partitioned_Findings(&[advisory], Under(EvidenceFloor::AtLeast(EvidenceClass::Authoritative)));

        assert!(partitioned.below_evidence_floor_findings.is_empty(), "{:?}", partitioned.below_evidence_floor_findings);
        assert!(partitioned.blocking_findings.is_empty());
    }

    /// The policies a partition is judged under when `floor` is the only thing stated.
    ///
    /// The three matchers are borrowed from process-wide empties rather than from locals,
    /// because [`DispositionPolicies`] borrows them and every test here states none: one empty
    /// set of each serves every call and outlives all of them.
    fn Under(floor: EvidenceFloor) -> DispositionPolicies<'static>
    {
        static EMPTY_ADOPTION: std::sync::OnceLock<AdoptionPolicy> = std::sync::OnceLock::new();
        static EMPTY_SUPPRESSIONS: std::sync::OnceLock<SuppressionPolicy> = std::sync::OnceLock::new();
        static EMPTY_BASELINE: std::sync::OnceLock<BaselinePolicy> = std::sync::OnceLock::new();

        return DispositionPolicies {
            evidence_floor: floor,
            adoption: EMPTY_ADOPTION.get_or_init(AdoptionPolicy::default),
            suppressions: EMPTY_SUPPRESSIONS.get_or_init(SuppressionPolicy::default),
            baseline: EMPTY_BASELINE.get_or_init(BaselinePolicy::default),
            now: Timestamp::From_Unix_Seconds(AT_THE_EPOCH),
        };
    }

    /// One blocking, fully-supported finding carrying `evidence`.
    ///
    /// Everything but the evidence class is held fixed, so a partition that moved it moved it
    /// for the one reason these tests are about.
    fn Finding_With(evidence: EvidenceClass) -> Finding
    {
        return Finding {
            address: None,
            rule: RuleId::New(RULE),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_SEED; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability: Applicability::Supported,
            evidence,
            gate: GateCategory::Blocking,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }
}
