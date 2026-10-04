//! [`SarifRun`], the one run a projected log carries.

use super::run_properties::RunProperties;
use super::sarif_invocation::SarifInvocation;
use super::sarif_result::SarifResult;
use super::sarif_tool::SarifTool;
use serde::Serialize;

/// SARIF 2.1.0 §3.14's `run` object: the tool, what it found, and how its execution went.
///
/// `artifacts`, `originalUriBaseIds` and `automationDetails` are not emitted. The first would
/// list every walked file for a consumer that reads only the ones results cite; the second is
/// declined where [`super::artifact_location::ArtifactLocation`] says why; and the third
/// wants an identity across runs that `GateRunResult::run` -- a fresh digest per execution,
/// by [`crate::Fresh_Run_Id`]'s own doc -- is deliberately not.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SarifRun
{
    /// The product and every rule it could have fired.
    pub(crate) tool: SarifTool,
    /// One per finding, in canonical order.
    pub(crate) results: Vec<SarifResult>,
    /// Exactly one, since a judgment is one execution.
    pub(crate) invocations: Vec<SarifInvocation>,
    /// The rules that judged an empty population and the values nobody declared, when there
    /// are any; [`RunProperties`] says why here and nowhere else. Absent rather than empty, so a
    /// run in which every selected rule judged something and every value was declared serializes
    /// exactly as it did before the bag existed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<RunProperties>,
}

impl SarifRun
{
    /// A run over `results`, whose execution `invocation` describes, carrying `properties`.
    pub(crate) fn Of(results: Vec<SarifResult>, invocation: SarifInvocation, properties: Option<RunProperties>) -> Self
    {
        return Self { tool: SarifTool::Of(&results), results, invocations: vec![invocation], properties };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_check_orchestration::CheckOutcome;

    #[test]
    fn Test_A_Run_Should_Serialize_Its_Tool_Results_And_One_Invocation()
    {
        let rendered = serde_json::to_value(SarifRun::Of(Vec::new(), SarifInvocation::Of_Check_Run(&CheckOutcome::NoSource), None))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert!(rendered.pointer("/tool/driver/name").is_some(), "{rendered}");
        assert_eq!(rendered.pointer("/results").and_then(serde_json::Value::as_array).map(Vec::len), Some(0), "{rendered}");
        assert_eq!(rendered.pointer("/invocations").and_then(serde_json::Value::as_array).map(Vec::len), Some(1), "{rendered}");
        assert!(rendered.get("properties").is_none(), "{rendered}");
    }
}
