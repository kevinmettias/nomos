//! [`ArtifactLocation`], the file a physical location names, relative to the run's root.

use serde::Serialize;
use std::path::Path;

/// SARIF 2.1.0 §3.4's `artifactLocation` object, carrying a `uri` relative to the run's root
/// with forward slashes.
///
/// No `uriBaseId` is emitted. The specification lets a relative `uri` be resolved against the
/// consumer's own notion of the root when no base is named, which is what every CI consumer
/// this projection exists for does with a checkout; naming a base would require an absolute
/// `originalUriBaseIds` entry, and the root a gate run carries is whatever path the caller
/// wrote, relative as often as not.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct ArtifactLocation
{
    /// Root-relative, forward slashes.
    pub(crate) uri: String,
}

impl ArtifactLocation
{
    /// `path` as a URI relative to `root`.
    ///
    /// This workspace's own walk already reports every source root-relative with forward
    /// slashes (`nomos_workspace_discovery`'s `Relative_Path` says so), so for a finding a rule
    /// produced over a walked source this is the identity. The two normalizations are for the
    /// findings that were not: a rule that echoes a path from a tool of its own, or a caller
    /// that hands an absolute location to `explain`. A backslash becomes a forward slash, a
    /// leading `root/` or `./` is removed, and a path under no such prefix is carried through
    /// whole rather than guessed at.
    pub(crate) fn Relative_To(root: &Path, path: &str) -> Self
    {
        let forward = path.replace('\\', "/");
        let root_forward = root.display().to_string().replace('\\', "/");
        let trimmed_root = root_forward.trim_end_matches('/');

        let under_root = if trimmed_root.is_empty() || trimmed_root == "."
        {
            forward.as_str()
        }
        else
        {
            forward
                .strip_prefix(trimmed_root)
                .and_then(|rest| return rest.strip_prefix('/'))
                .unwrap_or(&forward)
        };

        return Self { uri: under_root.strip_prefix("./").unwrap_or(under_root).to_owned() };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn Test_Relative_To_Should_Carry_A_Root_Relative_Forward_Slash_Path_Through_Unchanged()
    {
        let location = ArtifactLocation::Relative_To(Path::new("."), "src/lib.rs");

        assert_eq!(location.uri, "src/lib.rs");
    }

    /// The falsifier for the prefix strip: without it a location a rule reported under the
    /// root's own absolute spelling would reach a consumer as a path no checkout contains.
    #[test]
    fn Test_Relative_To_Should_Strip_The_Root_When_A_Location_Was_Reported_Under_It()
    {
        let root = PathBuf::from("F:\\repos\\probe");

        let location = ArtifactLocation::Relative_To(&root, "F:\\repos\\probe\\src\\lib.rs");

        assert_eq!(location.uri, "src/lib.rs");
    }

    /// `probe` is a prefix of `probe-two` as text and not as a path, and a strip that read only
    /// the text would turn one repository's file into a path that means nothing.
    #[test]
    fn Test_Relative_To_Should_Not_Strip_A_Root_That_Is_Only_A_Textual_Prefix()
    {
        let location = ArtifactLocation::Relative_To(Path::new("probe"), "probe-two/src/lib.rs");

        assert_eq!(location.uri, "probe-two/src/lib.rs");
    }

    #[test]
    fn Test_Relative_To_Should_Forward_Slash_A_Backslash_Path()
    {
        let location = ArtifactLocation::Relative_To(Path::new("."), "src\\response\\gate_findings.rs");

        assert_eq!(location.uri, "src/response/gate_findings.rs");
    }

    #[test]
    fn Test_Relative_To_Should_Drop_A_Leading_Dot_Segment()
    {
        let location = ArtifactLocation::Relative_To(Path::new("."), "./src/lib.rs");

        assert_eq!(location.uri, "src/lib.rs");
    }

    /// A root with a trailing separator names the same tree as one without.
    #[test]
    fn Test_Relative_To_Should_Treat_A_Trailing_Separator_On_The_Root_As_Absent()
    {
        let location = ArtifactLocation::Relative_To(Path::new("probe/"), "probe/src/lib.rs");

        assert_eq!(location.uri, "src/lib.rs");
    }

    /// An empty root is the `GateCommand` default, and it must not turn an absolute path into a
    /// relative one by stripping its leading separator.
    #[test]
    fn Test_Relative_To_Should_Leave_A_Path_Whole_Under_An_Empty_Root()
    {
        let location = ArtifactLocation::Relative_To(Path::new(""), "/srv/probe/src/lib.rs");

        assert_eq!(location.uri, "/srv/probe/src/lib.rs");
    }
}
