//! Every file under the root a real `gate run` is to judge.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying registered
//! language recognition plus `check-script-discipline`'s own script extensions -- the walk
//! itself, once this module's own copy of an identical walk three other hosts also carried,
//! now lives in `nomos-workspace-discovery`. `OD-HOST-008`.

use super::{Path, SourceFile};

/// Every rule/subject/script extension this walk recognizes: a registered language package's
/// own extensions, plus `check-script-discipline`'s five -- see `nomos_workspace_discovery::
/// SCRIPT_EXTENSIONS`'s own doc for why the second set is not itself package-registered.
fn Recognized_Extensions() -> Vec<&'static str>
{
    let mut recognized = nomos_workspace_discovery::Registered_Extensions();
    recognized.extend(nomos_workspace_discovery::SCRIPT_EXTENSIONS);
    return recognized;
}

/// The sources under `root` this walk recognizes, or `None` if `root` is not a directory.
pub(super) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    return nomos_workspace_discovery::Walked_Sources(root, &Recognized_Extensions());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Go_File_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-cli-gate-sources-go-discovery");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");
        std::fs::write(root.join("main.go"), "package main\n\nfunc One() {}\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Not_Discover_An_Unrelated_Extension()
    {
        let root = Fresh_Root("nomos-cli-gate-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Discover_A_Shebang_Script_And_A_Forbidden_Script_Extension()
    {
        let root = Fresh_Root("nomos-cli-gate-sources-script-discovery");
        std::fs::write(root.join("deploy.sh"), "#!/bin/bash\n# deploys\nset -euo pipefail\n").expect("writable");
        std::fs::write(root.join("tool.ps1"), "Write-Host 'hi'\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["deploy.sh", "tool.ps1"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-gate-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
