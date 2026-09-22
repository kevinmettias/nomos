//! [`SarifRegion`], the line a physical location points at.

use serde::Serialize;

/// SARIF 2.1.0 §3.30's `region` object, carrying the one coordinate this workspace's rules
/// report.
///
/// `startColumn`, `endLine` and `endColumn` are not emitted. A `Finding::locations` entry
/// carries at most a line (`crates/host/nomos-lsp/src/location.rs` catalogues the shapes), and
/// a region that invented a column or an end would point a consumer at a span the rule never
/// measured. The specification makes every other coordinate optional for exactly this case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SarifRegion
{
    /// One-based, as both the specification and this workspace's rules count lines.
    pub(crate) start_line: u32,
}

impl SarifRegion
{
    /// A region starting at, and saying nothing beyond, `start_line`.
    pub(crate) const fn Starting_At(start_line: u32) -> Self
    {
        return Self { start_line };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A line number that is not one, so the assertion cannot be satisfied by a default.
    const REPORTED_LINE: u32 = 17;

    #[test]
    fn Test_A_Region_Should_Serialize_Its_Line_Under_Start_Line_And_Nothing_Else()
    {
        let rendered = serde_json::to_value(SarifRegion::Starting_At(REPORTED_LINE))
            .expect("a derived Serialize over one integer has nothing to refuse");

        assert_eq!(rendered.pointer("/startLine").and_then(serde_json::Value::as_u64), Some(u64::from(REPORTED_LINE)), "{rendered}");
        assert_eq!(rendered.as_object().map(serde_json::Map::len), Some(1), "no coordinate the rule did not measure: {rendered}");
    }
}
