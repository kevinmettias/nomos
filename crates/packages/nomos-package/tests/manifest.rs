//! Exercises the generic reader with a `known_providers` list of its own -- not Rust's --
//! proving `OD-PACKAGE-007`'s split works for a provider set other than the one it was
//! extracted from.

use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
use nomos_package::{
    KnownProviders, ManifestError, PackageManifest, PackageVersion, Parse_Manifest, ProtocolRange,
    ProviderRegistration,
};

const KNOWN_PROVIDERS: [&str; 2] = ["nomos.lang.python.ast", "nomos.lang.python.scan"];

fn Field(name: &str, value: &str) -> String
{
    return format!("\"{name}\": {value}");
}

fn Manifest_Text(language_versions: &str, providers: &str) -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.lang.python\""),
        Field("package_kind", "\"LanguagePackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", language_versions),
        Field("providers", providers),
    );
}

fn Well_Formed() -> String
{
    return Manifest_Text(
        "[\"3.11\", \"3.12\"]",
        "[{\"provider_id\": \"nomos.lang.python.ast\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]",
    );
}

#[test]
fn Test_A_Well_Formed_Manifest_Resolves_With_A_Non_Rust_Provider_List()
{
    let manifest = Parse_Manifest(&Well_Formed(), "test", &KNOWN_PROVIDERS).expect("parses");

    assert_eq!(
        manifest,
        PackageManifest {
            package_id: PackageId::New("nomos.lang.python"),
            package_kind: PackageKind::LanguagePackage,
            package_version: PackageVersion::New(1, 0, 0),
            protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
            language_versions: vec!["3.11".to_owned(), "3.12".to_owned()],
            providers: vec![ProviderRegistration {
                provider: ProviderId::New("nomos.lang.python.ast"),
                tool_version: PackageVersion::New(0, 1, 0),
            }],
        }
    );
}

/// `language_versions` is carried as raw labels: this reader has no opinion on what a
/// Python version string looks like, unlike `nomos-lang-package`'s `RustEdition`
/// resolution one layer up.
#[test]
fn Test_Language_Versions_Are_Never_Resolved_To_A_Typed_Domain()
{
    let text = Manifest_Text(
        "[\"not-a-real-python-version\"]",
        "[{\"provider_id\": \"nomos.lang.python.ast\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]",
    );

    let manifest = Parse_Manifest(&text, "test", &KNOWN_PROVIDERS).expect("this reader resolves any label");

    assert_eq!(manifest.language_versions, vec!["not-a-real-python-version".to_owned()]);
}

#[test]
fn Test_A_Provider_Not_In_The_Callers_Allowlist_Is_Refused()
{
    let text = Manifest_Text(
        "[\"3.11\"]",
        "[{\"provider_id\": \"nomos.lang.rust.syn\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]",
    );

    let refused = Parse_Manifest(&text, "test", &KNOWN_PROVIDERS)
        .expect_err("nomos.lang.rust.syn is not in this caller's allowlist");

    assert_eq!(
        refused,
        ManifestError::UnresolvedProvider { at: "test".to_owned(), provider: "nomos.lang.rust.syn".to_owned() }
    );
}

/// `OD-PACKAGE-006`'s generic allowlist type, populated with a non-Rust provider set and
/// pulled through by reference the same way `nomos-lang-package::KNOWN_PROVIDERS` pulls
/// Rust's -- proof this base type serves a second language, not only the one it was
/// generalized from.
#[test]
fn Test_A_Known_Providers_Allowlist_Resolves_Through_The_Generic_Reader()
{
    let known_providers = KnownProviders::New(&["nomos.lang.python.ast", "nomos.lang.python.scan"]);

    let manifest =
        Parse_Manifest(&Well_Formed(), "test", known_providers.As_Slice()).expect("parses");

    assert_eq!(
        manifest.providers,
        vec![ProviderRegistration {
            provider: ProviderId::New("nomos.lang.python.ast"),
            tool_version: PackageVersion::New(0, 1, 0),
        }]
    );
}

#[test]
fn Test_An_Empty_Language_Versions_List_Is_Refused()
{
    let text = Manifest_Text(
        "[]",
        "[{\"provider_id\": \"nomos.lang.python.ast\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]",
    );

    let refused =
        Parse_Manifest(&text, "test", &KNOWN_PROVIDERS).expect_err("an empty list recognizes nothing");

    assert_eq!(
        refused,
        ManifestError::EmptyList { at: "test".to_owned(), field: "language_versions".to_owned() }
    );
}

/// A schema newer than this build understands is refused before any other field is read --
/// `Well_Formed` below is otherwise a valid manifest, so a refusal here can only be the
/// schema check, not some other field this reader happens to reject too.
#[test]
fn Test_A_Manifest_Newer_Than_This_Build_Understands_Is_Refused()
{
    let text = Well_Formed().replacen("\"schema_version\": 1", "\"schema_version\": 999", 1);

    let refused = Parse_Manifest(&text, "test", &KNOWN_PROVIDERS)
        .expect_err("schema 999 is newer than this build understands");

    assert_eq!(
        refused,
        ManifestError::UnknownSchema { at: "test".to_owned(), understood: nomos_package::SCHEMA_VERSION, found: 999 }
    );
}
