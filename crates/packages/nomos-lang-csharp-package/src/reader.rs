//! Reading a `LanguagePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! The identical split `nomos-lang-package::reader` and `nomos-lang-go-package::reader` state:
//! field presence, JSON shape, `package_kind`, `protocol_range` ordering and provider resolution
//! are all `nomos_package::Parse_Manifest`'s job, called here with this crate's own
//! [`crate::known_providers::KNOWN_PROVIDERS`]. What stays here is exactly the C#-specific step
//! nothing generic could do: resolving each `language_versions` label against
//! [`CsharpVersion`], with the same [`ManifestError::MalformedVersion`] shape — field name,
//! indexed position, cause wording — both sibling readers use.

use crate::csharp_version::CsharpVersion;
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

/// Every raw `language_versions` label resolved against [`CsharpVersion`], or a
/// [`ManifestError::MalformedVersion`] naming which entry did not resolve.
fn Resolved_Versions(labels: &[String], at: &str) -> Result<Vec<CsharpVersion>, ManifestError>
{
    let mut versions = Vec::with_capacity(labels.len());

    for (index, label) in labels.iter().enumerate()
    {
        let version = CsharpVersion::Of_Label(label).ok_or_else(|| {
            return ManifestError::MalformedVersion {
                at: at.to_owned(),
                field: format!("language_versions[{index}]"),
                cause: format!("`{label}` is not a `<major>` or `<major>.<minor>` C# version this reader knows"),
            };
        })?;

        versions.push(version);
    }

    return Ok(versions);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
    use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
    use std::path::PathBuf;

    /// The one place the reader is exercised against a file on disk rather than a string in a
    /// test, and the reason this test module is in the library rather than in `tests/`: the
    /// verification predicate for the item that added this crate runs `--lib`, so a round-trip
    /// living in an integration target would be a claim the item's own check never made.
    #[test]
    fn Test_The_Real_Csharp_Manifest_Should_Round_Trip_From_Disk()
    {
        let manifest = Read_Manifest(&Real_Manifest_Path()).expect("the real manifest parses");

        assert_eq!(manifest, Expected());
    }

    /// This crate's own resolution step, which is the only logic it adds to the generic reader.
    #[test]
    fn Test_A_Language_Version_This_Reader_Cannot_Account_For_Should_Be_Refused()
    {
        let refused = Parse_Manifest(&Manifest_With_Versions("[\"latest\"]"), "test")
            .expect_err("`latest` names no particular C# version");

        assert_eq!(
            refused,
            ManifestError::MalformedVersion {
                at: "test".to_owned(),
                field: "language_versions[0]".to_owned(),
                cause: "`latest` is not a `<major>` or `<major>.<minor>` C# version this reader knows".to_owned(),
            }
        );
    }

    /// The positive control for the refusal above: both label shapes resolve, so the test is
    /// about `latest` rather than about a reader that refuses everything.
    #[test]
    fn Test_Both_Label_Shapes_Should_Resolve_Through_The_Reader()
    {
        let manifest = Parse_Manifest(&Manifest_With_Versions("[\"7.3\", \"12\"]"), "test")
            .expect("both spellings are well-formed C# language versions");

        assert_eq!(
            manifest.language_versions,
            vec![CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)), CsharpVersion::New(MAJOR_ONLY, None)]
        );
    }

    /// That this crate's own allowlist really reaches the generic reader, rather than sitting
    /// beside it unread.
    #[test]
    fn Test_A_Provider_This_Package_Does_Not_Carry_Should_Be_Refused()
    {
        let refused = Parse_Manifest(&Manifest_With_Provider("nomos.lang.go.tree-sitter"), "test")
            .expect_err("the Go provider is not one a C# LanguagePackage registers");

        assert_eq!(
            refused,
            ManifestError::UnresolvedProvider {
                at: "test".to_owned(),
                provider: "nomos.lang.go.tree-sitter".to_owned(),
            }
        );
    }

    const LAST_MINOR_COMPONENT: u16 = 3;
    const LAST_MAJOR_WITH_A_MINOR: u16 = 7;
    const MAJOR_ONLY: u16 = 12;

    /// This crate's own manifest sits three directories below the repository root:
    /// `crates/packages/nomos-lang-csharp-package`.
    fn Repository_Root() -> PathBuf
    {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    }

    fn Real_Manifest_Path() -> PathBuf
    {
        return Repository_Root().join("packages/nomos.lang.csharp.json");
    }

    /// The manifest this test expects [`Read_Manifest`] to resolve, spelled out field by field so
    /// a change to `packages/nomos.lang.csharp.json` that silently drops a value fails a readable
    /// assertion rather than a byte comparison.
    fn Expected() -> LanguagePackage
    {
        return LanguagePackage {
            package_id: PackageId::New("nomos.lang.csharp"),
            package_kind: PackageKind::LanguagePackage,
            package_version: PackageVersion::New(1, 0, 0),
            protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
            language_versions: vec![
                CsharpVersion::New(LAST_MAJOR_WITH_A_MINOR, Some(LAST_MINOR_COMPONENT)),
                CsharpVersion::New(MAJOR_ONLY, None),
            ],
            providers: vec![ProviderRegistration {
                provider: ProviderId::New(nomos_lang_csharp::PROVIDER),
                tool_version: PackageVersion::New(0, 1, 0),
            }],
        };
    }

    fn Manifest_With_Versions(versions: &str) -> String
    {
        return Manifest(versions, nomos_lang_csharp::PROVIDER);
    }

    fn Manifest_With_Provider(provider: &str) -> String
    {
        return Manifest("[\"12\"]", provider);
    }

    /// A well-formed manifest but for whichever of the two fields a caller varies.
    fn Manifest(versions: &str, provider: &str) -> String
    {
        return format!(
            "{{\"schema_version\": 1, \
               \"package_id\": \"nomos.lang.csharp\", \
               \"package_kind\": \"LanguagePackage\", \
               \"package_version\": {{\"major\": 1, \"minor\": 0, \"patch\": 0}}, \
               \"protocol_range\": {{\"minimum\": {{\"major\": 1, \"minor\": 0}}, \
                                    \"maximum\": {{\"major\": 1, \"minor\": 0}}}}, \
               \"language_versions\": {versions}, \
               \"providers\": [{{\"provider_id\": \"{provider}\", \
                                \"tool_version\": {{\"major\": 0, \"minor\": 1, \"patch\": 0}}}}]}}"
        );
    }
}
