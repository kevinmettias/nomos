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
            "files\t{}\nunreachable\t{}\nitems\t{}\npublic\t{}\n",
            self.files, self.unreachable, self.items, self.public
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
            "items" => surface.items = value,
            "public" => surface.public = value,
            other => return Err(format!("`{other}` is not a surface field")),
        }
        seen = seen.saturating_add(1);
    }

    if seen != 4
    {
        return Err(format!("a surface has four fields and this had {seen}"));
    }

    return Ok(surface);
}

/// Counts the publicly declared items in a `nomos-lang-rust` syntax payload.
///
/// # Why the slice decodes rather than sharing a type
///
/// This is the only place the encoding `nomos-lang-rust` writes is read back by something
/// that is not `nomos-lang-rust`. A shared struct would make the two agree by
/// construction and prove nothing; parsing the bytes is what makes the payload an
/// interface rather than an internal detail that happens to be public.
///
/// # Errors
///
/// Returns the offending line for anything that is not a syntax payload.
pub fn Public_Items(payload: &[u8]) -> Result<(u32, u32), String>
{
    let text = core::str::from_utf8(payload).map_err(|error| return error.to_string())?;
    let mut items = 0_u32;
    let mut public = 0_u32;

    for line in text.lines()
    {
        let mut fields = line.split('\t');

        match fields.next()
        {
            Some("unexpanded") =>
            {}
            Some("item") =>
            {
                items = items.saturating_add(1);

                // ordinal, kind, visibility, name — the visibility is the fourth field,
                // so two `next()` calls land on it.
                let visibility = fields.nth(2);
                if visibility == Some("Public")
                {
                    public = public.saturating_add(1);
                }
            }
            _ => return Err(format!("`{line}` is not a syntax payload record")),
        }
    }

    return Ok((items, public));
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
            "a surface has four fields, and three of them defaulting to zero is a lie"
        );
    }

    #[test]
    fn Test_A_Surface_Should_Survive_A_Round_Trip()
    {
        let surface = Surface {
            files: 3,
            unreachable: 1,
            items: 42,
            public: 7,
        };

        assert_eq!(Decode_Surface(&surface.Encode()), Ok(surface));
    }
}
