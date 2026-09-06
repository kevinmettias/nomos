//! Exercises `Parse_Manifest` against a manifest naming both of this workspace's real
//! `ToolProvider`s, and every refusal it names.

use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
use nomos_tool_package::{Family, ManifestError, PackageVersion, Parse_Manifest, ProtocolRange, Read_Manifest, ToolPackage, ToolProviderRegistration};
use std::path::PathBuf;

fn Field(name: &str, value: &str) -> String
{
    return format!("\"{name}\": {value}");
}

/// The `providers` array naming both of this workspace's real `ToolProvider`s, referenced
/// by their own `PROVIDER` constants rather than hand-typed literals that could drift.
fn Providers_Json() -> String
{
    return format!(
        "[{{\"provider_id\": \"{}\", \"tool_version\": {{\"major\": 0, \"minor\": 1, \"patch\": 0}}, \
         \"family\": \"LINTER\"}}, {{\"provider_id\": \"{}\", \"tool_version\": {{\"major\": 0, \
         \"minor\": 1, \"patch\": 0}}, \"family\": \"PACKAGE_MANAGER\"}}]",
        nomos_lang_rust_clippy::PROVIDER,
        nomos_lang_rust_deny::PROVIDER,
    );
}

/// A manifest naming both of this workspace's real `ToolProvider`s.
fn Manifest_Text() -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.tool.rust\""),
        Field("package_kind", "\"ToolProvider\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("providers", &Providers_Json()),
    );
}

fn Expected() -> ToolPackage
{
    return ToolPackage {
        package_id: PackageId::New("nomos.tool.rust"),
        package_kind: PackageKind::ToolProvider,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        providers: vec![
            ToolProviderRegistration {
                provider: ProviderId::New(nomos_lang_rust_clippy::PROVIDER),
                tool_version: PackageVersion::New(0, 1, 0),
                family: Family::Linter,
            },
            ToolProviderRegistration {
                provider: ProviderId::New(nomos_lang_rust_deny::PROVIDER),
                tool_version: PackageVersion::New(0, 1, 0),
                family: Family::PackageManager,
            },
        ],
    };
}

#[test]
fn Test_A_Manifest_Naming_Both_Real_Tool_Providers_Resolves()
{
    let manifest = Parse_Manifest(&Manifest_Text(), "test").expect("parses");

    assert_eq!(manifest, Expected());
}

#[test]
fn Test_A_RulePackage_Kind_Is_Refused()
{
    let text = Manifest_Text().replacen("\"ToolProvider\"", "\"RulePackage\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("this reader does not read RulePackage");

    assert_eq!(refusal, ManifestError::WrongPackageKind { at: "test".to_owned(), found: PackageKind::RulePackage });
}

#[test]
fn Test_A_Package_Kind_Not_Defined_At_All_Is_Refused()
{
    let text = Manifest_Text().replacen("\"ToolProvider\"", "\"NotARealPackageKind\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("this kind does not exist");

    assert_eq!(refusal, ManifestError::UnknownPackageKind { at: "test".to_owned(), found: "NotARealPackageKind".to_owned() });
}

#[test]
fn Test_A_Missing_Field_Is_Refused()
{
    let text = "{\"schema_version\": 1, \"package_id\": \"nomos.tool.rust\"}";

    let refusal = Parse_Manifest(text, "test").expect_err("package_kind is absent");

    assert_eq!(refusal, ManifestError::MissingField { at: "test".to_owned(), field: "package_kind".to_owned() });
}

#[test]
fn Test_A_Wrong_Type_Field_Is_Refused()
{
    let text = Manifest_Text().replacen(
        "\"package_version\": {\"major\": 1, \"minor\": 0, \"patch\": 0}",
        "\"package_version\": \"not an object\"",
        1,
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("package_version is a string, not an object");

    assert_eq!(
        refusal,
        ManifestError::WrongType { at: "test".to_owned(), field: "package_version".to_owned(), expected: "object".to_owned() }
    );
}

#[test]
fn Test_An_Empty_Providers_List_Is_Refused()
{
    let text = Manifest_Text().replacen(&Providers_Json(), "[]", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("a ToolProvider manifest registering no provider registers nothing");

    assert_eq!(refusal, ManifestError::EmptyList { at: "test".to_owned(), field: "providers".to_owned() });
}

#[test]
fn Test_An_Unknown_Provider_Is_Refused()
{
    let text = Manifest_Text().replacen(nomos_lang_rust_clippy::PROVIDER, "nomos.lang.python.ast", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("this crate registers no Python provider");

    assert_eq!(refusal, ManifestError::UnresolvedProvider { at: "test".to_owned(), provider: "nomos.lang.python.ast".to_owned() });
}

#[test]
fn Test_An_Unknown_Family_Label_Is_Refused()
{
    let text = Manifest_Text().replacen("\"LINTER\"", "\"REVIEWER\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("REVIEWER is not one of the twelve closed FAMILY names");

    assert_eq!(
        refusal,
        ManifestError::MalformedVersion {
            at: "test".to_owned(),
            field: "providers[0].family".to_owned(),
            cause: "`REVIEWER` is not one of the twelve FAMILY names OD-CAPABILITY-013 closes the vocabulary at"
                .to_owned(),
        }
    );
}

#[test]
fn Test_An_Inverted_Protocol_Range_Is_Refused()
{
    let text = Manifest_Text().replacen(
        "\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}",
        "\"minimum\": {\"major\": 2, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}",
        1,
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("2.0 sorts after 1.0");

    assert_eq!(refusal, ManifestError::InvertedProtocolRange { at: "test".to_owned() });
}

#[test]
fn Test_A_Schema_Newer_Than_This_Build_Understands_Is_Refused()
{
    let text = "{\"schema_version\": 99}";

    let refusal = Parse_Manifest(text, "test").expect_err("schema 99 does not exist yet");

    assert_eq!(refusal, ManifestError::UnknownSchema { at: "test".to_owned(), understood: 1, found: 99 });
}

#[test]
fn Test_Text_That_Is_Not_Json_Is_Refused()
{
    let refusal = Parse_Manifest("not json", "test").expect_err("this is not JSON");

    assert!(matches!(refusal, ManifestError::NotJson { at, .. } if at == "test"));
}

#[test]
fn Test_A_Missing_File_Is_Refused_As_Unreadable()
{
    let refusal = Read_Manifest(&PathBuf::from("this/path/does/not/exist.json")).expect_err("nothing is there");

    assert!(matches!(refusal, ManifestError::Unreadable { .. }));
}
