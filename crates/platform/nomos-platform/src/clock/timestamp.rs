//! A point in time, as everything in Nomos records one.
//!
//! # Why this file no longer defines the type
//!
//! It did, and the definition was carried down into XVPE on 2026-09-11 as
//! `xvpe_clock::Timestamp`, under `OD-PLATFORM-003`: Nomos is an application over
//! that engine, and a wall-clock value type with saturating arithmetic is exactly
//! the domain-neutral capability `D-135` sends there. What was here — second
//! resolution, a backwards clock reporting zero elapsed, an advance that saturates
//! rather than wraps — is what moved, unchanged and with its tests.
//!
//! What stays here is the one thing that is genuinely Nomos's: how a timestamp is
//! written into the work ledger. See [`serialization`].

pub use xvpe_clock::Timestamp;

/// Reading and writing a [`Timestamp`] as the work ledger already holds one.
///
/// # Why an adapter rather than a derive
///
/// [`Timestamp`] is XVPE's now, and `xvpe-clock` is a `no_std` crate with no serde
/// dependency — deliberately, because a foundation crate down there does not take a
/// third-party dependency to serve one consumer up here. So the wire format is
/// Nomos's own concern, which is the right place for it: `work/ledger.json` is
/// committed and read under `git diff`, and the representation is a compatibility
/// obligation this workspace owes its own history, not a property of what a
/// timestamp *is*.
///
/// The format is a bare JSON number of whole seconds — byte-for-byte what the
/// previous `#[derive(Serialize)]` on a newtype produced, because a newtype struct
/// is transparent to serde. A ledger written before this change reads back
/// identically after it, which this module's own cases assert directly rather than
/// leaving to this comment.
///
/// Applied at each field with `#[serde(with = "nomos_platform::timestamp_serde")]`.
/// Nine fields across eight types name it, all of them a bare `Timestamp` — there is
/// no `Option` or collection shape to serve, which is why this module has one pair of
/// functions and not a family.
pub mod serialization
{
    use serde::{Deserialize, Deserializer, Serializer};

    use super::Timestamp;

    /// Writes the timestamp as whole seconds since the Unix epoch.
    ///
    /// # Errors
    ///
    /// Whatever the serializer reports for writing an `i64`.
    pub fn serialize<S: Serializer>(timestamp: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    {
        return serializer.serialize_i64(timestamp.Unix_Seconds());
    }

    /// Reads whole seconds since the Unix epoch back into a timestamp.
    ///
    /// # Errors
    ///
    /// Whatever the deserializer reports for a value that is not an `i64`.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Timestamp, D::Error>
    {
        let seconds = i64::deserialize(deserializer)?;

        return Ok(Timestamp::From_Unix_Seconds(seconds));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The instant the round-trip cases use. An arbitrary real one.
    const AN_INSTANT: i64 = 1_786_344_651;

    /// The obligation this adapter exists to keep.
    const THE_LEDGER_FORMAT_IS_A_BARE_NUMBER: &str =
        "a timestamp is written as whole seconds, exactly as every committed ledger holds it";

    /// A field as the ledger carries one, so the assertion is about the shape a
    /// record actually has rather than about a value serialized on its own.
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
    struct Record
    {
        #[serde(with = "super::serialization")]
        acquired_at: Timestamp,
    }

    /// `work/ledger.json` holds `"acquired_at": 1786344651`. It must keep doing so:
    /// the file is committed, and a reader is a `git diff`.
    #[test]
    fn Test_A_Timestamp_Should_Serialize_As_A_Bare_Number_Of_Seconds()
    {
        let record = Record { acquired_at: Timestamp::From_Unix_Seconds(AN_INSTANT) };

        let written = serde_json::to_string(&record).expect("a record serializes");

        assert_eq!(
            written,
            format!("{{\"acquired_at\":{AN_INSTANT}}}"),
            "{THE_LEDGER_FORMAT_IS_A_BARE_NUMBER}"
        );
    }

    /// The other half: a ledger written before this change still reads.
    #[test]
    fn Test_A_Ledger_Written_As_A_Bare_Number_Should_Still_Read()
    {
        let written = format!("{{\"acquired_at\":{AN_INSTANT}}}");

        let read: Record = serde_json::from_str(&written).expect("a record deserializes");

        assert_eq!(
            read.acquired_at,
            Timestamp::From_Unix_Seconds(AN_INSTANT),
            "{THE_LEDGER_FORMAT_IS_A_BARE_NUMBER}"
        );
    }

    /// A negative instant is a real value the type admits, and the shape must not
    /// change for it — a quoted or object-wrapped negative would be a silent format
    /// break that only a pre-epoch ledger would find.
    #[test]
    fn Test_A_Timestamp_Before_The_Epoch_Should_Round_Trip()
    {
        let record = Record { acquired_at: Timestamp::From_Unix_Seconds(-AN_INSTANT) };

        let written = serde_json::to_string(&record).expect("a record serializes");
        let read: Record = serde_json::from_str(&written).expect("a record deserializes");

        assert_eq!(written, format!("{{\"acquired_at\":{}}}", -AN_INSTANT));
        assert_eq!(read, record, "{THE_LEDGER_FORMAT_IS_A_BARE_NUMBER}");
    }
}
