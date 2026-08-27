// The content wrapper this vocabulary marks as untrusted, kept in its own file.
mod content;

pub use content::UntrustedPromptContent;

use serde::{Deserialize, Serialize};

const REPOSITORY_FILE_LABEL: &str = "RepositoryFile";
const COMMENT_LABEL: &str = "Comment";
const ISSUE_TEXT_LABEL: &str = "IssueText";
const TOOL_OUTPUT_LABEL: &str = "ToolOutput";
const RETRIEVED_KNOWLEDGE_LABEL: &str = "RetrievedKnowledge";

/// `AGT-EXEC-003`: "Repository files, comments, issue text, tool output, and retrieved
/// knowledge shall be treated as untrusted prompt content. Tool invocations and
/// privilege changes may not be authorized solely by embedded instructions."
///
/// The five named sources, in the corpus's own order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UntrustedPromptOrigin
{
    RepositoryFile,
    Comment,
    IssueText,
    ToolOutput,
    RetrievedKnowledge,
}

impl UntrustedPromptOrigin
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::RepositoryFile => REPOSITORY_FILE_LABEL,
            Self::Comment => COMMENT_LABEL,
            Self::IssueText => ISSUE_TEXT_LABEL,
            Self::ToolOutput => TOOL_OUTPUT_LABEL,
            Self::RetrievedKnowledge => RETRIEVED_KNOWLEDGE_LABEL,
        };
    }

    /// `AGT-EXEC-003`: "Tool invocations and privilege changes may not be authorized
    /// solely by embedded instructions." Content from any of these five origins can
    /// never, by itself, satisfy an authorization check -- always `false`, for every
    /// variant, on purpose.
    #[must_use]
    pub const fn May_Solely_Authorize(self) -> bool
    {
        return false;
    }
}

impl core::fmt::Display for UntrustedPromptOrigin
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [UntrustedPromptOrigin; 5] = [
        UntrustedPromptOrigin::RepositoryFile,
        UntrustedPromptOrigin::Comment,
        UntrustedPromptOrigin::IssueText,
        UntrustedPromptOrigin::ToolOutput,
        UntrustedPromptOrigin::RetrievedKnowledge,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|origin| return origin.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two origins share a wire spelling");
    }

    #[test]
    fn Test_No_Origin_May_Solely_Authorize()
    {
        for origin in ALL
        {
            assert!(!origin.May_Solely_Authorize(), "{origin} should never solely authorize a tool invocation or privilege change");
        }
    }
}
