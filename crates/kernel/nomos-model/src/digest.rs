//! Content addressing.
//!
//! One hash function, one framing rule, one place. Everything in Nomos whose identity
//! is its content resolves through here, so there is nothing for a second
//! implementation to disagree with.

use nomos_contracts::Digest128;

/// BLAKE3 is wider than a [`Digest128`], checked where the two constants are, and not at
/// the truncation.
///
/// A `const` block is evaluated during compilation, so this is the width relationship
/// established once rather than re-established on every hash. The previous spelling asked
/// it at run time and answered with `.expect`, which is a panic in the analysis path — and
/// a panic there is a determinism defect and not merely a crash: a replay must reach the
/// same panic at the same step, and one that depends on a digest width would not.
const _: () = assert!(blake3::OUT_LEN >= Digest128::BYTE_LENGTH);

/// Takes the leading [`Digest128::BYTE_LENGTH`] bytes of a full-width digest.
///
/// Written as a zip rather than a slice or a chunk so that no length is asserted here at
/// all. `zip` stops at the shorter of the two, the shorter is the output by its own type,
/// and the assertion above is what makes "the shorter" a fact rather than a hope.
fn Truncate(full: &[u8; blake3::OUT_LEN]) -> [u8; Digest128::BYTE_LENGTH]
{
    let mut truncated = [0_u8; Digest128::BYTE_LENGTH];

    for (slot, byte) in truncated.iter_mut().zip(full)
    {
        *slot = *byte;
    }

    return truncated;
}

/// The digest of a byte sequence.
///
/// BLAKE3 with a fixed configuration, truncated to 128 bits. The fixed configuration is
/// the part that matters: a hash whose seed varies per process — which is what Rust's
/// default hasher does, deliberately — would make every identity in the system
/// unreproducible across runs, and the failure would show up as a cache that never hits
/// rather than as an error.
#[must_use]
pub fn Content_Digest(bytes: &[u8]) -> Digest128
{
    let full = blake3::hash(bytes);
    return Digest128::From_Bytes(Truncate(full.as_bytes()));
}

/// The digest of an ordered sequence of parts.
///
/// Each part is length-prefixed before hashing. Without that framing,
/// `["ab", "c"]` and `["a", "bc"]` hash identically, and two genuinely different
/// identities collide — which for a fact cache means silently serving one subject's
/// analysis as another's. Concatenation is not a serialization.
#[must_use]
pub fn Digest_Of_Parts(parts: &[&[u8]]) -> Digest128
{
    let mut hasher = blake3::Hasher::new();

    for part in parts
    {
        let length = u64::try_from(part.len()).unwrap_or(u64::MAX);
        hasher.update(&length.to_le_bytes());
        hasher.update(part);
    }

    let mut truncated = [0_u8; Digest128::BYTE_LENGTH];
    truncated.copy_from_slice(&hasher.finalize().as_bytes()[..Digest128::BYTE_LENGTH]);

    return Digest128::From_Bytes(truncated);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Digest_Should_Be_Stable_Across_Calls()
    {
        assert_eq!(Content_Digest(b"nomos"), Content_Digest(b"nomos"));
    }

    #[test]
    fn Test_Different_Content_Should_Digest_Differently()
    {
        assert_ne!(Content_Digest(b"nomos"), Content_Digest(b"nomo"));
    }

    /// The framing property, and the reason `Digest_Of_Parts` exists at all. Without
    /// length prefixes these two would collide, and a collision here is a fact cache
    /// serving one subject's analysis under another subject's key.
    #[test]
    fn Test_Part_Boundaries_Should_Change_The_Digest()
    {
        let split_early = Digest_Of_Parts(&[b"ab", b"c"]);
        let split_late = Digest_Of_Parts(&[b"a", b"bc"]);

        assert_ne!(
            split_early, split_late,
            "part boundaries must be part of the digest, or distinct identities collide"
        );
    }

    /// An empty part is a part. Dropping it would let a two-component identity collide
    /// with a one-component identity that happens to have the same non-empty content.
    #[test]
    fn Test_Empty_Parts_Should_Be_Significant()
    {
        assert_ne!(Digest_Of_Parts(&[b"a", b""]), Digest_Of_Parts(&[b"a"]));
        assert_ne!(Digest_Of_Parts(&[b"", b"a"]), Digest_Of_Parts(&[b"a"]));
    }

    #[test]
    fn Test_Part_Order_Should_Change_The_Digest()
    {
        assert_ne!(Digest_Of_Parts(&[b"a", b"b"]), Digest_Of_Parts(&[b"b", b"a"]));
    }

    /// A pinned vector. If this changes, every stored identity in every snapshot and
    /// cache in existence has silently changed meaning, and the failure would otherwise
    /// present as an inexplicable total cache miss rather than as a broken promise.
    #[test]
    fn Test_Digest_Should_Match_Its_Pinned_Vector()
    {
        assert_eq!(
            Content_Digest(b"nomos").to_string(),
            "78355ed706564e7e1304eac4d92c31f8"
        );
    }
}
