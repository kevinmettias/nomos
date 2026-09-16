//! Reading GitHub's own JSON response for one pull-request review comment into this
//! connector's canonical `FindingPayload` -- the vendor-to-canonical side of
//! `ARC-CONNECTOR-001`'s seam.
//!
//! A pure function of the vendor's own bytes: no process is run here, no filesystem is
//! touched, and the same input always produces the same output -- which is what lets
//! `OD-CONNECTOR-002`'s recorded fixture stand in for a live fetch on every replay, and what
//! `crate::determinism::ReviewFindingProduction` declares.
//!
//! # What this reader trusts, and what it refuses rather than guesses at
//!
//! Three things are read from the vendor's response, and each has a different kind of
//! backing:
//!
//! - `path`, `line` (falling back to `original_line` for a comment whose diff position has
//!   since moved), `html_url` and `id` are GitHub's own formally documented REST fields for
//!   this endpoint (`GET /repos/{owner}/{repo}/pulls/comments/{comment_id}`).
//! - `user.login` must read `coderabbitai[bot]` exactly. This connector's entire reason for
//!   existing is `CodeRabbit`'s own review; translating an arbitrary human reviewer's comment
//!   through it would misreport whose judgment the resulting fact represents.
//! - `category` and `severity` are read from the comment body's own leading line --
//!   `_<category>_ | _<severity>_ | _<effort>_` -- a real, observed convention this reader
//!   was written against real `CodeRabbit` comments captured for this crate's fixture rather
//!   than against any formal GitHub or `CodeRabbit` field schema. GitHub's API does not
//!   version this convention; `CodeRabbit`'s own comment body does, with an embedded
//!   `<!-- cr-comment:v1:... -->` marker this reader requires before trusting the marker
//!   line at all. A body carrying a different or absent version marker is refused rather
//!   than parsed against a convention this reader was not written against.
//!
//! `effort` (the third marker segment -- "Quick win", "Heavy lift", ...) is deliberately not
//! read. This connector's ceiling is not pinned to a complete mapping of `CodeRabbit`'s own
//! comment vocabulary -- a curated subset, not the full schema.

use crate::ReviewFindingId;
use crate::payload::finding_payload::FindingPayload;

/// The GitHub account every comment this connector translates must be posted by.
const CODERABBIT_BOT_LOGIN: &str = "coderabbitai[bot]";

/// The version marker `CodeRabbit`'s own comment body carries, naming the comment-body
/// convention this reader is written against.
const RECOGNIZED_COMMENT_VERSION_MARKER: &str = "<!-- cr-comment:v1:";

/// GitHub's own response could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationError
{
    pub reason: String,
}

impl core::fmt::Display for TranslationError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Translates one `gh api repos/{repository}/pulls/comments/{id}` response into this
/// connector's canonical fields.
///
/// `repository` is not read out of the vendor bytes -- this endpoint's own response carries
/// no bare `owner/repo` field, only URLs it is embedded in, and
/// [`crate::fetching::Fetch_Review_Comment`] already knows which repository it asked about.
/// Nomos supplies it here rather than inferring it, the same "own the construction, do not
/// claim to have derived the vendor's value" discipline `ARC-CONNECTOR-001`'s first
/// invariant states, mirrored by [`crate::identity::ReviewFindingId::Of_Review_Comment`].
///
/// # Errors
///
/// [`TranslationError`] if the bytes are not valid UTF-8 JSON, are missing `id`, `path`,
/// both `line` and `original_line`, `html_url` or `user.login`, if the comment was not
/// posted by `CodeRabbit`'s own bot account, or if the body does not carry a recognized
/// `cr-comment:v1` marker and its leading category/severity line in the shape this reader
/// is written against.
pub fn Translate_Review_Comment(repository: &str, vendor_bytes: &[u8]) -> Result<FindingPayload, TranslationError>
{
    let value = Vendor_Value(vendor_bytes)?;
    let id = Review_Comment_Id(&value)?;
    Require_CodeRabbit_Posted_It(&value, id)?;

    let path = Field_Str(&value, "path")?;
    let html_url = Field_Str(&value, "html_url")?;
    let line = Line_Field(&value)?;
    let body = Field_Str(&value, "body")?;
    let (category, severity) = Marker_Line(&body)?;

    return Ok(FindingPayload {
        external_system: "coderabbit".to_owned(),
        external_id: ReviewFindingId::Of_Review_Comment(repository, id),
        locator: html_url,
        category,
        severity,
        path,
        line,
        message: body,
    });
}

