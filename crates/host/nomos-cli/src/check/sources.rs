//! Every file under the root the run is to judge.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying registered
//! language recognition plus `check-script-discipline`'s own script extensions -- the walk
//! itself, once this module's own copy of an identical walk three other hosts also carried,
//! now lives in `nomos-workspace-discovery`. `OD-HOST-008`.

use super::{Path, SourceFile};

/// The sources under `root` this walk recognizes, or `None` if `root` is not a directory.
pub(super) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    return nomos_workspace_discovery::Walked_Sources(root, &Recognized_Extensions());
}

/// Every rule/subject/script extension this walk recognizes: a registered language package's
/// own extensions, plus `check-script-discipline`'s five -- see `nomos_workspace_discovery::
/// SCRIPT_EXTENSIONS`'s own doc for why the second set is not itself package-registered.
fn Recognized_Extensions() -> Vec<&'static str>
{
    let mut recognized = nomos_workspace_discovery::Registered_Extensions();
    recognized.extend(nomos_workspace_discovery::SCRIPT_EXTENSIONS);
    return recognized;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Go_File_Alongside_A_Rust_One()
    {
        let paths = Discovered_Paths(
            "nomos-cli-check-sources-go-discovery",
            &[("a.rs", "pub fn One() {}\n"), ("main.go", "package main\n\nfunc One() {}\n")],
        );

        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Not_Discover_An_Unrelated_Extension()
    {
        let paths = Discovered_Paths(
            "nomos-cli-check-sources-unrelated-extension",
            &[("README.md", "# not source\n")],
        );

        assert!(paths.is_empty(), "{paths:?}");
    }

    /// The population `check-script-discipline`'s own rules judge -- a shebang script and a
    /// `standards.json`-forbidden extension -- must actually reach a walk, or those rules
    /// report clean regardless of what either file does.
    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Shebang_Script_And_A_Forbidden_Script_Extension()
    {
        let paths = Discovered_Paths(
            "nomos-cli-check-sources-script-discovery",
            &[
                ("deploy.sh", "#!/bin/bash\n# deploys\nset -euo pipefail\n"),
                ("tool.ps1", "Write-Host 'hi'\n"),
            ],
        );

        assert_eq!(paths, vec!["deploy.sh", "tool.ps1"], "{paths:?}");
    }

    /// A root that is not a directory at all reports `None` rather than an empty list, so a
    /// caller can tell "there was nothing to read" from "there was nowhere to read".
    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-check-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    /// The paths a walk of a fresh temporary root holding `files` reports, in the order it
    /// reports them. Every test above states only what it wrote and what came back; naming
    /// the root, creating it, walking it and removing it are the same four steps each time.
    fn Discovered_Paths(case: &str, files: &[(&str, &str)]) -> Vec<String>
    {
        let root = Fresh_Root(case);
        for &(name, contents) in files
        {
            std::fs::write(root.join(name), contents).expect("the root was created just above, so a new fixture file lands inside it");
        }

        let sources = Walked_Sources(&root).expect("the root was created above as a directory, so the walk reports Some");

        let _ignored = std::fs::remove_dir_all(&root);
        return sources.iter().map(|source| return source.path.clone()).collect();
    }

    /// Removes and recreates `root` under the system temp directory, so a test starts from a
    /// clean, empty tree regardless of what an earlier run left behind.
    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
