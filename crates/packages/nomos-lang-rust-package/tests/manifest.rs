//! Round-trips this workspace's real Rust `LanguagePackage` manifest, and refuses every
//! malformed shape [`ManifestError`] names.

use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
use nomos_lang_rust_package::{
    LanguagePackage, ManifestError, PackageVersion, Parse_Manifest, ProtocolRange,
    ProviderRegistration, Read_Manifest, RustEdition,
};
use std::path::PathBuf;

/// This crate's own manifest sits three directories below the repository root:
/// `crates/packages/nomos-lang-rust-package`.
fn Repository_Root() -> PathBuf
{
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

fn Real_Manifest_Path() -> PathBuf
{
    return Repository_Root().join("packages/nomos.lang.rust.json");
}

/// The manifest this test expects `Read_Manifest` to resolve, spelled out field by
/// field so a change to `packages/nomos.lang.rust.json` that silently drops a value
/// fails a readable assertion rather than a byte comparison.
fn Expected() -> LanguagePackage
{
    return LanguagePackage {
        package_id: PackageId::New("nomos.lang.rust"),
        package_kind: PackageKind::LanguagePackage,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        language_versions: vec![
            RustEdition::Edition2015,
            RustEdition::Edition2018,
            RustEdition::Edition2021,
            RustEdition::Edition2024,
        ],
        providers: vec![
            ProviderRegistration {
                provider: ProviderId::New(nomos_lang_rust::PROVIDER),
                tool_version: PackageVersion::New(0, 1, 0),
            },
            ProviderRegistration {
                provider: ProviderId::New(nomos_lang_rust_scan::PROVIDER),
                tool_version: PackageVersion::New(0, 1, 0),
            },
        ],
    };
}

#[test]
fn Test_The_Real_Rust_Manifest_Should_Round_Trip()
{
    let manifest = Read_Manifest(&Real_Manifest_Path()).expect("the real manifest parses");

    assert_eq!(manifest, Expected());
}

/// The positive control's negative twin: `PackageKind::LanguagePackage` is what a real
/// consumer resolves against, so its serialized label has to be the string the real
/// manifest carries, or the round trip above would be passing by coincidence.
#[test]
fn Test_Language_Package_Serializes_As_The_Label_The_Manifest_Carries()
{
    assert_eq!(PackageKind::LanguagePackage.Label(), "LanguagePackage");
}

fn Field(name: &str, value: &str) -> String
{
    return format!("\"{name}\": {value}");
}

/// A manifest with every field but `package_id` -- the identity field, present in every
/// other test in this file so that this is the one case exercising its absence.
fn Manifest_Missing_Package_Id() -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_kind", "\"LanguagePackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", "[\"2021\"]"),
        Field(
            "providers",
            "[{\"provider_id\": \"nomos.lang.rust.syn\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]"
        ),
    );
}

#[test]
fn Test_A_Missing_Required_Field_Should_Be_Refused()
{
    let refused = Parse_Manifest(&Manifest_Missing_Package_Id(), "test")
        .expect_err("package_id is absent from this document");

    assert_eq!(
        refused,
        ManifestError::MissingField { at: "test".to_owned(), field: "package_id".to_owned() }
    );
}

/// A well-formed manifest, but for a `RulePackage` rather than a `LanguagePackage`.
fn Manifest_With_Kind(kind: &str) -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.rules.completeness-mirror\""),
        Field("package_kind", &format!("\"{kind}\"")),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", "[\"2021\"]"),
        Field(
            "providers",
            "[{\"provider_id\": \"nomos.lang.rust.syn\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]"
        ),
    );
}

#[test]
fn Test_A_Package_Kind_That_Is_Not_A_Language_Package_Should_Be_Refused()
{
    let refused = Parse_Manifest(&Manifest_With_Kind("RulePackage"), "test")
        .expect_err("RulePackage is a real PackageKind, and not this one");

    assert_eq!(
        refused,
        ManifestError::WrongPackageKind { at: "test".to_owned(), found: PackageKind::RulePackage }
    );
}

