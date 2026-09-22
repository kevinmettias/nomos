//! The external review finding's own identity, as the agreed payload carries it.

use serde::{Deserialize, Serialize};

/// Identity of one review finding, as it travels in a `nomos.review.finding.v1` payload.
///
/// Opaque on purpose. A `Named_Identity`-shaped value, per `ARC-CONNECTOR-001`'s first
/// invariant: an honest mint, never a digest computed over bytes a provider does not
/// independently hold -- and the minting itself belongs to whichever provider holds those
/// bytes, not here. `nomos-connector-coderabbit`'s own `Review_Comment_Identity` joins a
/// repository and GitHub's permanent comment id into one of these, and a second provider
/// reading a different review system would mint a different shape without asking this
/// crate's permission. What the agreement fixes is that the payload's `external_id` field
/// is one identity, written and read back verbatim; it does not fix how any party arrived
/// at it, which is the same division `OD-CAPABILITY-002` draws between a contract and a
/// provider's own claims.
///
/// Hand-written rather than reused from `nomos-model`'s own `Named_Identity` macro (which
/// is private to that crate): this identity is this capability's own concept.
///
/// Never coerced into a `SubjectId`. This identity travels as canonical payload data, and
/// a connector's own fact keys under the whole-workspace placeholder subject
/// (`nomos_model::Subject_Of_Path("")`) until a bears-on relation names a real Nomos
/// subject -- a future connector's own work, per `ARC-CONNECTOR-001`'s "What This Record
/// Does Not Do".
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReviewFindingId(String);

impl ReviewFindingId
{
    /// Wraps an already-constructed identifier.
    #[must_use]
    pub fn New(value: impl Into<String>) -> Self
    {
        return Self(value.into());
    }

    /// The identifier as constructed.
    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

impl core::fmt::Display for ReviewFindingId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.0);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The identity carried by this capability's one real provider's own recorded fixture,
    /// spelled here as a literal rather than minted: minting is the provider's act, and
    /// this crate is the agreement it mints into.
    const RECORDED: &str = "coderabbitai/rabbits-playground#review-comment:3521038097";

    /// A different comment from the same repository -- one whose id differs from
    /// [`RECORDED`]'s in its last digits only, which is the case a reader comparing the two
    /// could otherwise read as the same finding.
    const OTHER: &str = "coderabbitai/rabbits-playground#review-comment:3521038104";

    #[test]
    fn Test_An_Identity_Should_Be_Carried_Verbatim()
    {
        let id = ReviewFindingId::New(RECORDED);

        assert_eq!(id.As_Str(), RECORDED);
        assert_eq!(id.to_string(), RECORDED);
    }

    #[test]
    fn Test_Two_Different_Findings_Should_Be_Different_Identities()
    {
        assert_ne!(ReviewFindingId::New(RECORDED), ReviewFindingId::New(OTHER));
    }
}
