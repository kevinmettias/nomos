//! What a root contains before anything judges it -- the first-run answer a repository
//! adopting nomos has no verb for.
//!
//! `P123-WORKSPACE-PROFILE-FOR-ADOPTION` measured the gap: the binary's verb groups are
//! `work`, `spec`, `check`, `request`, `gate`, `agent`, `correct` and `workflow`, and none of
//! them diagnoses a root. Nothing says which registered languages a tree contains, which
//! policy files it declares, or which it lacks; this crate was walk-only. A profile is a pure
//! function of that walk plus existence checks at the root, so it lives beside the walk,
//! ahead of the verb that will render it -- `nomos-cli` is another item's territory, and no
//! verb is added here.
//!
//! # What a profile deliberately does not do
//!
//! It does not open a policy file beyond reading its bytes as text. Whether
//! `nomos-gate.json` parses is `nomos_gate_orchestration`'s own answer, whether
//! `nomos-architecture.json` does is `nomos_repo_policy`'s, and a profile that parsed either
//! would depend on every reader's shape to say what a first run knows before any reader is
//! run. [`crate::PolicyFilePresence`] draws that line and says which side each answer is on.
//!
//! It does not walk for manifests. A `Cargo.toml` or `go.mod` below the root is a member's,
//! and which members a workspace has is the toolchain's own reading
//! (`nomos-lang-rust-cargo`, `nomos-lang-go-modules`); a profile reports what sits at the
//! root a run would be pointed at.
//!
//! It does not count what the walk does not read. A source count is the population a check
//! run judges -- [`crate::Walked_Sources`] over [`crate::Registered_Extensions`] -- so a file
//! under `target`, inside a nested repository root, or unreadable as text is absent from the
//! count for the same reason it is absent from a run.

use crate::{Carries_Root_Marker, LanguageManifest, PolicyFile, ProfileRefusal, Registered_Extensions, SourceCount, Walked_Sources};
use nomos_rules::SourceFile;
use std::path::Path;

/// What a root contains, as a first run sees it.
///
/// Deterministic for a given tree: every list is in a fixed order this crate spells once
/// ([`Registered_Extensions`] for the counts, `LanguageManifest::At_Root` and
/// `PolicyFile::At_Root` for the other two), the walk sorts what it finds, and nothing here
/// reads a clock, an environment variable or anything outside `root`. `tests` proves it by
/// computing the same tree twice and by asserting one tree's whole profile as a literal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceProfile
{
    /// Per registered extension, in [`Registered_Extensions`]'s own order, how many sources
    /// the walk found.
    pub sources: Vec<SourceCount>,
    /// Each manifest a registered language's toolchain reads at a root, and whether this
    /// root has it.
    pub manifests: Vec<LanguageManifest>,
    /// Each repository policy file a reader in this workspace opens, and what a first run
    /// finds of it.
    pub policy_files: Vec<PolicyFile>,
    /// Whether the root carries the marker the walk treats a nested directory's copy of as
    /// somebody else's root. The same file as the `standards.json` entry in `policy_files`,
    /// reported twice because it answers two questions: as a policy file, whether the five
    /// `nomos.cap.*.policy` providers have anything to read; as a marker, whether this
    /// directory is a repository root at all, which is what decides whether a run pointed
    /// one level up would judge it as its own or skip it as vendored.
    pub has_root_marker: bool,
}

impl WorkspaceProfile
{
    /// The profile of `root`.
    ///
    /// # Errors
    ///
    /// [`ProfileRefusal::RootIsNotADirectory`] when `root` is not a directory. An empty
    /// directory is not an error; it profiles as a tree with nothing in it.
    pub fn Of_Root(root: &Path) -> Result<Self, ProfileRefusal>
    {
        let Some(sources) = Walked_Sources(root, &Registered_Extensions())
        else
        {
            return Err(ProfileRefusal::RootIsNotADirectory { root: root.to_path_buf() });
        };

        return Ok(Self {
            sources: Counted_By_Extension(&sources),
            manifests: LanguageManifest::At_Root(root),
            policy_files: PolicyFile::At_Root(root),
            has_root_marker: Carries_Root_Marker(root),
        });
    }
}

/// How many of `sources` carry each registered extension, in [`Registered_Extensions`]'s
/// own order -- every extension reported, a zero included, so the shape of a profile does
/// not depend on which languages a tree happens to contain.
fn Counted_By_Extension(sources: &[SourceFile]) -> Vec<SourceCount>
{
    return Registered_Extensions()
        .into_iter()
        .map(|extension| {
            return SourceCount {
                extension,
                sources: sources.iter().filter(|source| return Has_Extension(&source.path, extension)).count(),
            };
        })
        .collect();
}

/// Whether a walked source's reported path carries `extension` -- the same last-component
/// reading the walk used to admit it, so the two cannot disagree about what `main.rs.bak`
/// is.
fn Has_Extension(path: &str, extension: &str) -> bool
{
    return Path::new(path).extension().and_then(std::ffi::OsStr::to_str) == Some(extension);
}

#[cfg(test)]
mod tests;