/// The other half of the same field: a `package_kind` this build does not define at all,
/// which is a different refusal from naming a real, wrong kind.
#[test]
fn Test_A_Package_Kind_Nothing_Defines_Should_Be_Refused_Differently_From_A_Wrong_One()
{
    let refused = Parse_Manifest(&Manifest_With_Kind("Plugin"), "test")
        .expect_err("Plugin is not one of the sixteen kinds PackageKind defines");

    assert_eq!(
        refused,
        ManifestError::UnknownPackageKind { at: "test".to_owned(), found: "Plugin".to_owned() }
    );
}

/// A well-formed manifest but for one malformed `language_versions` entry.
fn Manifest_With_Language_Version(value: &str) -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.lang.rust\""),
        Field("package_kind", "\"LanguagePackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", &format!("[\"{value}\"]")),
        Field(
            "providers",
            "[{\"provider_id\": \"nomos.lang.rust.syn\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]"
        ),
    );
}

#[test]
fn Test_A_Malformed_Version_Domain_Should_Be_Refused()
{
    let refused = Parse_Manifest(&Manifest_With_Language_Version("2027"), "test")
        .expect_err("Rust never shipped a 2027 edition");

    assert_eq!(
        refused,
        ManifestError::MalformedVersion {
            at: "test".to_owned(),
            field: "language_versions[0]".to_owned(),
            cause: "`2027` is not a Rust edition this reader knows".to_owned(),
        }
    );
}

/// A well-formed manifest but for one unresolvable provider registration.
fn Manifest_With_Provider(provider_id: &str) -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.lang.rust\""),
        Field("package_kind", "\"LanguagePackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", "[\"2021\"]"),
        Field(
            "providers",
            &format!(
                "[{{\"provider_id\": \"{provider_id}\", \"tool_version\": {{\"major\": 0, \"minor\": 1, \"patch\": 0}}}}]"
            )
        ),
    );
}

#[test]
fn Test_An_Unresolvable_Provider_Should_Be_Refused()
{
    let refused = Parse_Manifest(&Manifest_With_Provider("nomos.lang.python.ast"), "test")
        .expect_err("this crate registers no Python provider");

    assert_eq!(
        refused,
        ManifestError::UnresolvedProvider {
            at: "test".to_owned(),
            provider: "nomos.lang.python.ast".to_owned(),
        }
    );
}

/// A range whose ends are out of order is a distinct refusal from a value that will not
/// parse at all.
#[test]
fn Test_An_Inverted_Protocol_Range_Should_Be_Refused()
{
    let text = format!(
        "{{{}, {}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.lang.rust\""),
        Field("package_kind", "\"LanguagePackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 2, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("language_versions", "[\"2021\"]"),
        Field(
            "providers",
            "[{\"provider_id\": \"nomos.lang.rust.syn\", \"tool_version\": {\"major\": 0, \"minor\": 1, \"patch\": 0}}]"
        ),
    );

    let refused = Parse_Manifest(&text, "test").expect_err("2.0 sorts after 1.0");

    assert_eq!(refused, ManifestError::InvertedProtocolRange { at: "test".to_owned() });
}

/// Bytes that are not JSON at all refuse before any field is ever looked up.
#[test]
fn Test_Text_That_Is_Not_Json_Should_Be_Refused()
{
    let refused = Parse_Manifest("not json", "test").expect_err("this is not JSON");

    assert!(matches!(refused, ManifestError::NotJson { at, .. } if at == "test"));
}

/// A file that does not exist is a different refusal from a file that exists and does
/// not parse.
#[test]
fn Test_A_Missing_File_Should_Be_Refused_As_Unreadable()
{
    let refused = Read_Manifest(&PathBuf::from("this/path/does/not/exist.json"))
        .expect_err("nothing is there");

    assert!(matches!(refused, ManifestError::Unreadable { .. }));
}
