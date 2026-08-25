//! Exercises `Parse_Manifest` against a manifest shaped like a real shipped rule
//! (`Check_Completeness_Mirrors`) and every refusal it names.

use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, EvidenceClass, FactVariant, Guarantee,
    IncrementalGranularity, PackageId, PackageKind, ProviderId, RuleId,
};
use nomos_rule_package::{
    ApplicabilitySemantics, CapabilityRequirement, DiagnosticMapping, ManifestError,
    PackageVersion, Parse_Manifest, ProtocolRange, ProviderRegistration, RuleContract, RulePackage,
};

/// A manifest shaped exactly like `Check_Completeness_Mirrors`'s own real properties:
/// cites `D-134` version 2, always raises `Supported`, requires `nomos_cap_syntax` at
/// `Syntactic`/`Sound`/`Unknown`/`File`, and reports `EvidenceClass::Derived` -- the
/// convergent shape `OD-PACKAGE-008`'s four-rule measurement found.
fn Manifest_Text() -> &'static str
{
    return r#"{
        "schema_version": 1,
        "package_id": "nomos.rule.completeness-mirror",
        "package_kind": "RulePackage",
        "package_version": {"major": 1, "minor": 0, "patch": 0},
        "protocol_range": {"minimum": {"major": 1, "minor": 0}, "maximum": {"major": 1, "minor": 0}},
        "rule_id": "COMPLETENESS_MIRROR",
        "contract": {"record": "D-134", "version": 2},
        "applicability": "always_supported",
        "required_capabilities": [
            {
                "capability": "nomos.cap.syntax.tree",
                "minimum": {
                    "variant": "Syntactic",
                    "soundness": "Sound",
                    "completeness": "Unknown",
                    "incremental": "File"
                }
            }
        ],
        "evidence_schema": "Derived",
        "enhanced_implementation": [],
        "external_diagnostics": [],
        "correction_and_suppression": null,
        "examples": ["a completeness mirror missing a real member"],
        "counterexamples": ["a mirror whose members match reality"],
        "conformance_fixtures": [],
        "evaluation_corpus": null,
        "agent_guidance": [],
        "title": "Completeness mirrors must be kept in sync"
    }"#;
}

fn Expected() -> RulePackage
{
    return RulePackage {
        package_id: PackageId::New("nomos.rule.completeness-mirror"),
        package_kind: PackageKind::RulePackage,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        rule_id: RuleId::New("COMPLETENESS_MIRROR"),
        contract: Some(RuleContract::New("D-134".to_owned(), 2)),
        applicability: ApplicabilitySemantics::AlwaysSupported,
        required_capabilities: vec![CapabilityRequirement::New(
            CapabilityId::New("nomos.cap.syntax.tree"),
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        )],
        evidence_schema: EvidenceClass::Derived,
        enhanced_implementation: vec![],
        external_diagnostics: vec![],
        correction_and_suppression: None,
        examples: vec!["a completeness mirror missing a real member".to_owned()],
        counterexamples: vec!["a mirror whose members match reality".to_owned()],
        conformance_fixtures: vec![],
        evaluation_corpus: None,
        agent_guidance: vec![],
        title: "Completeness mirrors must be kept in sync".to_owned(),
    };
}

#[test]
fn Test_A_Manifest_Shaped_Like_A_Real_Shipped_Rule_Resolves()
{
    let manifest = Parse_Manifest(Manifest_Text(), "test").expect("parses");

    assert_eq!(manifest, Expected());
}

#[test]
fn Test_A_Rule_With_No_Contract_Resolves_To_None()
{
    let text = Manifest_Text().replacen("\"contract\": {\"record\": \"D-134\", \"version\": 2}", "\"contract\": null", 1);

    let manifest = Parse_Manifest(&text, "test").expect("Check_Naming_Convention has no contract, and that is real");

    assert_eq!(manifest.contract, None);
}

#[test]
fn Test_A_Structurally_Partial_Rule_Resolves()
{
    let text = Manifest_Text().replacen("\"always_supported\"", "\"structurally_partial\"", 1);

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(manifest.applicability, ApplicabilitySemantics::StructurallyPartial);
}

