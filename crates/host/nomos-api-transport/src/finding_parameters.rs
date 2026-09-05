//! What an explain names: a tree, and the one finding to answer for.

use nomos_contracts::RuleId;
use nomos_gate_orchestration::FindingQuery;
use serde::Deserialize;
use std::path::PathBuf;

/// The arguments `nomos.gate.explain` takes.
///
/// `rule` and `location` carry no default, matching `nomos gate explain`, which requires both
/// and refuses without them: an explain with no finding named is not a smaller explain, it is
/// a different question nobody asked. `root` defaults the same way [`crate::GateParameters`]'
/// does.
///
/// There is no scope or rule selector here, and the omission is `nomos_gate_orchestration::
/// Explain_Gate`'s own decision rather than this transport's: that function is documented as
/// deliberately independent of `command.scope` and `command.rules`, because a query about a
/// rule a selection excludes must still be answerable.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingParameters
{
    /// The tree to judge before answering. Absent, the serving process's own working
    /// directory.
    #[serde(default)]
    pub root: Option<PathBuf>,
    /// The rule the finding to explain was produced by.
    pub rule: String,
    /// One location that finding names, as a reader of a run's own output already sees it.
    pub location: String,
}

impl FindingParameters
{
    /// The tree to judge.
    #[must_use]
    pub fn Root(&self) -> PathBuf
    {
        return self.root.clone().unwrap_or_else(|| return PathBuf::from("."));
    }

    /// These arguments as the query the orchestration seam takes.
    #[must_use]
    pub fn Query(&self) -> FindingQuery
    {
        return FindingQuery { rule: RuleId::New(self.rule.clone()), location: self.location.clone() };
    }
}

#[cfg(test)]
mod tests
{
    use super::FindingParameters;
    use std::path::PathBuf;

    /// Both required arguments reach the query, and an absent root reaches the working
    /// directory.
    #[test]
    fn Test_A_Named_Finding_Should_Reach_The_Query_And_Default_Its_Root()
    {
        let body = r#"{"rule":"naming-convention","location":"a.rs"}"#;
        let parameters: FindingParameters = serde_json::from_str(body).expect("both required fields are present");

        let query = parameters.Query();

        assert_eq!(parameters.Root(), PathBuf::from("."));
        assert_eq!(query.rule.As_Str(), "naming-convention");
        assert_eq!(query.location, "a.rs");
    }

    /// A request naming no finding is refused, rather than answered about whichever finding
    /// an empty rule and an empty location happen to match.
    #[test]
    fn Test_A_Request_Naming_No_Finding_Should_Be_Refused()
    {
        let refused = serde_json::from_str::<FindingParameters>(r#"{"root":"."}"#);

        assert!(refused.is_err(), "{refused:?}");
    }
}
