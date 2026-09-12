//! An item refused, so nobody claims it again unread.

use serde::Deserialize;
use serde::Serialize;
use nomos_platform::Timestamp;
/// Who ended an item, and when.
///
/// The same job [`Abandonment`] does, for the transition [`ItemState::Declined`] is the
/// result of, and it exists for the same reason: a state is written down and a transition is
/// not, so who performed it and when are gone at the moment it happens unless a field on the
/// item is given the job of holding them.
///
/// It carries **no reason**, and that absence is the design rather than an omission. The
/// reason is the state's own content — [`ItemState::Declined`] is what makes an item
/// impossible to decline reasonlessly — and a second copy here would be one fact with two
/// homes that can come to disagree. `OD-LEDGER-019` decision 3.
///
/// The two fields differ from [`Abandonment`]'s in what they are about. An abandonment is
/// about a *claim*, so its holder is the agent that was holding the item. A declination is
/// about the *item*, and the item need not have been held by anybody — both of the items this
/// verb was built for were unclaimed — so `holder` here is whoever ran the verb.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declination
{
    /// Who ended it.
    pub holder: String,
    /// When they ended it.
    #[serde(with = "nomos_platform::timestamp_serde")]
    pub declined_at: Timestamp,
}
