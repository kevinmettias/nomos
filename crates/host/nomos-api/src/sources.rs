//! Every file under a root this crate is asked to judge.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying registered
//! language recognition plus `check-script-discipline`'s own script extensions -- the walk
//! itself, once this module's own copy of an identical walk three other hosts also carried,
//! now lives in `nomos-workspace-discovery`. `OD-HOST-008`.

use nomos_rules::SourceFile;
use std::path::Path;

/// The sources under `root` this walk recognizes, or `None` if `root` is not a directory.
pub(crate) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
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
    fn Test_A_Go_File_Should_Be_Discovered_Alongside_A_Rust_One()
    {
        let discovered = Discovered_In_A_Fresh_Tree(
            "nomos-api-sources-go-discovery",
            &[("a.rs", "pub fn One() {}\n"), ("main.go", "package main\n\nfunc One() {}\n")],
        );

        assert_eq!(discovered, vec!["a.rs", "main.go"], "{discovered:?}");
    }

    #[test]
    fn Test_An_Unrelated_Extension_Should_Not_Be_Discovered()
    {
        let discovered = Discovered_In_A_Fresh_Tree("nomos-api-sources-unrelated-extension", &[("README.md", "# not source\n")]);

        assert!(discovered.is_empty(), "{discovered:?}");
    }

    #[test]
    fn Test_A_Shebang_Script_And_A_Forbidden_Script_Extension_Should_Be_Discovered()
    {
        let discovered = Discovered_In_A_Fresh_Tree(
            "nomos-api-sources-script-discovery",
            &[("deploy.sh", "#!/bin/bash\n# deploys\nset -euo pipefail\n"), ("tool.ps1", "Write-Host 'hi'\n")],
        );

        assert_eq!(discovered, vec!["deploy.sh", "tool.ps1"], "{discovered:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Discover_Real_Files_And_Return_None_For_A_Non_Directory()
    {
        let root = Fresh_Root_Holding("nomos-api-sources-walked-sources", &[("a.rs", "pub fn One() {}\n")]);

        let discovered = Walked_Sources(&root).expect("root is a real directory");
        let not_a_directory = Walked_Sources(&root.join("a.rs"));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(discovered.len(), 1, "{discovered:?}");
        assert!(not_a_directory.is_none(), "{not_a_directory:?}");
    }

    /// Every path this crate's walk recognizes under a fresh tree built from `files`, which is
    /// removed again before this returns -- so a test that only asks what was discovered has
    /// no tree to clean up.
    fn Discovered_In_A_Fresh_Tree(name: &str, files: &[(&str, &str)]) -> Vec<String>
    {
        let root = Fresh_Root_Holding(name, files);
        let discovered = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        return discovered.iter().map(|source| return source.path.clone()).collect();
    }

    /// A fresh tree under `name` holding one file per `(name, content)` pair, so a test's own
    /// fixture reads as the files it holds rather than as a run of writes.
    fn Fresh_Root_Holding(name: &str, files: &[(&str, &str)]) -> PathBuf
    {
        let root = Fresh_Root(name);
        for (file_name, content) in files
        {
            std::fs::write(root.join(file_name), content).expect("the fresh root above was just created");
        }

        return root;
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
