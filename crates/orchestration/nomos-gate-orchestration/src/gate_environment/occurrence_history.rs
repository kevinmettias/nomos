//! What a run leaves behind about the occurrences a baseline entry tolerates, so that a later
//! run can say whether one persisted or came back.
//!
//! `OD-GATE-030` names three parts that would make a baseline's continuity provable. The first,
//! a quantity on the entry, is built and bounds capacity only. This module is the other two: an
//! occurrence identity that survives a revision -- `nomos_model::OccurrenceLineageId`, which
//! owns that decision rather than this crate -- and **a history of the states between**, which
//! is what a record is.
//!
//! # What the record is, and where it lives
//!
//! One JSON file at the run's own root, [`GATE_HISTORY_FILE`], holding one entry per
//! `rule`/`subject` scope a baseline entry addresses: how many states have been censused for
//! that scope, and for every occurrence lineage any of them saw, which states saw it and what
//! the record establishes about it now. `history_file`'s own doc carries why it is a file beside
//! `nomos-gate.json` rather than a key inside it, and why its mere presence is the whole opt-in.
//!
//! It is counters and not a list of past runs on purpose. `OD-GATE-022-A` deferred a run-history
//! store, and a store of whole `GateRunResult`s is a much larger thing than the question here
//! needs: whether an occurrence was there at each state is answered by three numbers per
//! lineage, and a file that grew with every run would be abandoned before it was old enough to
//! establish anything.
//!
//! # What a run censuses, and the three guards on it
//!
//! A state is one run's observation of one scope, and a wrong observation is worse than none --
//! a scope wrongly recorded as empty makes the next run report a recreation that did not happen,
//! which is a false regression nobody can act on. So a state is recorded only where the run
//! actually looked:
//!
//! 1. **The run must have judged.** An unreadable root and a root with no source observed
//!    nothing, and neither is evidence that a scope came up empty.
//! 2. **The run must not be scope-narrowed.** `gate run --include one/dir` deliberately reports
//!    on part of a tree (`OD-GATE-025`), and taking its findings for a census would record every
//!    scope outside that directory as empty. The test is `ScopeSelector::default()` exactly:
//!    anything a caller narrowed is refused rather than reasoned about.
//! 3. **The scope's own rule must be selected and must have reached a judgment.** A deselected
//!    rule was never asked, and a rule whose provider was unavailable could not look --
//!    `Claim_Of` over that rule's own findings is the existing answer to the second, so a
//!    `MissingCapability` or an unparseable subject leaves that rule's scopes uncensused for the
//!    run rather than recorded as clean.
//!
//! A census observes **every selected finding in the scope**, whatever disposition later took
//! it. A finding a suppression covered was still there, and recording only the ones that reached
//! the baseline bucket would make an expiring waiver read as a violation returning. That is also
//! why a run censuses before its dispositions are decided rather than after: what was observed
//! is settled the moment the judging returns, and the record the reduction then reads is the
//! same one the file ends up holding.
//!
//! # The limit these guards do not reach
//!
//! `Run_Gate` is handed an already-walked tree, and nothing in a walk says whether it was the
//! whole one. A caller that opts into recording and then hands this crate a partial walk records
//! absences for everything it left out. That is inherent to the seam -- the composition root
//! chose the walk (`OD-HOST-001`) -- and it is stated here rather than guarded by a heuristic
//! about how many files "should" have been seen.
//!
//! # What the record never decides
//!
//! Which occurrences inside an exceeded population are the adopted ones. `OD-GATE-030` forbids
//! that attribution, and an exceeded scope still moves whole to
//! `GateFindings::baseline_exceeded_findings` without continuity being asked about any of its
//! members. The counting bound is untouched by everything here: it is read first, over the whole
//! matched population, and continuity is asked only of the findings that bound already
//! tolerated.

mod history_file;
mod recorded_occurrence;
mod recorded_scope;

#[cfg(test)]
mod tests;

pub(super) use history_file::{Record_Occurrence_History, Resolve_Occurrence_History};
// The record's own name, reachable only to the tests that put a file at it: nothing in a real
// run needs to spell it, and a second spelling in a test is a second place for it to be wrong.
#[cfg(test)]
pub(super) use history_file::GATE_HISTORY_FILE;
use recorded_occurrence::RecordedOccurrence;
use recorded_scope::RecordedScope;

use nomos_check_orchestration::{CheckOutcome, Claim, Claim_Of};
use nomos_contracts::{Finding, RuleId, SubjectId};
use nomos_model::{IdentityTransitionKind, OccurrenceLineageId, OccurrenceTally};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::GateCommand;
use crate::policy::{BaselineDebt, BaselinePolicy, RuleSelector, ScopeSelector};

/// Every scope a run has censused, and what its states saw.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OccurrenceHistory
{
    /// One entry per `rule`/`subject` scope, in the order each was first censused.
    ///
    /// Appended to and never reordered, so two runs over one tree produce a file that differs
    /// only where the tree did.
    pub(super) scopes: Vec<RecordedScope>,
}

impl OccurrenceHistory
{
    /// What this record establishes about the lineage `finding` belongs to, or `None` when it
    /// establishes nothing.
    ///
    /// `None` is the answer for a scope this record has no state for, for a lineage none of its
    /// states saw, and for one that arrived inside the window and has been there since --
    /// `OccurrenceTally`'s own doc separates those three and says why each of them is evidence
    /// of nothing rather than evidence of a return.
    pub(super) fn Continuity_Of(&self, finding: &Finding) -> Option<IdentityTransitionKind>
    {
        let unrecorded = OccurrenceTally::default();

        return self
            .Scope_Of(&finding.rule, finding.subject)
            .map_or(unrecorded, |scope| return scope.Tally_Of(&Lineage_Key(finding)))
            .Established_Continuity();
    }

