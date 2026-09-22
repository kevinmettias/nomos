//! `compare`: what changed between two real gate runs.
//!
//! `plan`, `run` and `explain` were all real and CLI-wired before this file existed;
//! `compare` was named in `crate::gate_command`'s own module doc as deliberately absent,
//! the same "no invented shape ahead of a real body" discipline that doc states for every
//! verb this crate has not yet built. It is the verb a gate is actually adopted to
//! answer: not "what does this run say" but "what changed since the run I already
//! trusted" -- a baseline is a number nobody can diff without it, and an adoption
//! decision has no before and after.
//!
//! [`RunId`] already exists and is threaded through [`crate::GateRunResult::run`]
//! (`P13-GATE-REPORT-RUNID`), which is the identity a comparison needs; this file is the
//! missing verb, not a missing identity. [`Compare_Gate_Runs`] takes two already-produced
//! [`crate::GateRunResult`]s directly rather than looking either up by [`RunId`] from a
//! store this crate does not own -- persisting and retrieving a past run by its own
//! identity is a composition-root concern, the same division [`crate::Run_Gate`] itself
//! draws by taking an already-walked tree rather than a root to read.
//!
//! # Why disposition, not a raw finding diff
//!
//! Two runs over an unchanged tree can still disagree about what a finding *means* if the
//! [`crate::AdoptionPolicy`], [`crate::SuppressionPolicy`] or [`crate::BaselinePolicy`]
//! between them changed -- the exact case a real adoption decision needs to see. Diffing
//! [`crate::GateRunResult::findings`]'s own four buckets, rather than
//! [`crate::GateRunResult::check_outcome`]'s raw list, answers "did tightening or loosening
//! a policy change what can fail this build" as directly as `compare` can be asked to.

mod colliding_occurrences;
mod comparability;
mod disposition_change;
mod finding_disposition;
mod gate_compare_result;
mod judgment_difference;

#[cfg(test)]
mod occurrence_tests;
#[cfg(test)]
mod real_population;
#[cfg(test)]
mod reason_tests;
#[cfg(test)]
mod tests;

pub use colliding_occurrences::CollidingOccurrences;
pub use comparability::Comparability;
pub use disposition_change::DispositionChange;
pub use finding_disposition::FindingDisposition;
pub use gate_compare_result::GateCompareResult;
pub use judgment_difference::JudgmentDifference;

use nomos_contracts::{Finding, RuleId, RunId};
use nomos_model::{FindingOccurrenceId, Occurrence_Collisions_In};
use std::collections::BTreeMap;

use crate::{GateFindings, GateRunProvenance, GateRunResult, SuppressionReason};

/// What a finding sorts by in a rendered comparison.
///
/// A named result rather than a tuple: all three members a finding sorts by are of different
/// types, so the compiler would catch a transposition -- and it would still be a caller counting
/// positions to learn that the third one is where the occurrence's geometry went. The names say
/// it instead, which is what the tie-breaking sort below is actually about.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ReadingKey<'a>
{
    rule: &'a RuleId,
    subject_name: &'a String,
    locations: &'a Vec<String>,
}

/// Compares `baseline` and `candidate`'s own reduced findings, identifying one occurrence
/// with another across the two runs by [`FindingOccurrenceId`].
///
/// # Why occurrence identity, and why only here
///
/// This used to key on (`rule`, `subject`), matching what [`crate::SuppressionPolicy`] and
/// [`crate::BaselinePolicy`] address a finding by. That is the right identity for a *tolerance*,
/// which must outlive the revision it was written in, and the wrong one for a *comparison*: a
/// rule emits one finding per occurrence while the subject is the file, so five violations in
/// one file collapsed to one and four were dropped by a map insert. Measured on this
/// repository's own tree before the fix: 253 findings, 150 distinct pairs, 103 discarded, the
/// worst single key holding twenty-four.
///
/// So comparison moved to occurrence scope and **suppression and baseline deliberately did
/// not**. Giving either of them occurrence scope would expire every tolerated entry on the next
/// reformatting commit, which is a policy change and not a consequence of this one;
/// [`Reason_For`] is where that division is enforced, and a test holds it.
///
/// # Errors
///
/// [`CollidingOccurrences`] when either run's findings do not yield one identity each. Checked
/// per run *before* any index exists, so the silent-overwrite mechanism this function was fixed
/// to remove is not reachable on a path that has supposedly been validated.
pub fn Compare_Gate_Runs(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
) -> Result<GateCompareResult, CollidingOccurrences>
{
    let before = Occurrences_By_Id(&baseline.findings, baseline.run)?;
    let after = Occurrences_By_Id(&candidate.findings, candidate.run)?;

    let compared = GateCompareResult {
        baseline: baseline.run,
        candidate: candidate.run,
        added: Added_Between(&before, &after),
        removed: Removed_Between(&before, &after),
        changed: Changed_Between(baseline, candidate, &before, &after),
        comparability: Comparability_Of(baseline, candidate),
    };

    return Ok(In_Reading_Order(compared));
}

