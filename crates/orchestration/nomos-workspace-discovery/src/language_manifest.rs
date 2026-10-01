//! A language toolchain's own manifest at a root, and whether the root has one.

use std::path::Path;

/// Cargo's own manifest, and so the mark of a Rust workspace or package root.
///
/// A literal here because no crate in this workspace names it: `nomos-lang-rust-cargo` runs
/// `cargo metadata` and lets Cargo find its own manifest, and `nomos-lang-rust-package`
/// spells `Cargo.toml` only in prose about the `edition` field. There is no constant to
/// read, so there is nothing this repeats.
const CARGO_MANIFEST: &str = "Cargo.toml";

/// The Go module manifest, and so the mark of a Go module root.
///
/// `nomos-lang-go-modules` reads this file, and spells the name inline in its own
/// `module_error.rs` rather than as an exported constant, so there is no constant to read
/// from that crate either. This crate does not depend on it, and the literal would not be
/// reachable if it did.
const GO_MODULE_MANIFEST: &str = "go.mod";

/// The Go workspace manifest, which names the module roots a multi-module Go tree is made
/// of -- the same crate and the same inline spelling as [`GO_MODULE_MANIFEST`].
const GO_WORKSPACE_MANIFEST: &str = "go.work";

/// One manifest a registered language's toolchain reads at a root, and whether this root
/// has it.
///
/// Presence is an existence check on the root's own directory and nothing more: the
/// profile does not open a manifest, because what a manifest declares is the toolchain's
/// own reading (`nomos-lang-rust-cargo`, `nomos-lang-go-modules`), and a first run needs to
/// know only which toolchain has anything to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageManifest
{
    /// The manifest's own file name at the root.
    pub name: &'static str,
    /// The registered extension of the language whose toolchain reads it, spelled the way
    /// [`crate::Registered_Extensions`] spells it, so a profile can pair a manifest with a
    /// [`crate::SourceCount`] without a second vocabulary for naming a language.
    pub extension: &'static str,
    /// Whether the root has a file of that name.
    pub is_present: bool,
}

/// One manifest and the registered language whose toolchain reads it, before any root is
/// asked about it.
struct KnownManifest
{
    name: &'static str,
    extension: &'static str,
}

impl LanguageManifest
{
    /// Every manifest a registered language's toolchain reads at a root, in one fixed
    /// order, each checked against `root`.
    #[must_use]
    pub fn At_Root(root: &Path) -> Vec<Self>
    {
        return Known_Manifests()
            .into_iter()
            .map(|known| {
                return Self { name: known.name, extension: known.extension, is_present: root.join(known.name).is_file() };
            })
            .collect();
    }
}

/// The manifests a profile asks about, in the order a profile reports them: Rust's, then
/// Go's module manifest, then Go's workspace manifest. A function returning the list rather
/// than a module-level array, the way [`crate::Registered_Extensions`] is, because this is
/// the one place the pairing of a manifest with its language is spelled.
fn Known_Manifests() -> Vec<KnownManifest>
{
    return vec![
        KnownManifest { name: CARGO_MANIFEST, extension: nomos_lang_rust::RUST_EXTENSION },
        KnownManifest { name: GO_MODULE_MANIFEST, extension: nomos_lang_go::GO_EXTENSION },
        KnownManifest { name: GO_WORKSPACE_MANIFEST, extension: nomos_lang_go::GO_EXTENSION },
    ];
}
