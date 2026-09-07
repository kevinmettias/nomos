//! Every file under a root this crate is asked to judge.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying registered
//! language recognition plus `check-script-discipline`'s own script extensions -- the walk
//! itself, once this module's own copy of an identical walk three other hosts also carried,
//! now lives in `nomos-workspace-discovery`. `OD-HOST-008`.

use nomos_rules::SourceFile;
use std::path::Path;

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
pub(crate) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    return nomos_workspace_discovery::Walked_Sources(root, &Recognized_Extensions());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn Test_A_Go_File_Should_Be_Discovered_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-api-sources-go-discovery");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");
        std::fs::write(root.join("main.go"), "package main\n\nfunc One() {}\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_An_Unrelated_Extension_Should_Not_Be_Discovered()
    {
        let root = Fresh_Root("nomos-api-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
    }

    #[test]
    fn Test_A_Shebang_Script_And_A_Forbidden_Script_Extension_Should_Be_Discovered()
    {
        let root = Fresh_Root("nomos-api-sources-script-discovery");
        std::fs::write(root.join("deploy.sh"), "#!/bin/bash\n# deploys\nset -euo pipefail\n").expect("writable");
        std::fs::write(root.join("tool.ps1"), "Write-Host 'hi'\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["deploy.sh", "tool.ps1"], "{paths:?}");
    }

    #[test]
    fn Test_Walked_Sources_Should_Discover_Real_Files_And_Return_None_For_A_Non_Directory()
    {
        let root = Fresh_Root("nomos-api-sources-walked-sources");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let discovered = Walked_Sources(&root).expect("root is a real directory");
        let not_a_directory = Walked_Sources(&root.join("a.rs"));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(discovered.len(), 1, "{discovered:?}");
        assert!(not_a_directory.is_none(), "{not_a_directory:?}");
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
