//! Reading a `LanguagePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! `OD-PACKAGE-007` split the language-agnostic half of this into `nomos_package`: field
//! presence, JSON shape, `package_kind`, `protocol_range` ordering and provider
//! resolution are all `nomos_package::Parse_Manifest`'s job now, called here with this
//! crate's own [`crate::known_providers::KNOWN_PROVIDERS`]. What stays here is exactly
//! the Rust-specific step nothing generic could do: resolving each `language_versions`
//! label against [`RustEdition`], with the same [`ManifestError::MalformedVersion`]
//! shape -- field name, indexed position, cause wording -- this reader produced before
//! the split, so the move is behavior-preserving rather than a new refusal.

use crate::known_providers::KNOWN_PROVIDERS;
use crate::language_version::RustEdition;
use crate::manifest::LanguagePackage;

pub use nomos_package::ManifestError;

use std::path::Path;

/// Reads and resolves a manifest from a file.
///
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Read_Manifest(path: &Path) -> Result<LanguagePackage, ManifestError>
{
    let at = path.display().to_string();
    let text = std::fs::read_to_string(path).map_err(|error| {
        return ManifestError::Unreadable { at: at.clone(), cause: error.to_string() };
    })?;

    return Parse_Manifest(&text, &at);
}

/// Parses a manifest from its text, naming `at` in every refusal.
///
/// `at` is a caller-supplied label rather than derived from a path, so a manifest read
/// from an embedded string or a test fixture can still be refused with a meaningful
/// name.
///
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Parse_Manifest(text: &str, at: &str) -> Result<LanguagePackage, ManifestError>
{
    let generic = nomos_package::Parse_Manifest(text, at, KNOWN_PROVIDERS)?;
    let language_versions = Resolved_Editions(&generic.language_versions, at)?;

    return Ok(LanguagePackage {
        package_id: generic.package_id,
        package_kind: generic.package_kind,
        package_version: generic.package_version,
        protocol_range: generic.protocol_range,
        language_versions,
        providers: generic.providers,
    });
}

/// Every raw `language_versions` label resolved against [`RustEdition`], or the same
/// [`ManifestError::MalformedVersion`] the reader produced before `OD-PACKAGE-007` split
/// this resolution out of the generic core.
fn Resolved_Editions(labels: &[String], at: &str) -> Result<Vec<RustEdition>, ManifestError>
{
    let mut editions = Vec::with_capacity(labels.len());

    for (index, label) in labels.iter().enumerate()
    {
        let edition = RustEdition::Of_Label(label).ok_or_else(|| {
            return ManifestError::MalformedVersion {
                at: at.to_owned(),
                field: format!("language_versions[{index}]"),
                cause: format!("`{label}` is not a Rust edition this reader knows"),
            };
        })?;

        editions.push(edition);
    }

    return Ok(editions);
}
