//! The ledger's own tunable numbers: how long to wait, how old is too old, and which schema
//! version this build writes.

use std::time::Duration;

/// How long to wait for the ledger lock before giving up.
pub const LOCK_WAIT_LIMIT: Duration = Duration::from_secs(20);

/// How old a ledger lock must be before it may be broken.
///
/// Generous relative to how long a ledger write takes — writes are milliseconds — so
/// that breaking one is genuinely evidence the holder died rather than evidence the
/// machine was briefly busy.
pub const LOCK_STALE_AFTER: Duration = Duration::from_secs(15 * 60);

/// The highest ledger schema version this build can account for.
///
/// Written into every file this build saves, and compared against a file this build failed to
/// read. It does not *guarantee* anything: the guarantee is `deny_unknown_fields` on every
/// container reachable from [`crate::LedgerDocument`], which is mechanical and cannot be
/// forgotten. This number's only job is to decide which sentence an operator whose parse just
/// failed reads — [`crate::LedgerError::Unrecognized`] rather than
/// [`crate::LedgerError::Malformed`].
///
/// That ordering is deliberate and is `OD-LEDGER-008`'s decision. `e88f92d` added a field to
/// [`crate::LedgerItem`] and raised nothing, so a guard resting on the bump would report clean
/// on the next instance of the defect it was built for. Here a forgotten bump can only degrade
/// a message, and can never cost a field.
///
/// `5` since `OD-LEDGER-024` added [`crate::LedgerItem::kind`] and
/// [`crate::LedgerItem::origin`]; `4` was `OD-LEDGER-027`'s
/// [`crate::VerificationRecord::revision`]; `3` was `OD-LEDGER-019`'s
/// [`crate::LedgerItem::declined`]; `2` was `OD-LEDGER-012`'s [`crate::LedgerItem::displaced`].
/// The bump is not discretionary: `Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version`
/// counts the keys on a serialized item, so a field arriving without this number moving is a
/// refusal that misstates why.
///
/// `kind` and `origin` carry no `#[serde(default)]`, unlike every field before them — every
/// item on the board was migrated to carry both in the same commit that raised this number,
/// so a build older than this one refuses the file outright rather than silently accepting a
/// row missing either. `nomos work validate` prints the same two numbers on request, which is
/// how to tell before that refusal arrives rather than at it.
pub const SCHEMA_VERSION: u32 = 5;
