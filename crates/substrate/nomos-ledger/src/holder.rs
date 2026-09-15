//! What a unit of work is, and what it means to have finished one.
//!
//! # Why every container here refuses a key it does not declare
//!
//! Each type below carries `#[serde(deny_unknown_fields)]`, and the reason is stated once
//! here rather than eight times below. Serde's default is to ignore an unrecognized key, so
//! a binary built before a field existed read the current ledger, dropped that field, wrote
//! the document back and exited 0 — the state change surviving and the data not. Refusing
//! the parse is what makes such a writer stop at the door instead of succeeding quietly.
//! `OD-LEDGER-008` records the decision and what it does not reach.
//!
//! The refusal is asymmetric on purpose, and the two directions have two different
//! mechanisms. An old build no longer reads a new file: `deny_unknown_fields` refuses the
//! parse on a key it does not know, which is mechanical and applies to any field ever added.
//! Forward compatibility for this file was only ever buying the ability to lose it.
//!
//! Whether a *new* build reads an *old* file is decided per field by `#[serde(default)]`, and
//! it is not uniform. The fields added before `OD-LEDGER-024` carry one and are read as empty
//! on a row written before they existed, which for each of them is exactly true. `kind`,
//! `origin` and `widened` carry none, so a row missing any of the three is refused and the
//! board is migrated instead — an absence that a reader could not tell from an oversight is
//! worth more than the convenience of accepting it.
//!
//! Neither direction is the schema number's doing. `SCHEMA_VERSION` is consulted *after* a
//! parse has already failed and decides only which sentence the operator reads, which
//! `OD-LEDGER-008` chose deliberately: a field once arrived without the number moving, so a
//! guard resting on the bump would have reported clean on the next instance of the defect it
//! was built for.
//!
//! Nothing enumerates these attributes, because a hand-written list of types is only as
//! complete as the hand — `OD-COMPLETENESS-001`. What holds them in place is
//! `Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key`, which walks a fully
//! populated document and probes every object node it finds.

// An item's identity, the state it is in, what kind of work it is, and where it came from.
#[path = "item/id.rs"]
mod id;
#[path = "item/kind.rs"]
mod kind;
#[path = "item/origin.rs"]
mod origin;
#[path = "item/state.rs"]
mod state;

pub use id::Id as ItemId;
pub use kind::Kind as ItemKind;
pub use origin::Origin as ItemOrigin;
pub use state::State as ItemState;

// Why an item is being declined, and the item schema itself — each its own single public
// type, and the second large enough on its own to want a file of its own.
#[path = "item/decline_reason.rs"]
mod decline_reason;
#[path = "item/ledger_item.rs"]
mod ledger_item;
#[path = "item/widening.rs"]
mod widening;

pub use decline_reason::DeclineReason;
pub use ledger_item::LedgerItem;
pub use widening::Widening;

#[cfg(test)]
#[path = "item/tests.rs"]
mod tests;

use std::time::Duration;

/// Who is declining an item, distinguished from [`DeclineReason`] purely by type.
///
/// The two travel as adjacent parameters through every layer of `Decline` —
/// [`LedgerItem::Decline`], [`crate::FileLedger::Decline`], and the verb body beneath it —
/// and two same-typed strings at any of those layers would let a caller swap who is
/// declining for why and have the compiler accept it. `From<&str>` and `From<&String>` both
/// convert into one, so no existing call site needs to change shape to adopt it: every one
/// already passes a borrowed string.
#[derive(Clone, Copy, Debug)]
pub struct Holder<'a>(&'a str);

impl<'a, Text> From<&'a Text> for Holder<'a>
where
    Text: AsRef<str> + ?Sized,
{
    fn from(value: &'a Text) -> Self
    {
        return Holder(value.as_ref());
    }
}

impl<'a> Holder<'a>
{
    /// The holder's identifier as a plain string.
    #[must_use]
    pub fn As_Text(&self) -> &'a str
    {
        return self.0;
    }
}

/// A narrow, file-local proof for [`Holder::As_Text`], addressed by name.
///
/// [`tests`] above is `item/tests.rs`, a separate physical file whose behavioural suite this
/// does not repeat or replace. `check-test-coverage`'s Rust front end keys a test's companion
/// unit off the literal file it is textually written in, so a test living in that separate
/// file can never address a function declared here, however it is named — this module gives
/// [`Holder::As_Text`] the one-file address the check reads. Named apart from `tests` because
/// this file already declares that name for the `#[path]`-mapped module above.
#[cfg(test)]
mod holder_tests
{
    use super::*;

    #[test]
    fn Test_As_Text_Should_Return_The_Wrapped_String_Unchanged()
    {
        let name = "agent-a".to_owned();
        let holder: Holder<'_> = Holder::from(&name);

        assert_eq!(holder.As_Text(), "agent-a");
    }
}

/// The lease a claim gets when the holder does not ask for a specific one.
///
/// Long enough that an agent working a real item is not interrupted; short enough that
/// an agent which died at lunchtime does not hold territory until tomorrow.
pub const DEFAULT_LEASE: Duration = Duration::from_secs(2 * 60 * 60);

/// The longest lease anyone may hold.
///
/// A ceiling, not a suggestion. Without one, "just set it to a year" is the obvious
/// workaround for an agent that keeps getting interrupted, and the ledger stops
/// excluding anything.
pub const MAXIMUM_LEASE: Duration = Duration::from_secs(14 * 24 * 60 * 60);
