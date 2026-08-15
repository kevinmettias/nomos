//! A second capability, derived from the first.
//!
//! # Why the slice needs one
//!
//! Syntax facts are leaves: each one is computed from a file and depends on nothing. A
//! corpus of nothing but leaves cannot demonstrate that invalidation reaches *descendants*,
//! because there are none — and "touching one file recomputes exactly its descendants"
//! would be satisfied trivially by any implementation, including one with the dependency
//! propagation deleted.
//!
//! So the slice computes a rollup: the public surface of a directory, read through
//! [`nomos_analysis::Reader`] from the syntax facts of the files in it. The Reader records
//! a dependency edge per file it reads, nobody writes those edges by hand, and the edges
//! are what a change has to travel along.
//!
//! # Why it declares Project granularity
//!
//! A rollup over a directory cannot refresh half a directory. Declaring `File` would be a
//! provider claiming a precision it does not have, and the invalidation engine would
//! refresh one file's contribution and treat the rest of the rollup as current. Declaring
//! `Project` makes the engine broaden a file-granular cause to a directory, and
//! [`nomos_analysis::InvalidationReport::broadened`] records that it did — which is the
//! cost of the rollup, stated rather than absorbed.

use nomos_capability::{CapabilityContract, ProviderOffer};
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    ProviderId, SchemaId,
};

pub const CAPABILITY: &str = "nomos.cap.module.surface";
pub const PROVIDER: &str = "nomos.slice.surface.rollup";
pub const SCHEMA: &str = "nomos.module.surface.v1";
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// What the rollup promises.
///
/// Every axis is at most what its inputs were. The variant cannot exceed
/// [`FactVariant::Syntactic`] because a count of syntactic facts is a syntactic fact, and
/// completeness stays [`Assurance::Unknown`] because a rollup over inputs that may have
/// missed macro-generated items has missed them too. `EvidenceClass::Derived` says the
/// same thing about provenance: a derivation is no stronger than what it derived from.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: CapabilityId::New(CAPABILITY),
        version: CONTRACT_VERSION,
        summary: "How much a directory of source files declares publicly, rolled up from \
                  the items each file declares."
            .to_owned(),
        ceiling: Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        ),
    };
}

#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: CapabilityId::New(CAPABILITY),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// What a directory declares.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Surface
{
    /// Files whose syntax facts were readable. Not the number of files in the directory:
    /// a member whose fact could not be read is counted in `unreachable`, because a
    /// rollup over three of four files that reports four is a lie about its own basis.
    pub files: u32,
    /// Members whose syntax fact the rollup could not read.
    pub unreachable: u32,
    /// Members whose syntax fact came from a weaker provider than the run asked for.
    ///
    /// `OD-CAPABILITY-003`'s third condition, and the field that keeps per-subject fallback
    /// from being a laundering machine. Without it the scanner covers the file the parser
    /// refused, `unreachable` drops to zero, and a rollup over five parsed files and one
    /// pattern-matched one is byte-identical to a rollup over six parsed ones.
    ///
    /// Counted in `files` as well, because the member did answer. This says how much of
    /// that answer is an approximation, not how much of it is missing.
    pub approximate: u32,
    pub items: u32,
    pub public: u32,
}

impl Surface
{
    /// The canonical byte encoding, by the same rules and for the same reasons as
    /// `nomos-lang-rust`'s: line-oriented, tab-separated, `\n` only, written by hand so
    /// no dependency's field ordering can silently change every digest in the store.
    #[must_use]
    pub fn Encode(&self) -> Vec<u8>
    {
        return format!(
            "files\t{}\nunreachable\t{}\napproximate\t{}\nitems\t{}\npublic\t{}\n",
            self.files, self.unreachable, self.approximate, self.items, self.public
        )
        .into_bytes();
    }
}

/// Reads a surface payload back.
///
/// # Errors
///
/// Returns the offending line when the payload is not a surface. A decoder that returned
/// `Surface::default()` for unrecognized bytes would report every unreadable rollup as a
/// directory that declares nothing.
pub fn Decode_Surface(payload: &[u8]) -> Result<Surface, String>
{
    let text = core::str::from_utf8(payload).map_err(|error| return error.to_string())?;
    let mut surface = Surface::default();
    let mut seen = 0_u8;

    for line in text.lines()
    {
        Read_One_Field(&mut surface, line)?;
        seen = seen.saturating_add(1);
    }
    if seen != 5
    {
        return Err(format!("a surface has five fields and this had {seen}"));
    }

    return Ok(surface);
}

/// One `field\tcount` line folded into the surface being read.
fn Read_One_Field(surface: &mut Surface, line: &str) -> Result<(), String>
{
    let Some((field, value)) = line.split_once('\t')
    else
    {
        return Err(format!("`{line}` is not a field"));
    };
    let value: u32 = value
        .parse()
        .map_err(|_| return format!("`{value}` in `{field}` is not a count"))?;

    match field
    {
        "files" => surface.files = value,
        "unreachable" => surface.unreachable = value,
        "approximate" => surface.approximate = value,
        "items" => surface.items = value,
        "public" => surface.public = value,
        other => return Err(format!("`{other}` is not a surface field")),
    }

    return Ok(());
}

