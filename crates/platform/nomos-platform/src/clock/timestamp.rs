//! A point in time, as everything in Nomos records one.
//!
//! # Why this file defines the type again
//!
//! It defined it once, and the definition was carried down into XVPE on 2026-09-11 as
//! `xvpe_clock::Timestamp` under `OD-PLATFORM-003`, leaving a re-export here.
//! `OD-ROADMAP-005` decision 4 brings the declaration back, because the re-export made
//! the ports crate every band above depends on unable to compile without the engine
//! underneath it — a dependency running the wrong way through the seam this crate
//! exists to be.
//!
//! That is the whole of what moved. The crossing stays adopted and stays pinned,
//! `nomos-platform-xvpe` is still the adapter, and Nomos is still an application over
//! XVPE rather than a peer of it. XVPE keeps `xvpe_clock::Timestamp` for its own
//! consumers; this one is Nomos's, and the two are no longer one declaration.
//!
//! What a timestamp *is* did not change with the declaration: second resolution, an
//! advance that saturates rather than wraps, and a backwards clock reporting zero
//! elapsed. The cases that assert those three live in [`super`], beside the [`Clock`]
//! whose readings they are about, and they passed unchanged across both moves.
//!
//! [`Clock`]: super::Clock

/// A point in time, as whole seconds since the Unix epoch.
///
/// # Why seconds
///
/// Everything that records one of these writes it into a line a person reads — a ledger
/// entry, a lease, a receipt. Sub-second digits would churn every record without telling
/// that reader anything they act on. Where ordering matters more finely than a second,
/// carry the ordering explicitly rather than inferring it from a clock that is allowed to
/// step backwards.
///
/// # Why this is not an elapsed time
///
/// This answers *when*, which is what a record needs. How long since is a different
/// question with a different answer — a wall reading is not monotonic, because it steps
/// when the machine corrects, resumes or crosses a leap, so a duration derived from two
/// of these is a bound rather than a measurement. [`Since`](Self::Since) is that bound
/// and says so.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp
{
    /// Wraps whole seconds since the Unix epoch.
    #[must_use]
    pub const fn From_Unix_Seconds(seconds: i64) -> Self
    {
        return Self(seconds);
    }

    /// Whole seconds since the Unix epoch.
    #[must_use]
    pub const fn Unix_Seconds(self) -> i64
    {
        return self.0;
    }

    /// This timestamp advanced by a duration, saturating at the representable range.
    ///
    /// Saturates rather than wrapping: a wrap at the far end of the range reads as a time
    /// in the distant past, which inverts every comparison made against it.
    #[must_use]
    pub fn Plus(self, duration: std::time::Duration) -> Self
    {
        let seconds = i64::try_from(duration.as_secs()).unwrap_or(i64::MAX);

        return Self(self.0.saturating_add(seconds));
    }

    /// How long after `earlier` this timestamp is, or zero if it is not after it.
    ///
    /// Zero rather than an underflowed maximum. A clock that went backwards — a
    /// correction, a virtual machine resuming, a board with a dead battery — must not
    /// produce a wildly large elapsed time, because every deadline measured against it
    /// would read as long expired at once.
    #[must_use]
    pub fn Since(self, earlier: Self) -> std::time::Duration
    {
        let elapsed = self.0.saturating_sub(earlier.0);

        return std::time::Duration::from_secs(u64::try_from(elapsed).unwrap_or(0));
    }
}

/// Reading and writing a [`Timestamp`] as the work ledger already holds one.
///
/// # Why an adapter rather than a derive
///
/// The derive is available — [`Timestamp`] is declared in this crate again — and is
/// deliberately not taken, which is a different reason from the one that held while the
/// type was XVPE's. It is not a format choice: a newtype struct is transparent to serde,
/// so `#[derive(Serialize, Deserialize)]` would emit exactly the bytes these two
/// functions do. It is that the bytes are an obligation rather than a property.
/// `work/ledger.json` is committed and read under `git diff`, so what a timestamp looks
/// like in it is a compatibility debt this workspace owes its own history, and not a
/// property of what a timestamp *is*. A named pair of functions with cases asserting the
/// bytes states that debt where a derive would leave it implicit in a trait
/// implementation nobody reads.
///
/// The format is a bare JSON number of whole seconds — byte-for-byte what the
/// `#[derive(Serialize)]` this replaced produced. A ledger written before that change
/// reads back identically after it, which this module's own cases assert directly
/// rather than leaving to this comment.
///
/// Applied at each field as a pair of attributes naming one function below by its full
/// path — `#[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]`
/// and `#[serde(deserialize_with = "nomos_platform::timestamp_serde::Read_Unix_Seconds")]`.
/// Serde documents that pair as equivalent to its `with = "module"` form, and it is spelled
/// out here because `with` requires exactly the names `serialize` and `deserialize`, which
/// say nothing about what is written or read. Ten fields across nine types name them, all of
/// them a bare `Timestamp` — there is no `Option` or collection shape to serve, which is why
/// this module has one pair of functions and not a family. Four of the nine are the
/// `nomos-api` response types, which derive `Serialize` alone and carry only the writing
/// half.
///
/// The module is named for the path it is reached by, rather than `serialization` and
/// re-exported under a shorter name: an alias is a second public name for one item, and
/// this one was the only reason `clock` published two paths to the same module.
pub mod timestamp_serde
{
    use serde::{Deserialize, Deserializer, Serializer};

    use super::Timestamp;

    /// Writes the timestamp as whole seconds since the Unix epoch.
    ///
    /// # Errors
    ///
    /// Whatever the serializer reports for writing an `i64`.
    pub fn Write_Unix_Seconds<Format: Serializer>(timestamp: &Timestamp, serializer: Format) -> Result<Format::Ok, Format::Error>
    {
        return serializer.serialize_i64(timestamp.Unix_Seconds());
    }

    /// Reads whole seconds since the Unix epoch back into a timestamp.
    ///
    /// # Errors
    ///
    /// Whatever the deserializer reports for a value that is not an `i64`.
    pub fn Read_Unix_Seconds<'de, Format: Deserializer<'de>>(deserializer: Format) -> Result<Timestamp, Format::Error>
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
        #[serde(serialize_with = "super::timestamp_serde::Write_Unix_Seconds")]
        #[serde(deserialize_with = "super::timestamp_serde::Read_Unix_Seconds")]
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
