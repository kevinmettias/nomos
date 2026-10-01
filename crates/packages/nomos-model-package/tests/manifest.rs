//! Exercises `Parse_Manifest` against both accepted package kinds and every refusal it
//! names.

use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_model_package::{ManifestError, ModelRoutePackage, ModelSelection, PackageVersion, Parse_Manifest, ProtocolRange};

fn Field(name: &str, value: &str) -> String
{
    return format!("\"{name}\": {value}");
}

fn Manifest_Text(package_kind: &str, model_selection: &str) -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.model.acme\""),
        Field("package_kind", &format!("\"{package_kind}\"")),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("model_selection", model_selection),
    );
}

fn Expected(package_kind: PackageKind, model_selection: ModelSelection) -> ModelRoutePackage
{
    return ModelRoutePackage {
        package_id: PackageId::New("nomos.model.acme"),
        package_kind,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        model_selection,
    };
}

#[test]
fn Test_A_Well_Formed_ModelBackendPackage_Resolves_With_An_Opaque_Selection()
{
    let text = Manifest_Text("ModelBackendPackage", "{\"kind\": \"opaque\"}");

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(manifest, Expected(PackageKind::ModelBackendPackage, ModelSelection::Opaque));
}

#[test]
fn Test_A_Well_Formed_AgentExecutorPackage_Resolves_With_An_Executor_Controlled_Selection()
{
    let text = Manifest_Text("AgentExecutorPackage", "{\"kind\": \"executor_controlled\"}");

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(
        manifest,
        Expected(PackageKind::AgentExecutorPackage, ModelSelection::ExecutorControlled)
    );
}

#[test]
fn Test_A_Catalog_Selection_Carries_Its_Raw_Model_Identifiers()
{
    let text = Manifest_Text(
        "ModelBackendPackage",
        "{\"kind\": \"catalog\", \"models\": [\"acme-large\", \"acme-small\"]}",
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(
        manifest.model_selection,
        ModelSelection::Catalog(vec!["acme-large".to_owned(), "acme-small".to_owned()])
    );
}

#[test]
fn Test_A_LanguagePackage_Kind_Is_Refused()
{
    let text = Manifest_Text("LanguagePackage", "{\"kind\": \"opaque\"}");

    let refusal = Parse_Manifest(&text, "test").expect_err("this reader does not read LanguagePackage");

    assert_eq!(
        refusal,
        ManifestError::WrongPackageKind { at: "test".to_owned(), found: PackageKind::LanguagePackage }
    );
}

#[test]
fn Test_A_Package_Kind_Not_Defined_At_All_Is_Refused()
{
    let text = Manifest_Text("NotARealPackageKind", "{\"kind\": \"opaque\"}");

    let refusal = Parse_Manifest(&text, "test").expect_err("this kind does not exist");

    assert_eq!(
        refusal,
        ManifestError::UnknownPackageKind { at: "test".to_owned(), found: "NotARealPackageKind".to_owned() }
    );
}

#[test]
fn Test_A_Missing_Field_Is_Refused()
{
    let text = "{\"schema_version\": 1, \"package_id\": \"nomos.model.acme\"}";

    let refusal = Parse_Manifest(text, "test").expect_err("package_kind is absent");

    assert_eq!(
        refusal,
        ManifestError::MissingField { at: "test".to_owned(), field: "package_kind".to_owned() }
    );
}

#[test]
fn Test_An_Unknown_Model_Selection_Kind_Is_Refused()
{
    let text = Manifest_Text("ModelBackendPackage", "{\"kind\": \"psychic\"}");

    let refusal = Parse_Manifest(&text, "test").expect_err("psychic is not a real selection kind");

    assert_eq!(
        refusal,
        ManifestError::UnknownModelSelectionKind { at: "test".to_owned(), found: "psychic".to_owned() }
    );
}

#[test]
fn Test_An_Empty_Catalog_Is_Refused()
{
    let text = Manifest_Text("ModelBackendPackage", "{\"kind\": \"catalog\", \"models\": []}");

    let refusal = Parse_Manifest(&text, "test").expect_err("a catalog naming no model names nothing");

    assert_eq!(
        refusal,
        ManifestError::EmptyList { at: "test".to_owned(), field: "model_selection.models".to_owned() }
    );
}

#[test]
fn Test_An_Inverted_Protocol_Range_Is_Refused()
{
    let text = format!(
        "{{{}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.model.acme\""),
        Field("package_kind", "\"ModelBackendPackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field(
            "protocol_range",
            "{\"minimum\": {\"major\": 2, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"
        ),
        Field("model_selection", "{\"kind\": \"opaque\"}"),
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