/// Counts the publicly declared items in a syntax payload.
///
/// # Why the slice no longer decodes for itself
///
/// It used to, and the reason was that a shared struct would make the payload's writer and
/// its reader agree by construction and prove nothing. That reason was about the *writer*
/// and it still holds: both providers author these bytes by hand and neither shares an
/// encoder with the other or with anybody.
///
/// It never applied to the reader. This walk was a third opinion about what a well-formed
/// payload is, and it was the laxest of the three — it accepted an empty payload as a
/// directory that declares nothing, which is the rollup counting a file it could not read
/// as a file with nothing in it. `P10-SYNTAX-SCHEMA` moved the grammar and one reader into
/// `nomos-cap-syntax`, which is owned by neither provider, so using it is not agreeing with
/// either writer by construction.
///
/// # Errors
///
/// Returns what the schema's reader refused, for anything that is not a syntax payload.
pub fn Public_Items(payload: &[u8]) -> Result<(u32, u32), String>
{
    let decoded = nomos_cap_syntax::Parse_Payload(payload)
        .map_err(|refusal| return refusal.Describe())?;

    let items = u32::try_from(decoded.items.len()).unwrap_or(u32::MAX);
    let public = decoded
        .items
        .iter()
        .filter(|item| return item.Is_Public())
        .count();

    return Ok((items, u32::try_from(public).unwrap_or(u32::MAX)));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_lang_rust::{Encode_Payload, Read_Source, Reading};

    fn Encoded(source: &str) -> Vec<u8>
    {
        let Reading::Parsed(facts) = Read_Source(source)
        else
        {
            // The sample is a literal in the test just below. If the parser refuses it there
            // is no payload to hand the second crate, and the cross-crate agreement under
            // test would be asserted over zero items — which any decoder satisfies.
            panic!("the sample parses")
        };

        return Encode_Payload(&facts);
    }

    /// The agreement this crate exists to check: what `nomos-lang-rust` writes is what
    /// something else can read, without sharing a type.
    #[test]
    fn Test_A_Syntax_Payload_Should_Be_Readable_By_A_Second_Crate()
    {
        let payload = Encoded(
            "pub fn one() {}\n\
             fn two() {}\n\
             pub struct Three;\n\
             pub(crate) fn four() {}\n",
        );

        assert_eq!(Public_Items(&payload), Ok((4, 2)));
    }

    /// `pub(crate)` is not `pub`. A prefix match on the visibility field would count it,
    /// and a directory's "public surface" would include everything it deliberately kept
    /// inside the crate.
    #[test]
    fn Test_Restricted_Visibility_Should_Not_Count_As_Public()
    {
        let payload = Encoded("pub(crate) fn inside() {}\npub(super) fn also() {}\n");

        assert_eq!(Public_Items(&payload), Ok((2, 0)));
    }

    /// A file that declares nothing has a payload, and it decodes to zero rather than to
    /// an error. Zero items is an answer; unreadable bytes are not.
    #[test]
    fn Test_An_Empty_Payload_Should_Decode_To_Zero()
    {
        assert_eq!(Public_Items(&Encoded("// nothing\n")), Ok((0, 0)));
    }

    /// The negative control for the decoder. If it fell back to a default, a rollup over
    /// bytes it could not read would report a directory that declares nothing — which is
    /// indistinguishable from a directory that genuinely declares nothing.
    #[test]
    fn Test_Bytes_That_Are_Not_A_Payload_Should_Be_Refused()
    {
        assert!(Public_Items(b"not a payload").is_err());
        assert!(Decode_Surface(b"files\tmany\n").is_err());
        assert!(
            Decode_Surface(b"files\t1\n").is_err(),
            "a surface has five fields, and four of them defaulting to zero is a lie"
        );
        assert!(
            Decode_Surface(b"files\t3\nunreachable\t0\nitems\t9\npublic\t2\n").is_err(),
            "the four-field encoding predates `approximate`, and reading it as a surface \
             with nothing approximated would be reading a decision it never made"
        );
    }

    #[test]
    fn Test_A_Surface_Should_Survive_A_Round_Trip()
    {
        let surface = Surface {
            files: 3,
            unreachable: 1,
            approximate: 2,
            items: 42,
            public: 7,
        };

        assert_eq!(Decode_Surface(&surface.Encode()), Ok(surface));
    }

    /// A rollup over an approximated member must not encode as one over a parsed member.
    ///
    /// The negative control for `OD-CAPABILITY-003`'s third condition. If `approximate` did
    /// not reach the bytes, buying coverage would also buy the appearance of precision, and
    /// nothing downstream could tell the two rollups apart.
    #[test]
    fn Test_An_Approximated_Rollup_Should_Not_Encode_Like_An_Exact_One()
    {
        let exact = Surface {
            files: 2,
            unreachable: 0,
            approximate: 0,
            items: 9,
            public: 4,
        };
        let approximated = Surface {
            approximate: 1,
            ..exact
        };

        assert_ne!(exact.Encode(), approximated.Encode());
    }
}
