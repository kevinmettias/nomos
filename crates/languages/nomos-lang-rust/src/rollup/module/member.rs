//! One member a module declares.

use nomos_analysis::InputDigest;
use nomos_contracts::SubjectId;
/// One file a module is made of.
///
/// Carries the subject *and* the semantic inputs its syntax fact was computed from,
/// because a fact is looked up by rebuilding its key and the inputs are a component of
/// one. [`Member::Of`] is the way to construct it: it routes through
/// [`Syntax_Inputs`], so the digest this rebuilds a key with is the digest that wrote it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member
{
    pub subject: SubjectId,
    pub inputs: InputDigest,
}

impl Member
{
    /// A member from the file's subject and its entire contents.
    #[must_use]
    pub fn Of(subject: SubjectId, source: &str) -> Self
    {
        use crate::provider::Syntax_Inputs;

        return Self {
            subject,
            inputs: Syntax_Inputs(source),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::provider::Syntax_Inputs;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Of_Should_Route_Its_Inputs_Through_Syntax_Inputs()
    {
        let subject = SubjectId::From_Digest(Content_Digest(b"a.rs"));
        let source = "pub fn A() {}\n";

        let member = Member::Of(subject, source);

        assert_eq!(member.subject, subject);
        assert_eq!(member.inputs, Syntax_Inputs(source));
    }
}
