//! [`SarifTool`], the `tool` object a run carries.

use super::sarif_result::SarifResult;
use super::tool_driver::ToolDriver;
use serde::Serialize;

/// SARIF 2.1.0 §3.18's `tool` object. One `driver` and no `extensions`: every rule this
/// workspace runs is composed into the one registry, so there is no second component to name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SarifTool
{
    /// The product and its rules.
    pub(crate) driver: ToolDriver,
}

impl SarifTool
{
    /// The tool for a run whose results are `results`.
    pub(crate) fn Of(results: &[SarifResult]) -> Self
    {
        return Self { driver: ToolDriver::Of(results) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Tool_Should_Serialize_Its_Driver_Under_The_Specifications_Name()
    {
        let rendered = serde_json::to_value(SarifTool::Of(&[])).expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/driver/name").and_then(serde_json::Value::as_str), Some("nomos"), "{rendered}");
        assert!(rendered.get("extensions").is_none(), "{rendered}");
    }
}
