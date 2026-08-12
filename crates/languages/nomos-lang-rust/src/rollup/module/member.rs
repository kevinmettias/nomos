//! One member a module declares.

use crate::provider::Syntax_Inputs;
use nomos_analysis::InputDigest;
use nomos_contracts::SubjectId;
/// One file a module is made of.
///
/// Carries the subject *and* the semantic inputs its syntax fact was computed from,
/// because a fact is looked up by rebuilding its key and the inputs are a component of
/// one. [`ModuleMember::Of`] is the way to construct it: it routes through
/// [`Syntax_Inputs`], so the digest this rebuilds a key with is the digest that wrote it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleMember
{
    pub subject: SubjectId,
    pub inputs: InputDigest,
}

impl ModuleMember
{
    /// A member from the file's subject and its entire contents.
    #[must_use]
    pub fn Of(subject: SubjectId, source: &str) -> Self
    {
        return Self {
            subject,
            inputs: Syntax_Inputs(source),
        };
    }
}
