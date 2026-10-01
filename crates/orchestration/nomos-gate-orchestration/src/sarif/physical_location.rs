//! [`PhysicalLocation`], where in a file a result points.

use super::artifact_location::ArtifactLocation;
use super::sarif_region::SarifRegion;
use serde::Serialize;
use std::path::Path;

/// SARIF 2.1.0 §3.29's `physicalLocation` object: an artifact, and a region within it when the
/// finding reported one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PhysicalLocation
{
    /// The file, relative to the run's root.
    pub(crate) artifact_location: ArtifactLocation,
    /// The line, when the location string carried one. Absent rather than defaulted: a rule
    /// that named a package or a bare file did not mean line one, and a consumer given a region
    /// treats it as measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) region: Option<SarifRegion>,
}

impl PhysicalLocation
{
    /// One `Finding::locations` entry, split into its file and its optional line.
    ///
    /// Splits on the last `:` only when what follows is entirely digits and something precedes
    /// it -- the one convention that recurs across this workspace's rules, and the same reading
    /// `crates/host/nomos-lsp/src/location.rs` makes for an editor. A Windows drive letter, a
    /// trailing colon with nothing numeric after it, and a package name are each carried
    /// through whole.
    pub(crate) fn Of(root: &Path, raw: &str) -> Self
    {
        let (path, line) = Split_Line(raw);

        return Self { artifact_location: ArtifactLocation::Relative_To(root, path), region: line.map(SarifRegion::Starting_At) };
    }
}

/// `raw` as a path and the line it ends with, if it ends with one.
fn Split_Line(raw: &str) -> (&str, Option<u32>)
{
    if let Some((path, line)) = raw.rsplit_once(':')
        && let Ok(line) = line.parse::<u32>()
        && !path.is_empty()
    {
        return (path, Some(line));
    }

    return (raw, None);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line number the fixture location strings below carry, so an assertion reads as a
    /// citation of the fixture rather than a bare number.
    const REPORTED_LINE: u32 = 42;

    #[test]
    fn Test_Of_Should_Split_A_Path_And_Line_Into_An_Artifact_And_A_Region()
    {
        let location = PhysicalLocation::Of(Path::new("."), "crates/rules/src/lib.rs:42");

        assert_eq!(location.artifact_location.uri, "crates/rules/src/lib.rs");
        assert_eq!(location.region, Some(SarifRegion::Starting_At(REPORTED_LINE)));
    }

    #[test]
    fn Test_Of_Should_Carry_A_Bare_Path_With_No_Region()
    {
        let location = PhysicalLocation::Of(Path::new("."), "crates/rules/src/lib.rs");

        assert_eq!(location.artifact_location.uri, "crates/rules/src/lib.rs");
        assert_eq!(location.region, None);
    }

    /// The falsifier for the digits guard: `nomos-rules` has no colon and `lib.rs:` has nothing
    /// numeric after its colon, and a split that did not check would report a line of nothing.
    #[test]
    fn Test_Of_Should_Not_Mistake_A_Package_Name_Or_A_Trailing_Colon_For_A_Line()
    {
        for raw in ["nomos-rules", "crates/rules/src/lib.rs:"]
        {
            let location = PhysicalLocation::Of(Path::new("."), raw);

            assert_eq!(location.artifact_location.uri, raw, "{raw}");
            assert_eq!(location.region, None, "{raw}");
        }
    }

    /// A drive letter is a colon followed by a path, never by digits alone; it must survive
    /// as a path even though it is not one this workspace's own walk would ever produce.
    #[test]
    fn Test_Of_Should_Not_Mistake_A_Drive_Letter_For_A_Line()
    {
        let location = PhysicalLocation::Of(Path::new("C:\\probe"), "C:\\probe\\src\\lib.rs");

        assert_eq!(location.artifact_location.uri, "src/lib.rs");
        assert_eq!(location.region, None);
    }

    #[test]
    fn Test_A_Location_Without_A_Region_Should_Serialize_No_Region_Property_At_All()
    {
        let rendered = serde_json::to_value(PhysicalLocation::Of(Path::new("."), "a.rs"))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert!(rendered.get("region").is_none(), "{rendered}");
        assert_eq!(rendered.pointer("/artifactLocation/uri").and_then(serde_json::Value::as_str), Some("a.rs"), "{rendered}");
    }
}
