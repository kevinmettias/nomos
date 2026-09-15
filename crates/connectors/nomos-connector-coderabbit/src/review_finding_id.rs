//! The external review finding's own identity, honestly minted from what the vendor
//! supplied.

use serde::{Deserialize, Serialize};

/// Identity of one review finding, as this connector canonically constructs it from the
/// vendor's own stable key.
///
/// A `Named_Identity`-shaped value, per `ARC-CONNECTOR-001`'s first invariant: an honest
/// mint, never a digest computed over bytes this connector does not independently hold.
/// Hand-written here rather than reused from `nomos-model`'s own `Named_Identity` macro
/// (which is private to that crate): this identity is this crate's own vendor-specific
/// concept, not a shared kernel type.
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

    /// Canonically constructs this connector's own identity for one review comment, from
    /// the two vendor-supplied values that together are GitHub's own stable,
    /// human-addressable key for it -- the repository the comment was posted against and
    /// GitHub's own permanent numeric id for the comment row itself.
    ///
    /// Nomos owns this construction (the join and the `review-comment` label) and does not
    /// claim to have derived `repository` or `comment_id` from anything; both arrive
    /// verbatim from the vendor response [`crate::translation::Translate_Review_Comment`]
    /// reads. The label is `review-comment`, not `issue` or `comment`, because GitHub's own
    /// schema calls this object a "pull request review comment," distinct from an ordinary
    /// issue comment, and this connector only ever reads that one endpoint.
    #[must_use]
    pub fn Of_Review_Comment(repository: &str, comment_id: u64) -> Self
    {
        return Self(format!("{repository}#review-comment:{comment_id}"));
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

    /// The repository the comment this crate's own recorded fixture was captured from was
    /// posted against.
    const REPOSITORY: &str = "coderabbitai/rabbits-playground";

    /// GitHub's own permanent id for that comment.
    const COMMENT_ID: u64 = 3_521_038_097;

    /// A different comment, from the same repository -- one whose id differs from
    /// [`COMMENT_ID`] in its last digit only, which is the case a reader comparing the two
    /// could otherwise read as the same finding.
    const OTHER_COMMENT_ID: u64 = 3_521_038_104;

    #[test]
    fn Test_A_Review_Comment_Identity_Should_Join_Repository_And_Comment_Id()
    {
        let id = ReviewFindingId::Of_Review_Comment(REPOSITORY, COMMENT_ID);
        assert_eq!(id.As_Str(), "coderabbitai/rabbits-playground#review-comment:3521038097");
    }

    #[test]
    fn Test_Two_Different_Comments_Should_Be_Different_Identities()
    {
        assert_ne!(
            ReviewFindingId::Of_Review_Comment(REPOSITORY, COMMENT_ID),
            ReviewFindingId::Of_Review_Comment(REPOSITORY, OTHER_COMMENT_ID)
        );
        assert_ne!(
            ReviewFindingId::Of_Review_Comment(REPOSITORY, COMMENT_ID),
            ReviewFindingId::Of_Review_Comment("coderabbitai/other-repo", COMMENT_ID)
        );
    }
}
