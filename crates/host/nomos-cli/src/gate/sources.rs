//! Every file under the root a real `gate run` is to judge.
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
        let paths = Discovered_Paths("nomos-cli-gate-sources-go-discovery",
            &[("a.rs", "pub fn One() {}\n"), ("main.go", "package main\n\nfunc One() {}\n")]);

        assert_eq!(paths, ["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Not_Discover_An_Unrelated_Extension()
    {
        let paths = Discovered_Paths("nomos-cli-gate-sources-unrelated-extension",
            &[("README.md", "# not source\n")]);

        assert!(paths.is_empty(), "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Shebang_Script_And_A_Forbidden_Script_Extension()
    {
        let paths = Discovered_Paths("nomos-cli-gate-sources-script-discovery",
            &[("deploy.sh", "#!/bin/bash\n# deploys\nset -euo pipefail\n"), ("tool.ps1", "Write-Host 'hi'\n")]);

        assert_eq!(paths, ["deploy.sh", "tool.ps1"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-gate-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    /// Writes `files` into a fresh temporary root, walks it, removes it, and returns the
    /// relative paths the walk discovered -- the shape every discovery test above asserts
    /// over, so the root's lifecycle is written once rather than once per assertion.
    fn Discovered_Paths(name: &str, files: &[(&str, &str)]) -> Vec<String>
    {
        let root = Fresh_Root(name);
        for (file, body) in files
        {
            std::fs::write(root.join(file), body).expect("the fresh temporary root is writable");
        }

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        return sources.iter().map(|source| return source.path.clone()).collect();
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
