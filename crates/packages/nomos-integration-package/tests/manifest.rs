//! Exercises `Parse_Manifest` against a manifest declaring every field, each default and
//! each refusal it names, and `Read_Manifest` against the fixture transcribing this
//! repository's own placed rows.

use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_integration_package::{
    IntegrationPackage, ManifestError, MaterializationIntent, OwnedRegion, OwnershipClass, PackageVersion, Parse_Manifest,
    ProtocolRange, PublicationScope, Read_Manifest,
};
use std::path::PathBuf;

/// The fixture beside these tests, resolved from this crate's own manifest directory.
fn Fixture_Path() -> PathBuf
{
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nomos.integration.self.json");
}

fn Field(name: &str, value: &str) -> String
{
    return format!("\"{name}\": {value}");
}

/// One intent declaring every field, so a default is never what this text exercises.
const DECLARED_INTENT: &str = "{\"surface\": \"agent contract file\", \"source\": \"AGENTS.md\", \"target\": \"AGENTS.md\", \
                               \"ownership_class\": \"GeneratedOwned\", \"publication_scope\": \"Shared\"}";

/// A manifest declaring every field of one intent.
fn Manifest_Text() -> String
{
    return format!(
        "{{{}, {}, {}, {}, {}, {}}}",
        Field("schema_version", "1"),
        Field("package_id", "\"nomos.integration.test\""),
        Field("package_kind", "\"IntegrationPackage\""),
        Field("package_version", "{\"major\": 1, \"minor\": 0, \"patch\": 0}"),
        Field("protocol_range", "{\"minimum\": {\"major\": 1, \"minor\": 0}, \"maximum\": {\"major\": 1, \"minor\": 0}}"),
        Field("intents", &format!("[{DECLARED_INTENT}]")),
    );
}

fn Intent(surface: &str, path: &str, ownership_class: OwnershipClass, publication_scope: PublicationScope) -> MaterializationIntent
{
    return MaterializationIntent {
        surface: surface.to_owned(),
        source: path.to_owned(),
        target: path.to_owned(),
        ownership_class,
        publication_scope,
        owned_region: None,
    };
}

fn Expected() -> IntegrationPackage
{
    return IntegrationPackage {
        package_id: PackageId::New("nomos.integration.test"),
        package_kind: PackageKind::IntegrationPackage,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        intents: vec![Intent("agent contract file", "AGENTS.md", OwnershipClass::GeneratedOwned, PublicationScope::Shared)],
    };
}

/// The one declared intent with one of its `"key": "value"` pairs replaced.
fn With(pair: &str, replacement: &str) -> String
{
    return Manifest_Text().replacen(pair, replacement, 1);
}

fn Intent_Of(text: &str) -> MaterializationIntent
{
    let manifest = Parse_Manifest(text, "test").expect("parses");

    return manifest.intents.into_iter().next().expect("one intent");
}

#[test]
fn Test_A_Manifest_Declaring_Every_Field_Resolves()
{
    let manifest = Parse_Manifest(&Manifest_Text(), "test").expect("parses");

    assert_eq!(manifest, Expected());
}

/// `OD-PACKAGE-004`'s default, exercised by absence: the key is removed, not blanked.
#[test]
fn Test_An_Undeclared_Ownership_Class_Defaults_To_User_Owned()
{
    let text = With(", \"ownership_class\": \"GeneratedOwned\"", "");

    assert_eq!(Intent_Of(&text).ownership_class, OwnershipClass::UserOwned);
}

/// `OD-PACKAGE-005`'s default, exercised by absence: the key is removed, not blanked.
#[test]
fn Test_An_Undeclared_Publication_Scope_Defaults_To_Local()
{
    let text = With(", \"publication_scope\": \"Shared\"", "");

    assert_eq!(Intent_Of(&text).publication_scope, PublicationScope::Local);
}

/// The default covers absence only. A present key of the wrong shape is a malformed
/// manifest, and defaulting it would be the silent repair every sibling reader refuses.
#[test]
fn Test_An_Ownership_Class_Of_The_Wrong_Type_Is_Refused_Rather_Than_Defaulted()
{
    let text = With("\"ownership_class\": \"GeneratedOwned\"", "\"ownership_class\": null");

    let refusal = Parse_Manifest(&text, "test").expect_err("null is not a label, and not an absence either");

    assert_eq!(
        refusal,
        ManifestError::WrongType { at: "test".to_owned(), field: "intents[0].ownership_class".to_owned(), expected: "string".to_owned() }
    );
}

#[test]
fn Test_A_Publication_Scope_Of_The_Wrong_Type_Is_Refused_Rather_Than_Defaulted()
{
    let text = With("\"publication_scope\": \"Shared\"", "\"publication_scope\": 1");

    let refusal = Parse_Manifest(&text, "test").expect_err("a number is not a label, and not an absence either");

    assert_eq!(
        refusal,
        ManifestError::WrongType { at: "test".to_owned(), field: "intents[0].publication_scope".to_owned(), expected: "string".to_owned() }
    );
}

