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
//! The refusal is asymmetric on purpose: a new build still reads an old file, because every
//! added field carries `#[serde(default)]`, and an old build no longer reads a new one.
//! Forward compatibility for this file was only ever buying the ability to lose it.
//!
//! Nothing enumerates these attributes, because a hand-written list of types is only as
//! complete as the hand — `OD-COMPLETENESS-001`. What holds them in place is
//! `Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key`, which walks a fully
//! populated document and probes every object node it finds.

// An item's identity, the state it is in, what kind of work it is, and where it came from.
mod id;
mod kind;
mod origin;
mod state;

pub use id::ItemId;
pub use kind::ItemKind;
pub use origin::ItemOrigin;
pub use state::ItemState;

// Why an item is being declined, and the item schema itself — each its own single public
// type, and the second large enough on its own to want a file of its own.
mod decline_reason;
mod ledger_item;

pub use decline_reason::DeclineReason;
pub use ledger_item::LedgerItem;

#[cfg(test)]
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

impl<'a> From<&'a str> for Holder<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Holder(value);
    }
}

impl<'a> From<&'a String> for Holder<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Holder(value.as_str());
    }
}

impl<'a> Holder<'a>
{
    /// The holder's identifier as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
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
