//! Every file under the root this server judges.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying only the
//! extensions a registered language package recognizes -- the walk itself, once this
//! crate's own copy of an identical walk three other hosts also carried, now lives in
//! `nomos-workspace-discovery`. `OD-HOST-008`.

use nomos_rules::SourceFile;
use std::path::Path;

/// The sources under `root` a registered language package recognizes, or `None` if `root`
/// is not a directory.
pub(crate) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    return nomos_workspace_discovery::Walked_Sources(root, &nomos_workspace_discovery::Registered_Extensions());
}

#[cfg(test)]
mod tests
{
    use super::Walked_Sources;

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-lsp-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    #[test]
    fn Test_Walked_Sources_Should_Return_Sources_For_A_Real_Directory()
    {
        let root = std::env::temp_dir().join("nomos-lsp-sources-walked-sources-present");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(sources.len(), 1);
    }
}
