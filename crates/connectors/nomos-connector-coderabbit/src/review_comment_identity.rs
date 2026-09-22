//! This connector's own mint: one GitHub review comment, named as a
//! [`nomos_cap_review_finding::ReviewFindingId`].

use nomos_cap_review_finding::ReviewFindingId;

/// Canonically constructs this connector's own identity for one review comment, from the
/// two vendor-supplied values that together are GitHub's own stable, human-addressable key
/// for it -- the repository the comment was posted against and GitHub's own permanent
/// numeric id for the comment row itself.
///
/// Here rather than in `nomos-cap-review-finding` beside the type it returns, and that is
/// the point of the split `OD-ROADMAP-005`'s fifth decision required: the join and the
/// `review-comment` label are GitHub's addressing scheme, so a contract that spelled them
/// would be one vendor's convention wearing the agreement's name. What the agreement fixes
/// is that `external_id` is one identity carried verbatim; how a party arrives at one is
/// its own business, the same division `OD-CAPABILITY-002` draws for a `ProviderId` and a
/// `Guarantee`.
///
/// Nomos owns this construction and does not claim to have derived `repository` or
/// `comment_id` from anything; both arrive verbatim from the vendor response
/// [`crate::translation::Translate_Review_Comment`] reads. The label is `review-comment`,
/// not `issue` or `comment`, because GitHub's own schema calls this object a "pull request
/// review comment," distinct from an ordinary issue comment, and this connector only ever
/// reads that one endpoint.
#[must_use]
pub fn Review_Comment_Identity(repository: &str, comment_id: u64) -> ReviewFindingId
{
    return ReviewFindingId::New(format!("{repository}#review-comment:{comment_id}"));
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
        let id = Review_Comment_Identity(REPOSITORY, COMMENT_ID);
        assert_eq!(id.As_Str(), "coderabbitai/rabbits-playground#review-comment:3521038097");
    }

    #[test]
    fn Test_Two_Different_Comments_Should_Be_Different_Identities()
    {
        assert_ne!(
            Review_Comment_Identity(REPOSITORY, COMMENT_ID),
            Review_Comment_Identity(REPOSITORY, OTHER_COMMENT_ID)
        );
        assert_ne!(
            Review_Comment_Identity(REPOSITORY, COMMENT_ID),
            Review_Comment_Identity("coderabbitai/other-repo", COMMENT_ID)
        );
    }
}