#[test]
fn Test_A_LanguagePackage_Kind_Is_Refused()
{
    let text = Manifest_Text().replacen("\"RulePackage\"", "\"LanguagePackage\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("this reader does not read LanguagePackage");

    assert_eq!(
        refusal,
        ManifestError::WrongPackageKind { at: "test".to_owned(), found: PackageKind::LanguagePackage }
    );
}

#[test]
fn Test_A_Package_Kind_Not_Defined_At_All_Is_Refused()
{
    let text = Manifest_Text().replacen("\"RulePackage\"", "\"NotARealPackageKind\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("this kind does not exist");

    assert_eq!(
        refusal,
        ManifestError::UnknownPackageKind { at: "test".to_owned(), found: "NotARealPackageKind".to_owned() }
    );
}

#[test]
fn Test_A_Missing_Field_Is_Refused()
{
    let text = "{\"schema_version\": 1, \"package_id\": \"nomos.rule.x\"}";

    let refusal = Parse_Manifest(text, "test").expect_err("package_kind is absent");

    assert_eq!(
        refusal,
        ManifestError::MissingField { at: "test".to_owned(), field: "package_kind".to_owned() }
    );
}

#[test]
fn Test_An_Unknown_Applicability_Semantics_Is_Refused()
{
    let text = Manifest_Text().replacen("\"always_supported\"", "\"sometimes_maybe\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("sometimes_maybe is not a real semantics value");

    assert_eq!(
        refusal,
        ManifestError::UnknownApplicabilitySemantics { at: "test".to_owned(), found: "sometimes_maybe".to_owned() }
    );
}

#[test]
fn Test_An_Unknown_Guarantee_Variant_Is_Refused()
{
    let text = Manifest_Text().replacen("\"Syntactic\"", "\"Telepathic\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("Telepathic is not a real FactVariant");

    assert_eq!(
        refusal,
        ManifestError::UnknownGuaranteeValue {
            at: "test".to_owned(),
            field: "required_capabilities[0].minimum.variant".to_owned(),
            found: "Telepathic".to_owned(),
        }
    );
}

#[test]
fn Test_An_Unknown_Evidence_Class_Is_Refused()
{
    let text = Manifest_Text().replacen("\"Derived\"", "\"Guessed\"", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("Guessed is not a real EvidenceClass");

    assert_eq!(
        refusal,
        ManifestError::UnknownEvidenceClass { at: "test".to_owned(), found: "Guessed".to_owned() }
    );
}

#[test]
fn Test_A_Rule_With_A_Correction_And_Suppression_Contract_Resolves()
{
    let text = Manifest_Text().replacen(
        "\"correction_and_suppression\": null",
        "\"correction_and_suppression\": {\"mechanical_correction_available\": true, \"suppression_supported\": true}",
        1,
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    let contract = manifest.correction_and_suppression.expect("was set");
    assert!(contract.mechanical_correction_available);
    assert!(contract.suppression_supported);
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

// The five fields `OD-PACKAGE-008`'s amendment typed speculatively -- no shipped rule
// populates any of them yet. Each gets a realistic populated fixture proven to round-trip,
// and a malformed shape proven to be refused the same way `reader.rs` refuses everything
// else.

#[test]
fn Test_A_Rule_With_An_Enhanced_Implementation_Round_Trips()
{
    let text = Manifest_Text().replacen(
        "\"enhanced_implementation\": []",
        "\"enhanced_implementation\": [{\"provider\": \"rust-analyzer\", \"tool_version\": \
         {\"major\": 0, \"minor\": 4, \"patch\": 0}}]",
        1,
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(
        manifest.enhanced_implementation,
        vec![ProviderRegistration {
            provider: ProviderId::New("rust-analyzer"),
            tool_version: PackageVersion::New(0, 4, 0),
        }]
    );
}

#[test]
fn Test_An_Enhanced_Implementation_Entry_Missing_Its_Provider_Is_Refused()
{
    let text = Manifest_Text().replacen(
        "\"enhanced_implementation\": []",
        "\"enhanced_implementation\": [{\"tool_version\": {\"major\": 0, \"minor\": 4, \"patch\": 0}}]",
        1,
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("the entry names no provider");

    assert_eq!(
        refusal,
        ManifestError::MissingField {
            at: "test".to_owned(),
            field: "enhanced_implementation[0].provider".to_owned(),
        }
    );
}

#[test]
fn Test_A_Rule_With_External_Diagnostics_Round_Trips()
{
    let text = Manifest_Text().replacen(
        "\"external_diagnostics\": []",
        "\"external_diagnostics\": [{\"external_tool\": \"nomos.lang.rust.clippy\", \
         \"external_code\": \"clippy::needless_return\", \"maps_to\": \"COMPLETENESS_MIRROR\"}]",
        1,
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(
        manifest.external_diagnostics,
        vec![DiagnosticMapping::New(
            ProviderId::New("nomos.lang.rust.clippy"),
            "clippy::needless_return".to_owned(),
            RuleId::New("COMPLETENESS_MIRROR"),
        )]
    );
}

#[test]
fn Test_An_External_Diagnostics_Entry_Missing_Its_Code_Is_Refused()
{
    let text = Manifest_Text().replacen(
        "\"external_diagnostics\": []",
        "\"external_diagnostics\": [{\"external_tool\": \"nomos.lang.rust.clippy\", \
         \"maps_to\": \"COMPLETENESS_MIRROR\"}]",
        1,
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("the entry names no external_code");

    assert_eq!(
        refusal,
        ManifestError::MissingField {
            at: "test".to_owned(),
            field: "external_diagnostics[0].external_code".to_owned(),
        }
    );
}

#[test]
fn Test_A_Correction_And_Suppression_Contract_Missing_Suppression_Supported_Is_Refused()
{
    let text = Manifest_Text().replacen(
        "\"correction_and_suppression\": null",
        "\"correction_and_suppression\": {\"mechanical_correction_available\": true}",
        1,
    );

    let refusal = Parse_Manifest(&text, "test").expect_err("the contract names no suppression_supported");

    assert_eq!(
        refusal,
        ManifestError::MissingField {
            at: "test".to_owned(),
            field: "correction_and_suppression.suppression_supported".to_owned(),
        }
    );
}

#[test]
fn Test_A_Rule_With_An_Evaluation_Corpus_Round_Trips()
{
    let text = Manifest_Text().replacen(
        "\"evaluation_corpus\": null",
        "\"evaluation_corpus\": \"tests/corpus/completeness-mirror\"",
        1,
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(manifest.evaluation_corpus, Some("tests/corpus/completeness-mirror".to_owned()));
}

#[test]
fn Test_An_Evaluation_Corpus_Of_The_Wrong_Type_Is_Refused()
{
    let text = Manifest_Text().replacen("\"evaluation_corpus\": null", "\"evaluation_corpus\": 42", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("42 is neither a string nor null");

    assert_eq!(
        refusal,
        ManifestError::WrongType {
            at: "test".to_owned(),
            field: "evaluation_corpus".to_owned(),
            expected: "string or null".to_owned(),
        }
    );
}

#[test]
fn Test_A_Rule_With_Agent_Guidance_Round_Trips()
{
    let text = Manifest_Text().replacen(
        "\"agent_guidance\": []",
        "\"agent_guidance\": [\"Prefer adding the missing member to the mirror's own \
         enumeration over deleting the mismatched real one.\"]",
        1,
    );

    let manifest = Parse_Manifest(&text, "test").expect("parses");

    assert_eq!(
        manifest.agent_guidance,
        vec![
            "Prefer adding the missing member to the mirror's own enumeration over deleting \
             the mismatched real one."
                .to_owned()
        ]
    );
}

#[test]
fn Test_An_Agent_Guidance_Entry_Of_The_Wrong_Type_Is_Refused()
{
    let text = Manifest_Text().replacen("\"agent_guidance\": []", "\"agent_guidance\": [42]", 1);

    let refusal = Parse_Manifest(&text, "test").expect_err("42 is not a string");

    assert_eq!(
        refusal,
        ManifestError::WrongType {
            at: "test".to_owned(),
            field: "agent_guidance[0]".to_owned(),
            expected: "string".to_owned(),
        }
    );
}