    /// The recorded scope `rule` and `subject` name, if this record holds one.
    fn Scope_Of(&self, rule: &RuleId, subject: SubjectId) -> Option<&RecordedScope>
    {
        let subject = Subject_Key(subject);

        return self.scopes.iter().find(|scope| return scope.rule == rule.As_Str() && scope.subject == subject);
    }

    /// The recorded scope `entry` addresses, started empty if this record has never held one.
    fn Scope_For(&mut self, entry: &BaselineDebt) -> &mut RecordedScope
    {
        let rule = entry.rule.As_Str().to_owned();
        let subject = Subject_Key(entry.subject);

        if self.scopes.iter().all(|scope| return scope.rule != rule || scope.subject != subject)
        {
            self.scopes.push(RecordedScope::Empty(rule.clone(), subject.clone()));
        }

        return self
            .scopes
            .iter_mut()
            .find(|scope| return scope.rule == rule && scope.subject == subject)
            .expect("the scope was either already held or pushed by the branch above");
    }
}

/// The lineages one state observed in one scope, each with the sentence it was described by.
///
/// A named wrapper rather than a bare map, because the two strings in it are a key and display
/// material and a caller handed a raw `BTreeMap<String, String>` cannot see which is which.
pub(super) struct ObservedLineages(BTreeMap<String, String>);

/// What a finished run knows that its own record needs.
///
/// Grouped into one value so [`Recorded_Census`] takes one parameter: four borrows that always
/// travel together, and a caller assembling them by position could transpose the two policy
/// halves without the compiler objecting.
pub(super) struct ObservedRun<'a>
{
    /// The record this run read before it judged anything, or `None` when there was none --
    /// which is what makes the file's presence the opt-in.
    pub(super) history: Option<&'a OccurrenceHistory>,
    /// What the run judged.
    pub(super) outcome: &'a CheckOutcome,
    /// The command it judged under, read for its root, its scope and its rule selection.
    pub(super) command: &'a GateCommand,
    /// The baseline the run resolved, which is what names the scopes worth censusing.
    pub(super) baseline: &'a BaselinePolicy,
}

/// `run`'s record advanced by the state this run observed, or `None` when it must not record.
///
/// The three refusals are this module's own doc's three guards, in the order they are cheapest
/// to answer. Each returns `None` rather than an empty census, because a census of nothing and
/// no census at all are the difference between reporting a scope emptied and reporting nothing
/// about it.
pub(super) fn Recorded_Census(run: &ObservedRun<'_>) -> Option<OccurrenceHistory>
{
    let history = run.history?;
    let CheckOutcome::Judged { findings, .. } = run.outcome
    else
    {
        return None;
    };

    if run.command.scope != ScopeSelector::default()
    {
        return None;
    }

    let selected: Vec<Finding> = findings.iter().filter(|finding| return run.command.rules.Is_Included(&finding.rule)).cloned().collect();
    let mut census = history.clone();

    for entry in Censused_Entries(run.baseline, &run.command.rules, &selected)
    {
        census.Scope_For(entry).Advanced_By(&Observed_In(entry, &selected));
    }

    return Some(census);
}

/// The entries in `baseline` whose scopes this run is entitled to census.
fn Censused_Entries<'a>(baseline: &'a BaselinePolicy, rules: &RuleSelector, selected: &[Finding]) -> Vec<&'a BaselineDebt>
{
    return baseline
        .debt
        .iter()
        .filter(|entry| return rules.Is_Included(&entry.rule) && Reached_A_Judgment(&entry.rule, selected))
        .collect();
}

/// Whether `rule` reached a judgment about everything it bound in this run.
///
/// `Claim_Of` over that rule's own findings, which is the existing answer to "did this run reach
/// a judgment" and not a second one written here: a `MissingCapability`, an unreadable subject
/// or an agent-required one makes the claim incomplete, and a rule that found nothing at all
/// makes it complete, which is exactly the distinction a census needs.
fn Reached_A_Judgment(rule: &RuleId, selected: &[Finding]) -> bool
{
    let its_own: Vec<Finding> = selected.iter().filter(|finding| return finding.rule == *rule).cloned().collect();

    return Claim_Of(&its_own) == Claim::Complete;
}

/// Every lineage `selected` holds inside `entry`'s own scope.
///
/// Matched by `BaselineDebt::Is_Applicable_To`, the same `rule`/`subject` test that decides
/// which findings the entry tolerates, so a census and a tolerance can never disagree about
/// which occurrences belong to a scope.
fn Observed_In(entry: &BaselineDebt, selected: &[Finding]) -> ObservedLineages
{
    let mut observed = BTreeMap::new();

    for finding in selected.iter().filter(|finding| return entry.Is_Applicable_To(finding))
    {
        observed.insert(Lineage_Key(finding), finding.summary.clone());
    }

    return ObservedLineages(observed);
}

/// The spelling a record keys one occurrence lineage by.
///
/// The digest's own thirty-two characters rather than its bytes, so the file is something a
/// person can read and grep. Nothing parses it back: a stored key is only ever compared against
/// one derived the same way, so there is no second spelling for a reader and a writer to
/// disagree about.
fn Lineage_Key(finding: &Finding) -> String
{
    return OccurrenceLineageId::Of(finding).Digest().to_string();
}

/// The spelling a record keys one scope's subject by, for the reason [`Lineage_Key`] gives.
fn Subject_Key(subject: SubjectId) -> String
{
    return subject.Digest().to_string();
}
