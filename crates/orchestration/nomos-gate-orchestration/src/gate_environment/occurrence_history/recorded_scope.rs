//! One `rule`/`subject` scope's own states, and the lineages each of them observed.

use nomos_model::{IdentityTransitionKind, OccurrenceTally};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{ObservedLineages, RecordedOccurrence};

/// What a record remembers about one scope a baseline entry addresses.
///
/// The scope is the unit for the same reason `OD-GATE-030` made it the unit of the counting
/// bound: an entry accepts a population for a `rule`/`subject` pair, so how many states have
/// been censused is a fact about the pair and not about any one occurrence in it. A per-lineage
/// state count would let two lineages in one scope disagree about how many runs had happened.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::gate_environment) struct RecordedScope
{
    /// The rule this scope belongs to, spelled as the rule identifier a person writes.
    pub(in crate::gate_environment) rule: String,
    /// The subject digest, as the thirty-two characters it prints as.
    ///
    /// The digest and not the path an entry declared: `BaselineDebt::declared_path`'s own doc
    /// says a spelling is display material and two spellings are one scope, so a record keyed on
    /// the spelling would split one scope in two over a `./` its author cannot see.
    pub(in crate::gate_environment) subject: String,
    /// How many states this record holds for the scope, the most recent numbered by it.
    pub(in crate::gate_environment) states: u32,
    /// Every lineage any of those states observed, keyed by its digest.
    pub(in crate::gate_environment) occurrences: BTreeMap<String, RecordedOccurrence>,
}

impl RecordedScope
{
    /// A scope no state has been recorded for yet.
    pub(super) fn Empty(rule: String, subject: String) -> Self
    {
        return Self { rule, subject, states: 0, occurrences: BTreeMap::new() };
    }

    /// What this scope's states say about `lineage`, as the tally `nomos-model` reads.
    ///
    /// A lineage this scope has never seen tallies as observed by none of its states, which is
    /// not the same as a scope with no states at all — and [`OccurrenceTally`] answers the two
    /// differently, which is the whole reason the unseen case is spelled rather than defaulted.
    pub(super) fn Tally_Of(&self, lineage: &str) -> OccurrenceTally
    {
        return self
            .occurrences
            .get(lineage)
            .map_or(OccurrenceTally { recorded_states: self.states, ..OccurrenceTally::default() }, |occurrence| {
                return occurrence.Tally_In(self.states);
            });
    }

    /// This scope advanced by one state, which observed exactly `observed`.
    ///
    /// Every lineage the state saw is recorded as seen; every lineage it did not is left alone,
    /// and the state count moving past its `last_state` is what later reads as a gap. Nothing is
    /// removed, because a lineage that went away is precisely the evidence a recreation is
    /// established from.
    pub(super) fn Advanced_By(&mut self, observed: &ObservedLineages)
    {
        self.states = self.states.saturating_add(1);
        let state = self.states;

        for (lineage, summary) in &observed.0
        {
            self.occurrences.entry(lineage.clone()).or_insert_with(RecordedOccurrence::Unseen).Observed_At(state, summary);
        }

        self.Restated_Continuity(observed);
    }

    /// Every occurrence's `continuity`, recomputed against the state this scope now ends at.
    ///
    /// Recomputed rather than carried forward, so the field a reader sees always agrees with the
    /// counters printed beside it. A lineage the new state did not observe is left with no claim
    /// at all; one it did carries what the tally established, or `Unresolved` when it
    /// established nothing, which is `OD-GATE-030`'s floor written into the record.
    fn Restated_Continuity(&mut self, observed: &ObservedLineages)
    {
        let states = self.states;

        for (lineage, occurrence) in &mut self.occurrences
        {
            let established = occurrence.Tally_In(states).Established_Continuity();
            occurrence.continuity =
                observed.0.contains_key(lineage).then(|| return established.unwrap_or(IdentityTransitionKind::Unresolved));
        }
    }
}
