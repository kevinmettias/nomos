//! [`SarifLocation`], one entry of a result's `locations`.

use super::physical_location::PhysicalLocation;
use serde::Serialize;
use std::path::Path;

/// SARIF 2.1.0 §3.28's `location` object, in its physical form only.
///
/// `logicalLocations` is not emitted, although `Finding::subject_name` is often exactly the
/// qualified name one would carry: the specification's logical location wants a `kind` from a
/// closed vocabulary (`function`, `type`, `module`, ...) and this workspace's finding carries
/// no such classification of its subject name. Inventing one would be a claim the rule did not
/// make. The name reaches the reader through the message instead.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SarifLocation
{
    /// Where in which file.
    pub(crate) physical_location: PhysicalLocation,
}

impl SarifLocation
{
    /// One `Finding::locations` entry, relative to `root`.
    pub(crate) fn Of(root: &Path, raw: &str) -> Self
    {
        return Self { physical_location: PhysicalLocation::Of(root, raw) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Location_Should_Serialize_Its_Physical_Location_Under_The_Specifications_Name()
    {
        let rendered = serde_json::to_value(SarifLocation::Of(Path::new("."), "src/lib.rs:3"))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/physicalLocation/artifactLocation/uri").and_then(serde_json::Value::as_str), Some("src/lib.rs"), "{rendered}");
        assert_eq!(rendered.pointer("/physicalLocation/region/startLine").and_then(serde_json::Value::as_u64), Some(3), "{rendered}");
    }
}
