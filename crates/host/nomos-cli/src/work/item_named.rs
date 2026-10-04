//! Which item `show --item <text>` names: the item whose id it is, or the one item whose id it
//! begins.

use nomos_ledger::{ItemId, LedgerDocument, LedgerItem};

/// What a text names on a board.
pub(super) enum Named<'a>
{
    /// The item whose id is the text, or the one item whose id begins with it and a hyphen.
    One(&'a LedgerItem),
    /// The text is no item's id, and begins, followed by a hyphen, every one of these.
    Several(Vec<&'a ItemId>),
    /// The text is no item's id and begins none.
    Nothing,
}

/// The item `text` names on `document`, for `show` and for nothing else.
///
/// An exact id wins outright, so an id that also begins a longer one still shows itself. Failing
/// that, the text names the one id that begins with it followed by a hyphen. The hyphen is the
/// boundary: `P19` must never name `P190-…`, and a bare prefix would. When several ids begin
/// with it, none is chosen, because any rule choosing one would be a guess.
///
/// Only `show` asks. Ids run past a hundred characters, and on 2026-09-26 96 of the 153
/// P-numbers in use were shared by more than one item, so a P-number alone names no item and a
/// reader needs some shorter way to name the one they want to read. A verb that changes the board still takes the whole id: acting on a
/// guess is acting on the wrong item the day the guess is wrong.
pub(super) fn Item_Named<'a>(document: &'a LedgerDocument, text: &ItemId) -> Named<'a>
{
    if let Some(exact) = document.items.iter().find(|candidate| return &candidate.id == text)
    {
        return Named::One(exact);
    }

    let stem = format!("{text}-");
    let beginning: Vec<&LedgerItem> =
        document.items.iter().filter(|candidate| return candidate.id.As_Text().starts_with(&stem)).collect();

    return match beginning.as_slice()
    {
        [] => Named::Nothing,
        [only] => Named::One(only),
        several => Named::Several(several.iter().map(|item| return &item.id).collect()),
    };
}
