//! Every file under the root a resident judges.
//!
//! A thin wrapper over `nomos_workspace_discovery::Walked_Sources`, supplying the identical
//! extension set `nomos-cli::check::sources` supplies -- a registered language package's own
//! extensions plus `check-script-discipline`'s five. The set is not a preference here. A
//! resident that walked a narrower tree than a cold `nomos check` would return fewer findings
//! than one, and the difference would read as an effect of residency rather than as two
//! different populations. `OD-HOST-008` is why the walk itself is not here at all.

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

    /// The set this crate walks is the set `nomos-cli::check` walks, which is the property
    /// that keeps a resident's judgment and a cold invocation's judgment over one population.
    ///
    /// Asserted against the two declarations rather than against a written-down list, because
    /// a list is what goes stale when a fourth language package registers an extension.
    #[test]
    fn Test_Recognized_Extensions_Should_Be_Every_Registered_Extension_Plus_The_Script_Ones()
    {
        let recognized = Recognized_Extensions();

        for extension in nomos_workspace_discovery::Registered_Extensions()
        {
            assert!(recognized.contains(&extension), "{extension} is registered and not walked");
        }
        for extension in nomos_workspace_discovery::SCRIPT_EXTENSIONS
        {
            assert!(recognized.contains(&extension), "{extension} is a script extension and not walked");
        }
    }

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-daemon-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }
}
