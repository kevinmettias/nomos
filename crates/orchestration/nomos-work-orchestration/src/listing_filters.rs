//! What a listing narrows its rows to, beyond how much of the board it draws them from.

mod admission;

pub use admission::Admission;

use nomos_ledger::{ItemState, LedgerItem, Territory};

/// The two questions a listing can narrow its rows by: which items touch a path, and which
/// mention a text.
///
/// `OD-LEDGER-041` version 2 is why these exist. Agents asked the board both questions by
/// scripting against `work/ledger.json`, because no verb answered either, and a script read
/// the file around `OD-LEDGER-008`'s schema guard. The record decided to answer them as
/// filters on the listing, not as a second file, a query language or a board served to another
/// process.
///
/// Carried in [`crate::WorkCommand::List`] beside the state and the scope, and for the reason
/// they are carried there: the request is where a caller says what it wants, whichever
/// composition root renders the answer. Unlike those two, the answer is decided here rather
/// than by each renderer, because each filter is a question about one item that two adapters
/// must never answer differently -- [`ListingFilters::Admits`] is that answer.
///
/// Neither changes a bound. A filter narrows the rows the state or the scope already admits,
/// and an empty one admits every row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListingFilters
{
    /// Only items whose territory the claim check would treat as overlapping a territory of
    /// this one path.
    pub touching: Option<String>,
    /// Only items whose id, title, `why`, `done_when` or a recorded reason contains this text,
    /// literally and case-sensitively.
    pub mentions: Option<String>,
}

impl ListingFilters
{
    /// Whether no filter was given, so every row the bound admits is listed.
    #[must_use]
    pub const fn Is_Empty(&self) -> bool
    {
        return self.touching.is_none() && self.mentions.is_none();
    }

    /// Whether every filter given admits `item`, and how decided that answer is.
    ///
    /// `None` when a filter leaves the item out. The two compose by both having to admit it.
    #[must_use]
    pub fn Admits(&self, item: &LedgerItem) -> Option<Admission>
    {
        if let Some(text) = &self.mentions
            && !Mentions(item, text)
        {
            return None;
        }

        let Some(path) = &self.touching
        else
        {
            return Some(Admission::Admitted);
        };

        return Touches(item, path);
    }
}

/// Whether `item`'s territory overlaps a territory of `path` alone, decided by
/// [`Territory::Intersect`] -- the intersection a claim's exclusion is decided by.
///
/// No second overlap rule is written here: containment in either direction, and a record file
/// folding onto its identifier, are that intersection's own answers. Only the claim check's
/// reading of the answer is repeated, and it is `Intersection::Permits_Concurrency`'s: an
/// answer that does not permit two claims at once is an overlap or an undecided one, and
/// neither is left out.
///
/// The two are told apart by `Intersection::Conflicting`. [`Territory::Intersect`]
/// answers `Overlaps` only with the subjects both sides reach, never with none, so an answer
/// that refuses concurrency while naming no conflicting subject is its `Unknown` arm -- an
/// unexpanded pattern, or a territory authored at another resolution.
fn Touches(item: &LedgerItem, path: &str) -> Option<Admission>
{
    let intersection = Territory::Of_Files([path]).Intersect(&item.territory);
    if intersection.Permits_Concurrency()
    {
        return None;
    }
    if intersection.Conflicting().is_empty()
    {
        return Some(Admission::Undecided);
    }

    return Some(Admission::Admitted);
}

/// Whether `text` occurs, literally and case-sensitively, in `item`'s id, title, `why`,
/// `done_when` or a reason recorded on it.
///
/// The territory is not read: which items touch a path is `touching`'s question, and a text
/// filter that also matched paths would answer it a second way, by substring.
fn Mentions(item: &LedgerItem, text: &str) -> bool
{
    let written = [item.id.As_Text(), item.title.as_str(), item.why.as_str(), item.done_when.as_str()];

    return written.into_iter().chain(Recorded_Reasons(item)).any(|field| return field.contains(text));
}

/// Every reason recorded on `item`: why it was declined, and why each claim on it was abandoned.
///
/// Those are the two transitions the ledger takes a reason for. A displacement, a widening and a
/// verification record who and when, and carry no reason to read.
fn Recorded_Reasons(item: &LedgerItem) -> impl Iterator<Item = &str>
{
    let declined = match &item.state
    {
        ItemState::Declined { reason } => Some(reason.as_str()),
        ItemState::Ready | ItemState::Claimed | ItemState::Blocked | ItemState::Done => None,
    };

    return declined.into_iter().chain(item.abandoned.iter().map(|abandonment| return abandonment.reason.as_str()));
}

#[cfg(test)]
mod tests;
