//! [`SarifNotification`], why a run's execution was not successful.

use super::sarif_level::SarifLevel;
use super::sarif_message::SarifMessage;
use serde::Serialize;

const UNREADABLE_ROOT: &str = "the root could not be read as a tree, so nothing was judged";
const NO_SOURCE: &str = "the walk found no source under the root, so nothing was judged";
const INCOMPLETE_CLAIM: &str =
    "some rule that was asked could not look at its subject, so this judgment is an incomplete claim rather than a clean one";
const INCOMPLETE_COVERAGE: &str =
    "the declared coverage floor is require-completeness and the selected findings are an incomplete claim, so no verdict was reached";

/// SARIF 2.1.0 §3.58's `notification` object, as one of an invocation's
/// `toolExecutionNotifications`.
///
/// Every constructor here names one way a run stopped short of a clean judgment, in the
/// words the canonical response gives for it, so that a consumer reading only this object
/// still learns what a person reading `nomos gate run` at a terminal would. An `error` is a
/// fault -- a tree or a policy that could not be read, a registry that contradicts itself --
/// and a `warning` is a condition nothing is wrong about but a clean log would misrepresent:
/// no source to judge, no fact to reason over, a rule that could not look.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SarifNotification
{
    /// Fault or condition; see the type's doc.
    pub(crate) level: SarifLevel,
    /// Why, for a person.
    pub(crate) message: SarifMessage,
}

impl SarifNotification
{
    /// `CheckOutcome::Unreadable`: the root is not a directory that could be walked.
    pub(crate) fn Unreadable_Root() -> Self
    {
        return Self::Error(UNREADABLE_ROOT);
    }

    /// `CheckOutcome::Contradictory`: the rule composition beneath the run refuses itself.
    pub(crate) fn Contradictory(cause: &str) -> Self
    {
        return Self::Error(format!("the rule composition beneath this run is self-contradictory, so nothing was judged: {cause}"));
    }

    /// `CheckOutcome::NoSource`: the walk found nothing it recognizes.
    pub(crate) fn No_Source() -> Self
    {
        return Self::Warning(NO_SOURCE);
    }

    /// `CheckOutcome::NoFacts`: `files` were read and none materialized a fact.
    pub(crate) fn No_Facts(files: usize) -> Self
    {
        return Self::Warning(format!("{files} file(s) were read and no fact was materialized for any of them, so nothing was judged"));
    }

    /// A judged run whose claim is incomplete: some rule could not look. `OD-GATE-016`'s
    /// "unknown is not pass", as a consumer of the log must read it.
    pub(crate) fn Incomplete_Claim() -> Self
    {
        return Self::Warning(INCOMPLETE_CLAIM);
    }

    /// `NoVerdict::UnreadablePolicy`: a `nomos-gate.json` is present and could not be read.
    pub(crate) fn Unreadable_Policy(detail: &str) -> Self
    {
        return Self::Error(format!("the gate policy could not be read, so no verdict was reached: {detail}"));
    }

    /// `NoVerdict::MalformedPolicy`: a `nomos-gate.json` is present and is not a policy the
    /// reader accepts. `detail` is the reader's own message, which names the refused key.
    pub(crate) fn Malformed_Policy(detail: &str) -> Self
    {
        return Self::Error(format!("the gate policy is not one the reader accepts, so no verdict was reached: {detail}"));
    }

    /// `NoVerdict::IncompleteCoverage`: the repository asked for `require-completeness` and
    /// did not get it. Nothing is wrong with the tree or the policy.
    pub(crate) fn Incomplete_Coverage() -> Self
    {
        return Self::Warning(INCOMPLETE_COVERAGE);
    }

    fn Error(text: impl Into<String>) -> Self
    {
        return Self { level: SarifLevel::Error, message: SarifMessage::Text(text) };
    }

    fn Warning(text: impl Into<String>) -> Self
    {
        return Self { level: SarifLevel::Warning, message: SarifMessage::Text(text) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// How many files the `No_Facts` fixture below reports having read.
    const READ_FILES: usize = 3;

    /// A fault is an error and a condition is a warning, across every constructor.
    #[test]
    fn Test_Every_Fault_Should_Be_An_Error_And_Every_Condition_A_Warning()
    {
        for fault in [
            SarifNotification::Unreadable_Root(),
            SarifNotification::Contradictory("a cause"),
            SarifNotification::Unreadable_Policy("a detail"),
            SarifNotification::Malformed_Policy("a detail"),
        ]
        {
            assert_eq!(fault.level, SarifLevel::Error, "{}", fault.message.text);
        }

        for condition in [
            SarifNotification::No_Source(),
            SarifNotification::No_Facts(READ_FILES),
            SarifNotification::Incomplete_Claim(),
            SarifNotification::Incomplete_Coverage(),
        ]
        {
            assert_eq!(condition.level, SarifLevel::Warning, "{}", condition.message.text);
        }
    }

    /// The detail is what a person acts on, so it has to survive into the message.
    #[test]
    fn Test_A_Notification_Built_From_A_Detail_Should_Carry_That_Detail()
    {
        assert!(SarifNotification::Contradictory("registry says so").message.text.contains("registry says so"));
        assert!(SarifNotification::Unreadable_Policy("permission denied").message.text.contains("permission denied"));
        assert!(SarifNotification::Malformed_Policy("unknown field `basline`").message.text.contains("basline"));
        assert!(SarifNotification::No_Facts(READ_FILES).message.text.contains("3 file(s)"));
    }

    #[test]
    fn Test_A_Notification_Should_Serialize_Its_Level_And_Message_Under_The_Specifications_Names()
    {
        let rendered = serde_json::to_value(SarifNotification::No_Source()).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/level").and_then(serde_json::Value::as_str), Some("warning"), "{rendered}");
        assert_eq!(rendered.pointer("/message/text").and_then(serde_json::Value::as_str), Some(NO_SOURCE), "{rendered}");
    }
}
