//! This connector's one recorded vendor fixture, per `OD-CONNECTOR-002`.
//!
//! Captured from a real `gh api repos/coderabbitai/rabbits-playground/pulls/comments/
//! 3521038097` call, below the vendor-to-canonical translation -- GitHub's own response, in
//! GitHub's own field names, exactly as [`crate::fetching::Fetch_Review_Comment`] would
//! have received it live. The comment is a real `CodeRabbit` review finding
//! (`coderabbitai[bot]`) posted on a public pull request in `CodeRabbit`'s own
//! `coderabbitai/rabbits-playground` GitHub organization -- a demonstration repository
//! `CodeRabbit`'s own account maintains, chosen for exactly the public, stable,
//! no-credential reachability this connector's own live test needs.
//!
//! Embedded at compile time rather than read from disk at test time: a missing fixture is a
//! compile error here, which is louder than the runtime "must fail, not print `ok`"
//! `OD-CONNECTOR-002` requires of a fixture a test reads from disk.

/// This connector's one recorded vendor response, byte for byte as `gh api` printed it.
#[must_use]
pub fn Sample_Review_Comment_Response() -> &'static [u8]
{
    return include_bytes!("../tests/fixtures/coderabbit-review-comment-rabbits-playground-13-3521038097.json");
}