/// GitHub's own response for one review comment, decoded as JSON and otherwise unread.
fn Vendor_Value(vendor_bytes: &[u8]) -> Result<serde_json::Value, TranslationError>
{
    return serde_json::from_slice(vendor_bytes).map_err(|error| TranslationError {
        reason: format!("not valid JSON: {error}"),
    });
}

/// GitHub's own permanent numeric id for the comment row itself.
fn Review_Comment_Id(value: &serde_json::Value) -> Result<u64, TranslationError>
{
    return value
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| TranslationError {
            reason: format!("no numeric \"id\" field: {value}"),
        });
}

/// Refuses a comment posted by anybody but `CodeRabbit`'s own bot account. This connector's
/// entire reason for existing is `CodeRabbit`'s own review; translating an arbitrary human
/// reviewer's comment through it would misreport whose judgment the resulting fact
/// represents.
fn Require_CodeRabbit_Posted_It(value: &serde_json::Value, id: u64) -> Result<(), TranslationError>
{
    let login = value
        .get("user")
        .and_then(|user| user.get("login"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| TranslationError {
            reason: format!("no string \"user.login\" field: {value}"),
        })?;

    if login == CODERABBIT_BOT_LOGIN
    {
        return Ok(());
    }

    return Err(TranslationError {
        reason: format!(
            "comment {id} was posted by {login:?}, not {CODERABBIT_BOT_LOGIN:?}; this connector \
             only translates CodeRabbit's own comments"
        ),
    });
}

fn Field_Str(value: &serde_json::Value, field: &str) -> Result<String, TranslationError>
{
    return value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| TranslationError {
            reason: format!("no string \"{field}\" field: {value}"),
        });
}

/// `line` is `null` for a comment whose diff position has since moved out from under it;
/// `original_line` survives that. Preferring `line` when present reports where the comment
/// currently anchors; falling back to `original_line` is still an honest report of where
/// GitHub's own record locates it, never a guess.
fn Line_Field(value: &serde_json::Value) -> Result<String, TranslationError>
{
    if let Some(line) = value.get("line").and_then(serde_json::Value::as_u64)
    {
        return Ok(line.to_string());
    }
    if let Some(original) = value.get("original_line").and_then(serde_json::Value::as_u64)
    {
        return Ok(original.to_string());
    }

    return Err(TranslationError {
        reason: format!("neither \"line\" nor \"original_line\" is a number: {value}"),
    });
}

/// Reads the category and severity off a `CodeRabbit` comment body's own leading marker
/// line -- `_<category>_ | _<severity>_ | _<effort>_` -- after requiring the version marker
/// this reader is written against.
fn Marker_Line(body: &str) -> Result<(String, String), TranslationError>
{
    Require_Recognized_Marker(body)?;

    let first_line = First_Line(body)?;
    let (category, severity) = Marker_Segments(first_line)?;

    return Ok((Trim_Marker_Segment(category), Trim_Marker_Segment(severity)));
}

/// Refuses a body whose comment-body convention this reader was not written against: the
/// version marker is what says which one it is, and a body carrying a different or absent
/// one is refused rather than parsed against a guess.
fn Require_Recognized_Marker(body: &str) -> Result<(), TranslationError>
{
    if body.contains(RECOGNIZED_COMMENT_VERSION_MARKER)
    {
        return Ok(());
    }

    return Err(TranslationError {
        reason: format!(
            "body does not carry a recognized {RECOGNIZED_COMMENT_VERSION_MARKER:?} marker; \
             this reader is written against that comment-body version and refuses to guess \
             at an unversioned or different one"
        ),
    });
}

/// The body's own first line -- the one the marker convention writes category, severity and
/// effort on.
fn First_Line(body: &str) -> Result<&str, TranslationError>
{
    return body.lines().next().ok_or_else(|| TranslationError {
        reason: "body is empty; no marker line to read".to_owned(),
    });
}

