//! [`NoVerdictResponse`], why a judged run reached no verdict, as
//! [`super::gate_run_response::GateRunResponse`] carries it.

use nomos_gate_orchestration::NoVerdict;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::NoVerdict`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other
/// one in this module.
///
/// # Why the three are kept apart
///
/// Because the reader has to do something different about each, and telling them apart is
/// the entire reason the domain type exists. Both policy causes send a reader to the
/// repository's own `nomos-gate.json`, and to different parts of the problem: a file that
/// cannot be read is a path or a permission, and one that cannot be parsed is its content,
/// which is the same distinction `Applicability` keeps between `ProviderUnavailable` and
/// `MissingCapability` rather than folding into one "could not look".
///
/// [`Self::IncompleteCoverage`] is not a fault at all. `OD-GATE-016` decided that a run
/// where some rules could not look must not be reported as a pass, and a repository that
/// declared `require-completeness` asked for exactly this. A caller that cannot tell it from
/// a broken policy file will go looking for a fault that is not there.
#[derive(Debug, Serialize)]
#[serde(tag = "cause", rename_all = "snake_case")]
pub enum NoVerdictResponse
{
    /// A `nomos-gate.json` is present under the run's root and could not be read at all, so
    /// there were no declared rules to reduce this run's findings by.
    UnreadablePolicy
    {
        /// The path and the failure, as the file system reported them.
        detail: String,
    },
    /// A `nomos-gate.json` is present and is not a policy the reader accepts.
    MalformedPolicy
    {
        /// The reader's own message, which names the refused key for a mis-spelling and the
        /// position for a syntax error. This is the field the whole chain exists to deliver:
        /// it was computed and discarded before `NoVerdict` existed, so a repository with one
        /// typo in its policy file was told only that something was wrong with it.
        detail: String,
    },
    /// The declared coverage floor is `require-completeness` and this run's selected findings
    /// are an incomplete claim, so a run that would otherwise have passed is not reported as
    /// one. Nothing is wrong with the tree or with the policy.
    IncompleteCoverage,
}

impl NoVerdictResponse
{
    pub(crate) fn From(cause: &NoVerdict) -> Self
    {
        return match cause
        {
            NoVerdict::UnreadablePolicy(detail) => Self::UnreadablePolicy { detail: detail.clone() },
            NoVerdict::MalformedPolicy(detail) => Self::MalformedPolicy { detail: detail.clone() },
            NoVerdict::IncompleteCoverage => Self::IncompleteCoverage,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent, so these tests read
    /// through `get` and report a missing key as `Null` rather than as a panic inside an
    /// assertion.
    fn At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    #[test]
    fn Test_Every_Cause_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        assert_eq!(At(&Rendered(&NoVerdict::UnreadablePolicy(String::new())), &["cause"]), "unreadable_policy");
        assert_eq!(At(&Rendered(&NoVerdict::MalformedPolicy(String::new())), &["cause"]), "malformed_policy");
        assert_eq!(At(&Rendered(&NoVerdict::IncompleteCoverage), &["cause"]), "incomplete_coverage");
    }

    /// The detail is what a person acts on, so it has to survive the crossing intact.
    #[test]
    fn Test_A_Malformed_Policy_Should_Carry_The_Readers_Own_Message()
    {
        let cause = NoVerdict::MalformedPolicy("unknown field `basline`, expected one of `suppressions`".to_owned());

        assert_eq!(At(&Rendered(&cause), &["detail"]), "unknown field `basline`, expected one of `suppressions`");
    }

    /// The coverage floor carries no detail, because there is nothing to go and look at.
    #[test]
    fn Test_The_Coverage_Floor_Should_Carry_No_Detail_To_Go_And_Read()
    {
        let rendered = Rendered(&NoVerdict::IncompleteCoverage);

        assert!(rendered.get("detail").is_none(), "{rendered}");
    }

    /// `cause` as the wire publishes it, so a test can read one field of it by path.
    fn Rendered(cause: &NoVerdict) -> serde_json::Value
    {
        return serde_json::to_value(NoVerdictResponse::From(cause)).expect("a derived Serialize over owned data has nothing to refuse");
    }
}
