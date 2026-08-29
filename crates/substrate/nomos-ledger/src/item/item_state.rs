//! Where an item stands in its own lifecycle.

use serde::Deserialize;
use serde::Serialize;
/// What state a ledger item is in.
///
/// A closed set. The compiler refuses a sixth, which is the point: a state that exists
/// only in one tool's imagination is a state no query can filter on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ItemState
{
    /// Available to claim.
    Ready,
    /// Someone is working on it.
    Claimed,
    /// Cannot proceed.
    Blocked,
    /// Finished, with its verification predicate having passed.
    Done,
    /// Deliberately not going to be done.
    ///
    /// Reached by `nomos work decline` and by nothing else. It was reachable by nothing at
    /// all until `OD-LEDGER-019`, which is what that record measures: a state the ledger
    /// could describe, filter on and count terminal, and could not enter, so a superseded
    /// item went back to [`ItemState::Ready`] and the board offered it again.
    Declined
    {
        /// Why not. Carried in the variant so an item cannot be declined reasonlessly.
        ///
        /// The only copy. [`Declination`] records who ended it and when, and deliberately
        /// does not repeat this — a reader asking *why* asks the state and a reader asking
        /// *who* asks the item, and neither can be told two different things.
        reason: String,
    },
}

impl ItemState
{
    /// Whether an item in this state may be claimed.
    #[must_use]
    pub const fn Is_Claimable(&self) -> bool
    {
        return matches!(self, Self::Ready);
    }

    /// Whether this state is terminal.
    #[must_use]
    pub const fn Is_Finished(&self) -> bool
    {
        return matches!(self, Self::Done | Self::Declined { .. });
    }

    /// This state as one line, for a refusal that has to name it.
    ///
    /// `Debug` was what every caller used, and it was fine for four of the five variants
    /// because they carry nothing. [`ItemState::Declined`] carries prose, and the moment the
    /// state became reachable that prose started arriving inside single-line refusals with
    /// its newlines escaped — the reason `P10-DERIVED-FACT` was declined with runs to five
    /// paragraphs, and `nomos work decline` on it printed all of them as one line.
    ///
    /// So the reason is kept and cut to its first line. Which of the two things a caller has
    /// hit — a duplicate, or a disagreement with somebody's reading — is decided by the first
    /// sentence, and the whole of it is one `work show` away. That split is the one the usage
    /// text already draws between the two verbs: `list` is a column per item, `show` is the
    /// one that can carry prose.
    ///
    /// One implementation, for the reason [`crate::Claim_Refusal`] is one function: two
    /// renderings of a state is how a listing and a refusal come to disagree about what an
    /// item is, which `OD-LEDGER-014` measured at the next layer up.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Declined { reason } =>
            {
                let first = reason.lines().next().unwrap_or_default();
                let elided = reason.lines().count() > 1;

                format!("Declined: {first}{}", if elided { " […]" } else { "" })
            }
            other => format!("{other:?}"),
        };
    }
}