/// `findings`, keyed by occurrence identity against which bucket each one fell into.
///
/// # Errors
///
/// [`CollidingOccurrences`] when two findings in `run` reach one identity.
///
/// # Why the check is before the map and not inside it
///
/// Because a check inside the loop would be reading `BTreeMap::insert`'s displaced value, and
/// the whole defect this function was rewritten to remove is that nobody reads it. Validating
/// the population first means the map is built only over input already known to be injective,
/// and there is no branch on which a displaced value could be produced and dropped. The order is
/// the guarantee; a comment asking the next reader to preserve it would not be.
fn Occurrences_By_Id(
    findings: &GateFindings,
    run: RunId,
) -> Result<BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>, CollidingOccurrences>
{
    let population = Population_Of(findings);
    let subjects: Vec<&Finding> = population.iter().map(|(_, finding)| return *finding).collect();
    let collisions = Occurrence_Collisions_In(&subjects);

    if !collisions.is_empty()
    {
        return Err(CollidingOccurrences { run, collisions });
    }

    let mut indexed = BTreeMap::new();

    for (disposition, finding) in population
    {
        indexed.insert(FindingOccurrenceId::Of(finding), (disposition, finding.clone()));
    }

    return Ok(indexed);
}

/// Every finding in `findings`, paired with the bucket it fell into.
///
/// Separate from [`Occurrences_By_Id`] because the validation below has to see the whole population
/// *before* anything is keyed by identity. A finding matched by more than one bucket cannot
/// occur, the same disjointness [`GateFindings`]'s own field docs already state.
fn Population_Of(findings: &GateFindings) -> Vec<(FindingDisposition, &Finding)>
{
    let mut population = Vec::new();

    for (bucket, disposition) in [
        (&findings.blocking_findings, FindingDisposition::Blocking),
        (&findings.calibrated_findings, FindingDisposition::Calibrated),
        (&findings.suppressed_findings, FindingDisposition::Suppressed),
        (&findings.baselined_findings, FindingDisposition::Baselined),
        (&findings.baseline_exceeded_findings, FindingDisposition::BaselineExceeded),
        (&findings.below_evidence_floor_findings, FindingDisposition::BelowEvidenceFloor),
    ]
    {
        for finding in bucket
        {
            population.push((disposition, finding));
        }
    }

    return population;
}

/// Every finding in `after` that no identity in `before` answers to.
fn Added_Between(
    before: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
    after: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
) -> Vec<Finding>
{
    return after
        .iter()
        .filter(|(key, _)| return !before.contains_key(*key))
        .map(|(_, (_, finding))| return finding.clone())
        .collect();
}

/// Every finding in `before` that no identity in `after` answers to.
fn Removed_Between(
    before: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
    after: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
) -> Vec<Finding>
{
    return before
        .iter()
        .filter(|(key, _)| return !after.contains_key(*key))
        .map(|(_, (_, finding))| return finding.clone())
        .collect();
}

/// Every occurrence present in both runs whose bucket or recorded reason moved.
fn Changed_Between(
    baseline: &GateRunResult,
    candidate: &GateRunResult,
    before: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
    after: &BTreeMap<FindingOccurrenceId, (FindingDisposition, Finding)>,
) -> Vec<DispositionChange>
{
    return after
        .iter()
        .filter_map(|(occurrence, (after_disposition, after_finding))| {
            let (before_disposition, before_finding) = before.get(occurrence)?;
            let before_reason = Reason_For(&baseline.findings, before_finding);
            let after_reason = Reason_For(&candidate.findings, after_finding);

            // Two states are the same state only when the treatment and the reason
            // producing it are both the same. Comparing the bucket alone made a
            // disposition change within `Suppressed` report as nothing at all.
            if before_disposition == after_disposition && before_reason == after_reason
            {
                return None;
            }

            return Some(DispositionChange {
                rule: after_finding.rule.clone(),
                subject: after_finding.subject,
                subject_name: after_finding.subject_name.clone(),
                locations: after_finding.locations.clone(),
                before: *before_disposition,
                after: *after_disposition,
                before_reason,
                after_reason,
            });
        })
        .collect();
}

