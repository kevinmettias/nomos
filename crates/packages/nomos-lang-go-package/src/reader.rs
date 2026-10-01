//! Reading a `LanguagePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! The identical split `nomos-lang-package::reader` states: field presence, JSON shape,
//! `package_kind`, `protocol_range` ordering and provider resolution are all
//! `nomos_package::Parse_Manifest`'s job, called here with this crate's own
//! [`crate::known_providers::KNOWN_PROVIDERS`]. What stays here is exactly the Go-specific
//! step nothing generic could do: resolving each `language_versions` label against
//! [`GoVersion`], with the same [`ManifestError::MalformedVersion`] shape --
//! field name, indexed position, cause wording -- `nomos-lang-package`'s own reader uses.

use crate::go_version::GoVersion;
use crate::known_providers::KNOWN_PROVIDERS;
use crate::language_package::LanguagePackage;

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
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Parse_Manifest(text: &str, at: &str) -> Result<LanguagePackage, ManifestError>
{
    let generic = nomos_package::Parse_Manifest(text, at, KNOWN_PROVIDERS)?;
    let language_versions = Resolved_Versions(&generic.language_versions, at)?;

    return Ok(LanguagePackage {
        package_id: generic.package_id,
        package_kind: generic.package_kind,
        package_version: generic.package_version,
        protocol_range: generic.protocol_range,
        language_versions,
        providers: generic.providers,
    });
}

/// Every raw `language_versions` label resolved against [`GoVersion`], or a
/// [`ManifestError::MalformedVersion`] naming which entry did not resolve.
fn Resolved_Versions(labels: &[String], at: &str) -> Result<Vec<GoVersion>, ManifestError>
{
    let mut versions = Vec::with_capacity(labels.len());

    for (index, label) in labels.iter().enumerate()
    {
        let version = GoVersion::Of_Label(label).ok_or_else(|| {
            return ManifestError::MalformedVersion {
                at: at.to_owned(),
                field: format!("language_versions[{index}]"),
                cause: format!("`{label}` is not a `<major>.<minor>` Go version this reader knows"),
            };
        })?;

        versions.push(version);
    }

    return Ok(versions);
}
