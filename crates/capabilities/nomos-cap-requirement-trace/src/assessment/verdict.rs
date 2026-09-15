//! What an assessment says about a requirement, and what that verdict owes.
//!
//! Ported from `tests/contract/tests/requirement_trace/assessment.rs`, which `OD-TRACE-001`
//! and `OD-TRACE-003` already settled the shape of. Its own file, because every scope here
//! declares one public type and a verdict is the one a caller writes.

/// What an assessment says about a requirement.
///
/// Four verdicts named in `OD-TRACE-001`, three of them writable there. `OD-TRACE-003`
/// adds a fifth, `Partial`, for the case those four could not name: a requirement that
/// binds this build and is satisfied at some of its sites and not at others. `Unassessed`
/// is still held by the *absence* of an entry and is refused as a written word by
/// [`crate::registry::Parse`] — a file saying `Unassessed` would be somebody looking and
/// recording that they had not, which is the one thing this registry must not be able to
/// express.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict
{
    /// The site satisfies the requirement, and the entry names where.
    Met,
    /// The site is derived from the requirement and departs from it deliberately. The
    /// entry names the governing record carrying the reason; a divergence with no record
    /// is not a verdict, it is the state `OD-TRACE-001` exists to end.
    Diverges,
    /// The requirement is read as not reaching this build, with a record saying why a
    /// corpus requirement does not bind the thing built to enforce it.
    NotBinding,
    /// The requirement binds this build and is satisfied at some of its sites and not at
    /// others. Unfinished is not a decision, so — unlike `Diverges` and `NotBinding` —
    /// this verdict owes no governing record; `OD-TRACE-003` is why. What it owes instead
    /// is at least one `gap`: a `path#symbol` site naming where the unsatisfied part
    /// actually lives, checked exactly like `sites` so the entry decays the same visible
    /// way if the gap closes or moves out from under it.
    Partial,
}

impl Verdict
{
    /// The word an entry is written with.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Met => "Met",
            Self::Diverges => "Diverges",
            Self::NotBinding => "NotBinding",
            Self::Partial => "Partial",
        };
    }

    /// Whether a verdict of this kind is owed a governing record.
    ///
    /// `Diverges` and `NotBinding` are. Departing from a normative requirement and
    /// declaring one out of scope are the same act from the corpus's side — somebody
    /// deciding this build will not do what the requirement says — and the reason is what
    /// makes either reviewable.
    ///
    /// `Partial` is not, even though it is neither `Met` nor a record-owing verdict.
    /// `OD-TRACE-003` draws that line deliberately: a half-finished requirement is not a
    /// decision anybody made, so demanding a record for it would demand a reason for
    /// something that has none. Its obligation is a gap instead — see [`Self::Partial`].
    #[must_use]
    pub const fn Owes_A_Record(self) -> bool
    {
        return matches!(self, Self::Diverges | Self::NotBinding);
    }
}