/// The suppression reason `findings` recorded for `finding`, matched by (`rule`, `subject`).
///
/// **Deliberately not by occurrence.** `GateFindings::suppression_reasons` is written by policy
/// evaluation, and a suppression addresses a rule and a subject so that it survives edits around
/// the finding it tolerates. Looking it up by occurrence identity would silently narrow every
/// existing suppression to one line of one revision -- the policy change this increment does not
/// make and must not make by accident. Written as its own function so the division has a place
/// to be stated and a place for a test to point at.
fn Reason_For(findings: &GateFindings, finding: &Finding) -> Option<SuppressionReason>
{
    return findings.suppression_reasons.get(&(finding.rule.clone(), finding.subject)).copied();
}

/// What the two runs' provenance says about attributing a difference between them.
///
/// Reads both sides' [`crate::GateRunProvenance`] and nothing else — in particular not the
/// findings, which is what keeps this a statement about the instruments rather than a second
/// opinion about the diff.
#[must_use]
fn Comparability_Of(baseline: &GateRunResult, candidate: &GateRunResult) -> Comparability
{
    let unknown = Runs_Without_Provenance(baseline, candidate);

    if !unknown.is_empty()
    {
        return Comparability::Incomparable(unknown);
    }

    let (Some(before), Some(after)) = (baseline.provenance.as_ref(), candidate.provenance.as_ref())
    else
    {
        // Unreachable past the filter above, and answered rather than asserted away: an
        // unreachable claim about exactly this shape is what aborted `nomos gate run` before
        // `P106` measured it.
        return Comparability::Incomparable(vec![baseline.run, candidate.run]);
    };

    return Differences_Between(before, after);
}

/// The runs among `baseline` and `candidate` that never said what judged them.
///
/// The evidence for an incomparability, gathered before the pair is read for differences: a
/// comparison made from a side that cannot say what produced it has nothing to attribute a
/// difference to, so this is answered first rather than reported as one more difference.
fn Runs_Without_Provenance(baseline: &GateRunResult, candidate: &GateRunResult) -> Vec<RunId>
{
    return [baseline, candidate]
        .iter()
        .filter(|result| return result.provenance.is_none())
        .map(|result| return result.run)
        .collect();
}

/// What two stated provenances disagree about.
///
/// A caveat rather than a refusal: the two runs still happened, and which of the instrument
/// facts moved is what a reader needs in order to weigh the difference they are being shown.
fn Differences_Between(before: &GateRunProvenance, after: &GateRunProvenance) -> Comparability
{
    let mut differences = Vec::new();

    if before.policy != after.policy
    {
        differences.push(JudgmentDifference::Policy);
    }
    if before.selection != after.selection
    {
        differences.push(JudgmentDifference::Selection);
    }
    if before.instrument != after.instrument
    {
        differences.push(JudgmentDifference::Instrument);
    }

    if differences.is_empty()
    {
        return Comparability::Compatible;
    }

    return Comparability::CompatibleWith(differences);
}

/// `compared` with its three lists in the order a reader scans them.
///
/// Sorted by what a reader scans, with `locations` breaking the tie that occurrence scope
/// introduced: (rule, subject_name) alone stopped being unique the moment one subject could
/// contribute more than one entry, and a comparison whose output order is unstable between
/// runs over identical input is a diff nobody can diff.
fn In_Reading_Order(mut compared: GateCompareResult) -> GateCompareResult
{
    compared.added.sort_by(|left, right| return Reading_Order(left).cmp(&Reading_Order(right)));
    compared.removed.sort_by(|left, right| return Reading_Order(left).cmp(&Reading_Order(right)));
    compared.changed.sort_by(|left, right| {
        return (&left.rule, &left.subject_name, &left.locations).cmp(&(&right.rule, &right.subject_name, &right.locations));
    });

    return compared;
}

/// What a finding sorts by in a rendered comparison.
fn Reading_Order(finding: &Finding) -> ReadingKey<'_>
{
    return ReadingKey { rule: &finding.rule, subject_name: &finding.subject_name, locations: &finding.locations };
}
