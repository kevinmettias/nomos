//! The stable name a published workflow definition is known by.

/// The stable name a published [`super::WorkflowDefinition`] is known by.
///
/// A human-authored spelling rather than a content digest, which is deliberate and is the
/// half of identity that a digest cannot carry: a definition edited and republished is
/// still *the same workflow*, and a caller asking to replay "the nightly gate workflow"
/// is naming this, not a particular revision of it. Which revision is
/// [`super::WorkflowDefinition::Version`]'s question, and whether the content behind a
/// revision moved is [`crate::ReplayRefusal::Content`]'s.
///
/// `nomos_contracts::Digest128`'s own doc draws exactly this line -- every identity in
/// Nomos is either a content digest or a human-authored string with a stable spelling --
/// and this is the second kind. It is not admitted to `nomos-contracts`, because no peer
/// that never compiles this crate has to agree with us about it: a workflow definition is
/// assembled and replayed inside this workspace, and `OD-CONTRACTS-001`'s admission test
/// refuses a type that fails that question.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowDefinitionId(String);

impl WorkflowDefinitionId
{
    /// The identity spelled `name`.
    #[must_use]
    pub fn New(name: &str) -> Self
    {
        return Self(name.to_owned());
    }

    /// The spelling this identity was minted with.
    #[must_use]
    pub fn Text(&self) -> &str
    {
        return &self.0;
    }
}

impl core::fmt::Display for WorkflowDefinitionId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.0);
    }
}