#[test]
fn Test_An_Unknown_Ownership_Class_Label_Is_Refused()
{
    let text = With("\"ownership_class\": \"GeneratedOwned\"", "\"ownership_class\": \"Owned\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("Owned is none of OD-PACKAGE-004's three classes");

    assert_eq!(
        refusal,
        ManifestError::UnknownOwnershipClass { at: "test".to_owned(), field: "intents[0].ownership_class".to_owned(), found: "Owned".to_owned() }
    );
}

#[test]
fn Test_An_Unknown_Publication_Scope_Label_Is_Refused()
{
    let text = With("\"publication_scope\": \"Shared\"", "\"publication_scope\": \"Public\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("Public is none of OD-PACKAGE-005's three scopes");

    assert_eq!(
        refusal,
        ManifestError::UnknownPublicationScope { at: "test".to_owned(), field: "intents[0].publication_scope".to_owned(), found: "Public".to_owned() }
    );
}

/// `owned_region` is absent for every class but `Composed`, so absence is the ordinary case
/// and not a default standing in for something.
#[test]
fn Test_An_Intent_Declaring_No_Owned_Region_Should_Carry_None()
{
    assert_eq!(Intent_Of(&Manifest_Text()).owned_region, None);
}

/// A `Composed` target declares where its owned region sits. The reader resolves the pair
/// and does not judge whether the class may carry one: that is a rule about what a write may
/// do, and this crate performs no write.
#[test]
fn Test_A_Declared_Owned_Region_Should_Resolve_To_Its_Two_Markers()
{
    let text = With(
        "\"ownership_class\": \"GeneratedOwned\"",
        "\"ownership_class\": \"Composed\", \"owned_region\": {\"opening_marker\": \"<!-- a -->\", \"closing_marker\": \"<!-- b -->\"}",
    );

    assert_eq!(Intent_Of(&text).owned_region, Some(OwnedRegion::New("<!-- a -->", "<!-- b -->")));
}

/// Half a region is not a region, and a reader that accepted one would hand the materializer
/// a boundary with one end.
#[test]
fn Test_An_Owned_Region_Missing_A_Marker_Should_Be_Refused()
{
    let text = With(
        "\"ownership_class\": \"GeneratedOwned\"",
        "\"ownership_class\": \"Composed\", \"owned_region\": {\"opening_marker\": \"<!-- a -->\"}",
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("the closing marker is absent");

    assert_eq!(
        refusal,
        ManifestError::MissingField { at: "test".to_owned(), field: "intents[0].owned_region.closing_marker".to_owned() }
    );
}

#[test]
fn Test_An_Owned_Region_Of_The_Wrong_Shape_Should_Be_Refused()
{
    let text = With("\"ownership_class\": \"GeneratedOwned\"", "\"owned_region\": \"between the markers\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("a region is an object, not a string");

    assert_eq!(
        refusal,
        ManifestError::WrongType { at: "test".to_owned(), field: "intents[0].owned_region".to_owned(), expected: "object".to_owned() }
    );
}

#[test]
fn Test_A_ToolProvider_Kind_Is_Refused()
{
    let text = With("\"IntegrationPackage\"", "\"ToolProvider\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("this reader does not read ToolProvider");

    assert_eq!(refusal, ManifestError::WrongPackageKind { at: "test".to_owned(), found: PackageKind::ToolProvider });
}

#[test]
fn Test_A_Package_Kind_Not_Defined_At_All_Is_Refused()
{
    let text = With("\"IntegrationPackage\"", "\"NotARealPackageKind\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("this kind does not exist");

    assert_eq!(refusal, ManifestError::UnknownPackageKind { at: "test".to_owned(), found: "NotARealPackageKind".to_owned() });
}

#[test]
fn Test_An_Absolute_Target_Is_Refused()
{
    for target in ["/etc/hosts", "\\\\\\\\server\\\\share", "C:\\\\Windows", "c:/Windows", "D:drive-relative"]
    {
        let text = With("\"target\": \"AGENTS.md\"", &format!("\"target\": \"{target}\""));

        let refusal = Parse_Manifest(&text, "test").expect_err("an absolute target names nowhere inside the repository");

        assert!(matches!(refusal, ManifestError::AbsoluteTarget { ref field, .. } if field == "intents[0].target"), "{target}: {refusal}");
    }
}

#[test]
fn Test_A_Target_Climbing_Above_The_Repository_Root_Is_Refused()
{
    for target in ["..", "../sibling", "docs/../../sibling", "./../sibling"]
    {
        let text = With("\"target\": \"AGENTS.md\"", &format!("\"target\": \"{target}\""));

        let refusal = Parse_Manifest(&text, "test").expect_err("a target above the root names nowhere inside the repository");

        assert!(matches!(refusal, ManifestError::EscapingTarget { ref field, .. } if field == "intents[0].target"), "{target}: {refusal}");
    }
}

/// The escape check is about where a path ends up, not whether it contains `..` at all:
/// a target that descends before climbing stays inside and is kept as spelled.
#[test]
fn Test_A_Target_That_Descends_Before_Climbing_Is_Accepted_As_Spelled()
{
    for target in ["docs/../AGENTS.md", "./AGENTS.md", "a/b/../c", ".github/workflows/gate.yml"]
    {
        let text = With("\"target\": \"AGENTS.md\"", &format!("\"target\": \"{target}\""));

        assert_eq!(Intent_Of(&text).target, target);
    }
}

#[test]
fn Test_An_Empty_Intents_List_Is_Refused()
{
    let text = With(&format!("[{DECLARED_INTENT}]"), "[]");

    let refusal = Parse_Manifest(&text, "test").expect_err("an IntegrationPackage declaring no placement materializes nothing");

    assert_eq!(refusal, ManifestError::EmptyList { at: "test".to_owned(), field: "intents".to_owned() });
}

#[test]
fn Test_A_Missing_Field_Is_Refused()
{
    let text = "{\"schema_version\": 1, \"package_id\": \"nomos.integration.test\"}";

    let refusal = Parse_Manifest(text, "test").expect_err("package_kind is absent");

    assert_eq!(refusal, ManifestError::MissingField { at: "test".to_owned(), field: "package_kind".to_owned() });
}

#[test]
fn Test_A_Missing_Intent_Field_Is_Refused_At_Its_Indexed_Position()
{
    let text = With("\"source\": \"AGENTS.md\", ", "");

    let refusal = Parse_Manifest(&text, "test").expect_err("the intent's source is absent");

    assert_eq!(refusal, ManifestError::MissingField { at: "test".to_owned(), field: "intents[0].source".to_owned() });
}

#[test]
fn Test_A_Wrong_Type_Field_Is_Refused()
{
    let text = With("\"package_version\": {\"major\": 1, \"minor\": 0, \"patch\": 0}", "\"package_version\": \"not an object\"");

    let refusal = Parse_Manifest(&text, "test").expect_err("package_version is a string, not an object");

    assert_eq!(
        refusal,
        ManifestError::WrongType { at: "test".to_owned(), field: "package_version".to_owned(), expected: "object".to_owned() }
    );
}

#[test]
fn Test_An_Inverted_Protocol_Range_Is_Refused()
{
    let text = With("\"minimum\": {\"major\": 1, \"minor\": 0}", "\"minimum\": {\"major\": 2, \"minor\": 0}");

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

/// The fixture transcribes the four rows `OD-PACKAGE-003`'s placement table marks "placed"
/// for this repository -- `AGENTS.md`, `CLAUDE.md`, `.claude/skills`,
/// `.github/workflows/gate.yml` -- with the surface label each row carries there.
///
/// Every one resolves to `UserOwned`, and the fixture declares no class for any of them,
/// because that is the class `OD-PACKAGE-004` gives them: its own table classifies exactly
/// four assets (`diagrams/relations.mmd`, `spec/domain-specification.md`, `README.md`,
/// `.claude/settings.local.json`) and none of these, and its rule for an asset with no
/// recorded class is `UserOwned`. That is also the truthful state of the four: hand-placed
/// and hand-maintained, with no renderer holding authority over any byte of them.
///
/// Every one is `Shared`: `.github/workflows/gate.yml` is a named `Shared` case in
/// `OD-PACKAGE-005`'s own table, and the other three are tracked and handed to every clone,
/// which is that record's definition of the scope.
#[test]
fn Test_The_Self_Fixture_Should_Resolve_To_The_Four_Placed_Rows()
{
    let manifest = Read_Manifest(&Fixture_Path()).expect("the fixture parses");

    assert_eq!(manifest.package_id, PackageId::New("nomos.integration.self"));
    assert_eq!(manifest.package_kind, PackageKind::IntegrationPackage);
    assert_eq!(
        manifest.intents,
        vec![
            Intent("agent contract file", "AGENTS.md", OwnershipClass::UserOwned, PublicationScope::Shared),
            Intent("per-agent adapter", "CLAUDE.md", OwnershipClass::UserOwned, PublicationScope::Shared),
            Intent("procedural skills", ".claude/skills", OwnershipClass::UserOwned, PublicationScope::Shared),
            Intent("CI projection", ".github/workflows/gate.yml", OwnershipClass::UserOwned, PublicationScope::Shared),
        ]
    );
}
