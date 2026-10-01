//! One refused thing, and where it was.
//!
//! A refusal is printed and never stored (`OD-POLICY-002` decision 5), so it carries the text
//! it matched and the policy's reason only for the person reading their terminal.

/// Where a refused thing was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place
{
    /// An added line of a file.
    Line
    {
        /// The commit that adds it, or `None` for the commit being made.
        commit: Option<String>,
        /// Repository-relative path.
        path: String,
        /// One-based line number in the new file.
        line: usize,
    },
    /// A file's path.
    Path
    {
        /// The commit that adds it, or `None` for the commit being made.
        commit: Option<String>,
        /// Repository-relative path.
        path: String,
    },
    /// A commit message.
    Message
    {
        /// The commit, or `None` for the commit being made.
        commit: Option<String>,
    },
    /// A commit's author or committer.
    Identity
    {
        /// The commit, or `None` for the commit being made.
        commit: Option<String>,
        /// `author` or `committer`.
        role: &'static str,
    },
}

impl Place
{
    /// The path an exception would name for this place, or `None` for an identity, which no
    /// exception can cover.
    #[must_use]
    pub fn Exception_Path(&self) -> Option<&str>
    {
        return match self
        {
            Self::Line { path, .. } | Self::Path { path, .. } => Some(path),
            Self::Message { .. } => Some(crate::party_policy::MESSAGE_PLACE),
            Self::Identity { .. } => None,
        };
    }
}

impl std::fmt::Display for Place
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::Line { commit, path, line } => write!(formatter, "{path}:{line} ({})", Commit_Label(commit.as_deref())),
            Self::Path { commit, path } => write!(formatter, "file name {path} ({})", Commit_Label(commit.as_deref())),
            Self::Message { commit } => write!(formatter, "message of {}", Commit_Label(commit.as_deref())),
            Self::Identity { commit, role } => write!(formatter, "{role} of {}", Commit_Label(commit.as_deref())),
        };
    }
}

/// How long a commit id is shown.
const SHOWN_COMMIT_LENGTH: usize = 10;

fn Commit_Label(commit: Option<&str>) -> String
{
    return match commit
    {
        Some(id) => format!("commit {}", id.get(..SHOWN_COMMIT_LENGTH).unwrap_or(id)),
        None => "the commit being made".to_owned(),
    };
}

/// A thing the policy refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    /// The policy rule it matched, or `identity` for a party address.
    pub rule: String,
    /// Where it was.
    pub place: Place,
    /// The offending text, trimmed to [`SHOWN_TEXT_LENGTH`] characters.
    pub text: String,
    /// The policy's reason.
    pub why: String,
}

/// How much of an offending line a refusal repeats.
pub const SHOWN_TEXT_LENGTH: usize = 160;

/// The rule id an identity refusal carries.
pub const IDENTITY_RULE: &str = "identity";

pub(crate) fn Shown(text: &str) -> String
{
    let trimmed = text.trim();
    if trimmed.chars().count() <= SHOWN_TEXT_LENGTH
    {
        return trimmed.to_owned();
    }
    let kept: String = trimmed.chars().take(SHOWN_TEXT_LENGTH).collect();
    return format!("{kept}...");
}