/// A marker line's own ` | `-separated segments, at least the category and the severity.
fn Marker_Segments(first_line: &str) -> Result<(&str, &str), TranslationError>
{
    let segments: Vec<&str> = first_line.split(" | ").collect();
    let [category, severity, ..] = segments.as_slice()
    else
    {
        return Err(TranslationError {
            reason: format!(
                "leading line does not carry at least category and severity segments separated \
                 by \" | \": {first_line:?}"
            ),
        });
    };

    return Ok((category, severity));
}

fn Trim_Marker_Segment(segment: &str) -> String
{
    return segment.trim().trim_matches('_').trim().to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The exact bytes `crate::fixture::Sample_Review_Comment_Response` embeds -- this
    /// repository's own recorded fixture, captured from a real `gh api` call against a
    /// real, public `CodeRabbit` review comment. `OD-CONNECTOR-002`'s fixture rule: recorded
    /// below the translation, in vendor wire format, read here exactly as a live response
    /// would be.
    #[test]
    fn Test_The_Recorded_Fixture_Should_Translate_To_The_Expected_Canonical_Fields()
    {
        let payload = Translate_Review_Comment(
            "coderabbitai/rabbits-playground",
            crate::fixture::Sample_Review_Comment_Response(),
        )
        .expect("this crate's own recorded fixture");

        assert_eq!(payload.external_system, "coderabbit");
        assert_eq!(
            payload.external_id.As_Str(),
            "coderabbitai/rabbits-playground#review-comment:3521038097"
        );
        assert_eq!(
            payload.locator,
            "https://github.com/coderabbitai/rabbits-playground/pull/13#discussion_r3521038097"
        );
        assert_eq!(payload.category, "🔒 Security & Privacy");
        assert_eq!(payload.severity, "🟡 Minor");
        assert_eq!(payload.path, "modules/security/main.tf");
        assert_eq!(payload.line, "43");
        assert!(payload.message.starts_with("_🔒 Security & Privacy_"));
        assert!(payload.message.contains("Consider defining an explicit KMS key policy"));
    }

    #[test]
    fn Test_Non_Json_Bytes_Should_Be_Refused()
    {
        assert!(Translate_Review_Comment("owner/repo", b"not json").is_err());
    }

    #[test]
    fn Test_A_Response_Missing_A_Field_Should_Be_Refused()
    {
        let incomplete = br#"{"id":1,"user":{"login":"coderabbitai[bot]"}}"#;
        assert!(Translate_Review_Comment("owner/repo", incomplete).is_err());
    }

    #[test]
    fn Test_A_Comment_From_A_Human_Should_Be_Refused()
    {
        let human = r#"{
            "id": 1,
            "user": {"login": "octocat"},
            "path": "a.rs",
            "line": 1,
            "html_url": "https://github.com/o/r/pull/1#discussion_r1",
            "body": "_🔒 Security & Privacy_ | _🟡 Minor_ | _⚡ Quick win_\n<!-- cr-comment:v1:abc -->\nlooks fine to me"
        }"#
        .as_bytes();
        let refusal = Translate_Review_Comment("owner/repo", human);
        assert!(refusal.is_err());
        assert!(refusal.unwrap_err().reason.contains("octocat"));
    }

    #[test]
    fn Test_A_Body_With_No_Version_Marker_Should_Be_Refused()
    {
        let unmarked = r#"{
            "id": 1,
            "user": {"login": "coderabbitai[bot]"},
            "path": "a.rs",
            "line": 1,
            "html_url": "https://github.com/o/r/pull/1#discussion_r1",
            "body": "_🔒 Security & Privacy_ | _🟡 Minor_ | _⚡ Quick win_\nlooks fine to me"
        }"#
        .as_bytes();
        let refusal = Translate_Review_Comment("owner/repo", unmarked);
        assert!(refusal.is_err());
    }

    #[test]
    fn Test_A_Null_Line_Should_Fall_Back_To_Original_Line()
    {
        let outdated = r#"{
            "id": 1,
            "user": {"login": "coderabbitai[bot]"},
            "path": "a.rs",
            "line": null,
            "original_line": 7,
            "html_url": "https://github.com/o/r/pull/1#discussion_r1",
            "body": "_🔒 Security & Privacy_ | _🟡 Minor_ | _⚡ Quick win_\n<!-- cr-comment:v1:abc -->\nstill here"
        }"#
        .as_bytes();
        let payload = Translate_Review_Comment("owner/repo", outdated).expect("original_line stands in for a null line");
        assert_eq!(payload.line, "7");
    }
}
