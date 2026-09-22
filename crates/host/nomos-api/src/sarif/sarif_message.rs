//! [`SarifMessage`], the one-property `message` object a result and a notification each carry.

use nomos_contracts::Finding;
use serde::Serialize;

/// SARIF 2.1.0 §3.11's `message` object, in its plain-text form only.
///
/// `markdown` and `id`/`arguments` are not emitted: nothing here has a message template to
/// resolve an argument against, and a summary a rule wrote for a terminal is not markdown.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct SarifMessage
{
    /// The message, as a person reads it.
    pub(crate) text: String,
}

impl SarifMessage
{
    /// `finding`'s summary, prefixed with what the finding calls its subject.
    ///
    /// The subject name is in the text on purpose. `Finding::subject_name` is frequently finer
    /// than any location the finding carries -- a qualified name within a module, a package --
    /// and a SARIF `location` has nowhere to put it, so a message that omitted it would send a
    /// reader to a file with no word about which declaration inside it the rule meant.
    pub(crate) fn Of(finding: &Finding) -> Self
    {
        return Self { text: format!("{}: {}", finding.subject_name, finding.summary) };
    }

    /// A message carrying `text` verbatim -- the form a notification uses, since a
    /// notification is about the run rather than about a subject.
    pub(crate) fn Text(text: impl Into<String>) -> Self
    {
        return Self { text: text.into() };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Finding_At;

    #[test]
    fn Test_Of_Should_Name_The_Subject_And_Then_The_Summary()
    {
        let message = SarifMessage::Of(&Finding_At("a-rule", "a.rs:1"));

        assert_eq!(message.text, "a.rs: a real summary");
    }

    #[test]
    fn Test_Text_Should_Carry_Its_Argument_Verbatim()
    {
        assert_eq!(SarifMessage::Text("as written").text, "as written");
    }

    #[test]
    fn Test_A_Message_Should_Serialize_Under_The_Specifications_Text_Property()
    {
        let rendered = serde_json::to_value(SarifMessage::Text("hello"))
            .expect("a derived Serialize over one owned string has nothing to refuse");

        assert_eq!(rendered.pointer("/text").and_then(serde_json::Value::as_str), Some("hello"), "{rendered}");
    }
}
