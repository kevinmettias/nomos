//! One occurrence lineage's sightings across the states a record kept for its scope.

use nomos_model::{IdentityTransitionKind, OccurrenceTally};
use serde::{Deserialize, Serialize};

/// What a record remembers about one occurrence lineage inside one scope.
///
/// Counters rather than a list of the states themselves, so the file stays the size of the
/// tree's own debt rather than growing with every run. [`OccurrenceTally`] is what the counters
/// are read through, and its own doc carries the argument that the totals answer the whole
/// question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::gate_environment) struct RecordedOccurrence
{
    /// The rule's own sentence, as the state that last saw this lineage spelled it.
    ///
    /// Carried so a person opening the record reads what the violation was rather than a
    /// thirty-two character digest. Nothing matches on it: the lineage digest is the key, and
    /// the sentence is one of the three things that digest was taken over, so comparing it too
    /// would be comparing the same material twice.
    pub(in crate::gate_environment) summary: String,
    /// The earliest state that observed this lineage, or zero when none has.
    pub(in crate::gate_environment) first_state: u32,
    /// The latest state that observed it.
    pub(in crate::gate_environment) last_state: u32,
    /// How many states observed it.
    pub(in crate::gate_environment) observed_states: u32,
    /// What the record established about this lineage at the state it now ends at, in the
    /// vocabulary `nomos-model` publishes.
    ///
    /// `Some(ExactContinuity)` and `Some(Recreated)` are what
    /// [`OccurrenceTally::Established_Continuity`] returned; `Some(Unresolved)` is the state
    /// `OD-GATE-030` calls the floor — the lineage is there and nothing establishes whether it
    /// persisted or came back, which is reported as undetermined rather than as persistent debt.
    ///
    /// `None` means the latest state did not observe this lineage at all. There is no continuity
    /// to claim about something that is not there, and writing `Unresolved` for it would say the
    /// record had looked and come away unsure.
    pub(in crate::gate_environment) continuity: Option<IdentityTransitionKind>,
}

impl RecordedOccurrence
{
    /// An entry for a lineage no state has observed yet, which is how every one of them starts.
    pub(super) fn Unseen() -> Self
    {
        return Self { summary: String::new(), first_state: 0, last_state: 0, observed_states: 0, continuity: None };
    }

    /// This entry after the state numbered `state` observed the lineage, described by `summary`.
    ///
    /// `first_state` is set only once: it is when the record first saw this lineage, and a later
    /// sighting must not move it or a recreation would read as an arrival.
    pub(super) fn Observed_At(&mut self, state: u32, summary: &str)
    {
        if self.first_state == 0
        {
            self.first_state = state;
        }

        self.last_state = state;
        self.observed_states = self.observed_states.saturating_add(1);
        summary.clone_into(&mut self.summary);
    }

    /// This entry's sightings as the tally `nomos-model` reads them, against a scope holding
    /// `recorded_states` states.
    pub(super) const fn Tally_In(&self, recorded_states: u32) -> OccurrenceTally
    {
        return OccurrenceTally {
            recorded_states,
            first_state: self.first_state,
            last_state: self.last_state,
            observed_states: self.observed_states,
        };
    }
}
