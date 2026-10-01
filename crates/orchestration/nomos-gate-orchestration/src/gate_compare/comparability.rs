//! Whether a difference between two runs may be attributed to the repository.

use crate::JudgmentDifference;
use nomos_contracts::RunId;

/// Whether a difference between two runs may be attributed to the repository.
///
/// `OD-GATE-031`'s invariant, made into a value: a comparison attributes a difference to
/// repository state only when the non-source judgment inputs of its two sides were
/// compatible, or their differences are explicitly represented — never by the absence of
/// evidence that they differed.
///
/// # Why this qualifies rather than refuses
///
/// A refusal would throw away the findings diff, which is real information the caller asked
/// for, in order to report something *about* it. `P106` settled the same question one verb
/// over, where a run that reaches no verdict still prints every finding it found: refusing
/// the verdict is not refusing the answer. So a comparison always reports what moved, and
/// says alongside it what a reader is entitled to conclude from that.
///
/// # Why the moment is not one of these
///
/// Two runs essentially always have different moments, so classifying that as a discrepancy
/// would put a line on every comparison ever made, which is how a reader learns to skip the
/// line that matters. And deciding whether a moment difference *could* have changed anything
/// means knowing whether a waiver expired between the two, which needs the policy itself
/// rather than the digest `crate::GateRunProvenance` carries of it.
///
/// What this must not do is imply the moments were equal, and it does not: a caller holds
/// both runs, each carries its own moment, and `Compatible` is a statement about the policy,
/// the selection and the instrument rather than about time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Comparability
{
    /// Both sides recorded what judged them, and the policy, the selection and the instrument
    /// all agree. A difference is a difference in the repository, and may be read as one.
    Compatible,
    /// Both sides recorded what judged them, and something other than the source differed
    /// too. The differences are named so that a reader attributes the change themselves.
    ///
    /// Never empty: an empty list is [`Self::Compatible`], and constructing one here would
    /// make "there were stated differences" true of a comparison that had none.
    CompatibleWith(Vec<JudgmentDifference>),
    /// At least one side does not say what judged it, so nothing can be attributed either
    /// way. Names the runs that did not.
    ///
    /// `OD-GATE-031` decided this rather than assuming a match, for the reason `Applicability`
    /// keeps `MissingCapability` apart from a clean result: nothing could look must never read
    /// as nothing was wrong. A comparability claim made from missing evidence is precisely the
    /// failure the whole mechanism exists to prevent.
    Incomparable(Vec<RunId>),
}
