//! Reading a `RulePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! The same discipline `nomos_package::reader` and `nomos_model_package::reader` already
//! hold: a missing field, a `package_kind` this crate does not read, a version domain
//! that will not parse, an enum value this crate does not define -- each is a named
//! [`ManifestError`] variant, never a default or a skip.

use crate::{
    ApplicabilitySemantics, CapabilityRequirement, CorrectionAndSuppressionContract,
    DiagnosticMapping, Judgment, RuleContract, RulePackage,
};
use nomos_contracts::{
    Assurance, CapabilityId, EvidenceClass, FactVariant, Guarantee, IncrementalGranularity,
    PackageId, PackageKind, ProviderId, RuleId,
};
use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
use serde_json::{Map, Value};
use std::path::Path;

/// The highest manifest schema this build can read.
pub const SCHEMA_VERSION: u32 = 1;

/// Why a manifest was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError
{
    /// The file did not read.
    Unreadable
    {
        at: String,
        cause: String,
    },
    /// The bytes are not valid JSON at all.
    NotJson
    {
        at: String,
        cause: String,
    },
    /// A required key is absent.
    MissingField
    {
        at: String,
        field: String,
    },
    /// A key holds a value of the wrong JSON shape for what this format needs there.
    WrongType
    {
        at: String,
        field: String,
        expected: String,
    },
    /// `package_kind` names something [`PackageKind`] does not define at all.
    UnknownPackageKind
    {
        at: String,
        found: String,
    },
    /// `package_kind` is a real [`PackageKind`], but not `RulePackage`.
    WrongPackageKind
    {
        at: String,
        found: PackageKind,
    },
    /// A version domain's value could not be resolved to the type this format uses
    /// there.
    MalformedVersion
    {
        at: String,
        field: String,
        cause: String,
    },
    /// `protocol_range.minimum` sorts after `protocol_range.maximum`.
    InvertedProtocolRange
    {
        at: String,
    },
    /// `applicability` names something [`ApplicabilitySemantics`] does not define.
    UnknownApplicabilitySemantics
    {
        at: String,
        found: String,
    },
    /// `judgment` names something [`Judgment`] does not define.
    UnknownJudgment
    {
        at: String,
        found: String,
    },
    /// A `required_capabilities[n].minimum` sub-field names something its own enum does
    /// not define.
    UnknownGuaranteeValue
    {
        at: String,
        field: String,
        found: String,
    },
    /// `evidence_schema` names something [`EvidenceClass`] does not define.
    UnknownEvidenceClass
    {
        at: String,
        found: String,
    },
    /// `schema_version` names a manifest shape newer than this build understands.
    UnknownSchema
    {
        at: String,
        understood: u32,
        found: u32,
    },
}

impl core::fmt::Display for ManifestError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unreadable { at, cause } => write!(formatter, "{at} did not read: {cause}."),
            Self::NotJson { at, cause } => write!(formatter, "{at} is not valid JSON: {cause}."),
            Self::MissingField { at, field } => write!(formatter, "{at} carries no `{field}` field."),
            Self::WrongType { at, field, expected } =>
            {
                write!(formatter, "{at}'s `{field}` is not a {expected}.")
            }
            Self::UnknownPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, which is not a PackageKind this build defines."
            ),
            Self::WrongPackageKind { at, found } =>
            {
                write!(formatter, "{at}'s `package_kind` is `{found}`, not `RulePackage`.")
            }
            Self::MalformedVersion { at, field, cause } =>
            {
                write!(formatter, "{at}'s `{field}` did not resolve to a version: {cause}.")
            }
            Self::InvertedProtocolRange { at } => write!(
                formatter,
                "{at}'s `protocol_range.minimum` sorts after `protocol_range.maximum`."
            ),
            Self::UnknownApplicabilitySemantics { at, found } => write!(
                formatter,
                "{at}'s `applicability` is `{found}`, which is not `always_supported` or \
                 `structurally_partial`."
            ),
            Self::UnknownJudgment { at, found } => write!(
                formatter,
                "{at}'s `judgment` is `{found}`, which is not `mechanical` or `model_judged`."
            ),
            Self::UnknownGuaranteeValue { at, field, found } =>
            {
                write!(formatter, "{at}'s `{field}` is `{found}`, not a value that field defines.")
            }
            Self::UnknownEvidenceClass { at, found } => write!(
                formatter,
                "{at}'s `evidence_schema` is `{found}`, which is not an EvidenceClass this build defines."
            ),
            Self::UnknownSchema { at, understood, found } => write!(
                formatter,
                "{at}'s `schema_version` is {found}, and this build only understands manifest \
                 schema {understood}."
            ),
        };
    }
}

/// Reads and resolves a manifest from a file.
///
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Read_Manifest(path: &Path) -> Result<RulePackage, ManifestError>
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
pub fn Parse_Manifest(text: &str, at: &str) -> Result<RulePackage, ManifestError>
{
    let value: Value = serde_json::from_str(text).map_err(|error| {
        return ManifestError::NotJson { at: at.to_owned(), cause: error.to_string() };
    })?;
    let root = Object_At(&value, at, "<root>")?;

    Schema_Version_Field(root, at)?;

    let package_id = String_At(root, "package_id", at, "package_id")?;
    let package_kind = Package_Kind_Field(root, at)?;
    let package_version = Version_Triple_At(root, "package_version", at, "package_version")?;
    let protocol_range = Protocol_Range_Field(root, at)?;
    let rule_id = String_At(root, "rule_id", at, "rule_id")?;
    let contract = Contract_Field(root, at)?;
    let judgment = Judgment_Field(root, at)?;
    let applicability = Applicability_Field(root, at)?;
    let required_capabilities = Required_Capabilities_Field(root, at)?;
    let evidence_schema = Evidence_Schema_Field(root, at)?;
    let enhanced_implementation = Enhanced_Implementation_Field(root, at)?;
    let external_diagnostics = External_Diagnostics_Field(root, at)?;
    let correction_and_suppression = Correction_And_Suppression_Field(root, at)?;
    let examples = String_Array_At(root, "examples", at, "examples")?;
    let counterexamples = String_Array_At(root, "counterexamples", at, "counterexamples")?;
    let conformance_fixtures = String_Array_At(root, "conformance_fixtures", at, "conformance_fixtures")?;
    let evaluation_corpus = Optional_String_At(root, "evaluation_corpus", at, "evaluation_corpus")?;
    let agent_guidance = String_Array_At(root, "agent_guidance", at, "agent_guidance")?;
    let title = String_At(root, "title", at, "title")?;

    return Ok(RulePackage {
        package_id: PackageId::New(package_id),
        package_kind,
        package_version,
        protocol_range,
        rule_id: RuleId::New(rule_id),
        contract,
        judgment,
        applicability,
        required_capabilities,
        evidence_schema,
        enhanced_implementation,
        external_diagnostics,
        correction_and_suppression,
        examples,
        counterexamples,
        conformance_fixtures,
        evaluation_corpus,
        agent_guidance,
        title,
    });
}

fn Object_At<'a>(value: &'a Value, at: &str, full: &str) -> Result<&'a Map<String, Value>, ManifestError>
{
    return value.as_object().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: full.to_owned(), expected: "object".to_owned() };
    });
}

fn Field_At<'a>(object: &'a Map<String, Value>, key: &str, at: &str, full: &str) -> Result<&'a Value, ManifestError>
{
    return object.get(key).ok_or_else(|| {
        return ManifestError::MissingField { at: at.to_owned(), field: full.to_owned() };
    });
}

fn String_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<String, ManifestError>
{
    let value = Field_At(object, key, at, full)?;

    return value.as_str().map(str::to_owned).ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: full.to_owned(), expected: "string".to_owned() };
    });
}

fn Bool_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<bool, ManifestError>
{
    let value = Field_At(object, key, at, full)?;

    return value.as_bool().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: full.to_owned(), expected: "boolean".to_owned() };
    });
}

fn Array_At<'a>(object: &'a Map<String, Value>, key: &str, at: &str, full: &str) -> Result<&'a Vec<Value>, ManifestError>
{
    let value = Field_At(object, key, at, full)?;

    return value.as_array().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: full.to_owned(), expected: "array".to_owned() };
    });
}

fn String_Array_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<Vec<String>, ManifestError>
{
    let array = Array_At(object, key, at, full)?;
    let mut values = Vec::with_capacity(array.len());

    for (index, item) in array.iter().enumerate()
    {
        let field = format!("{full}[{index}]");
        let label = item.as_str().ok_or_else(|| {
            return ManifestError::WrongType { at: at.to_owned(), field: field.clone(), expected: "string".to_owned() };
        })?;

        values.push(label.to_owned());
    }

    return Ok(values);
}

fn Optional_String_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<Option<String>, ManifestError>
{
    let Some(raw) = object.get(key)
    else
    {
        return Err(ManifestError::MissingField { at: at.to_owned(), field: full.to_owned() });
    };

    if raw.is_null()
    {
        return Ok(None);
    }

    let label = raw.as_str().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: full.to_owned(), expected: "string or null".to_owned() };
    })?;

    return Ok(Some(label.to_owned()));
}

fn U16_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<u16, ManifestError>
{
    let value = Field_At(object, key, at, full)?;
    let found = value.as_u64().ok_or_else(|| {
        return ManifestError::MalformedVersion {
            at: at.to_owned(),
            field: full.to_owned(),
            cause: "not a non-negative integer".to_owned(),
        };
    })?;

    return u16::try_from(found).map_err(|_| {
        return ManifestError::MalformedVersion {
            at: at.to_owned(),
            field: full.to_owned(),
            cause: format!("{found} does not fit in sixteen bits"),
        };
    });
}

fn U32_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<u32, ManifestError>
{
    let value = Field_At(object, key, at, full)?;
    let found = value.as_u64().ok_or_else(|| {
        return ManifestError::MalformedVersion {
            at: at.to_owned(),
            field: full.to_owned(),
            cause: "not a non-negative integer".to_owned(),
        };
    })?;

    return u32::try_from(found).map_err(|_| {
        return ManifestError::MalformedVersion {
            at: at.to_owned(),
            field: full.to_owned(),
            cause: format!("{found} does not fit in thirty-two bits"),
        };
    });
}

fn Version_Triple_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<PackageVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;
    let patch = U16_At(inner, "patch", at, &format!("{full}.patch"))?;

    return Ok(PackageVersion::New(major, minor, patch));
}

fn Contract_Version_At(
    object: &Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<nomos_contracts::ContractVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;

    return Ok(nomos_contracts::ContractVersion::New(major, minor));
}

fn Schema_Version_Field(object: &Map<String, Value>, at: &str) -> Result<(), ManifestError>
{
    let raw = Field_At(object, "schema_version", at, "schema_version")?;
    let found = raw.as_u64().and_then(|value| return u32::try_from(value).ok()).ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: "schema_version".to_owned(),
            expected: "non-negative integer".to_owned(),
        };
    })?;

    if found > SCHEMA_VERSION
    {
        return Err(ManifestError::UnknownSchema { at: at.to_owned(), understood: SCHEMA_VERSION, found });
    }

    return Ok(());
}

/// `package_kind`, resolved and checked against the one kind this crate reads.
fn Package_Kind_Field(object: &Map<String, Value>, at: &str) -> Result<PackageKind, ManifestError>
{
    let label = String_At(object, "package_kind", at, "package_kind")?;
    let parsed: PackageKind = serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownPackageKind { at: at.to_owned(), found: label.clone() };
    })?;

    if parsed != PackageKind::RulePackage
    {
        return Err(ManifestError::WrongPackageKind { at: at.to_owned(), found: parsed });
    }

    return Ok(parsed);
}

fn Protocol_Range_Field(object: &Map<String, Value>, at: &str) -> Result<ProtocolRange, ManifestError>
{
    let raw = Field_At(object, "protocol_range", at, "protocol_range")?;
    let inner = Object_At(raw, at, "protocol_range")?;
    let minimum = Contract_Version_At(inner, "minimum", at, "protocol_range.minimum")?;
    let maximum = Contract_Version_At(inner, "maximum", at, "protocol_range.maximum")?;
    let range = ProtocolRange::New(minimum, maximum);

    if !range.Is_Ordered()
    {
        return Err(ManifestError::InvertedProtocolRange { at: at.to_owned() });
    }

    return Ok(range);
}

/// `contract`, resolved to `None` when the key is JSON `null` -- a rule may genuinely
/// have no versioned governing record to cite.
fn Contract_Field(object: &Map<String, Value>, at: &str) -> Result<Option<RuleContract>, ManifestError>
{
    let Some(raw) = object.get("contract")
    else
    {
        return Err(ManifestError::MissingField { at: at.to_owned(), field: "contract".to_owned() });
    };

    if raw.is_null()
    {
        return Ok(None);
    }

    let inner = Object_At(raw, at, "contract")?;
    let record = String_At(inner, "record", at, "contract.record")?;
    let version = U32_At(inner, "version", at, "contract.version")?;

    return Ok(Some(RuleContract::New(record, version)));
}

/// `judgment`, which every manifest must state.
///
/// Required rather than defaulted to `mechanical`. A manifest that does not say whether
/// anything judges its rule is exactly the ambiguity `OD-RULES-022` closes, and a default
/// would resolve it silently in the direction that makes a model-judged rule look like a
/// mechanical one whose implementation is missing.
fn Judgment_Field(object: &Map<String, Value>, at: &str) -> Result<Judgment, ManifestError>
{
    let label = String_At(object, "judgment", at, "judgment")?;

    return match label.as_str()
    {
        "mechanical" => Ok(Judgment::Mechanical),
        "model_judged" => Ok(Judgment::ModelJudged),
        _ => Err(ManifestError::UnknownJudgment { at: at.to_owned(), found: label }),
    };
}

fn Applicability_Field(object: &Map<String, Value>, at: &str) -> Result<ApplicabilitySemantics, ManifestError>
{
    let label = String_At(object, "applicability", at, "applicability")?;

    return match label.as_str()
    {
        "always_supported" => Ok(ApplicabilitySemantics::AlwaysSupported),
        "structurally_partial" => Ok(ApplicabilitySemantics::StructurallyPartial),
        _ => Err(ManifestError::UnknownApplicabilitySemantics { at: at.to_owned(), found: label }),
    };
}

fn Guarantee_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<Guarantee, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;

    let variant_label = String_At(inner, "variant", at, &format!("{full}.variant"))?;
    let variant: FactVariant = serde_json::from_value(Value::String(variant_label.clone())).map_err(|_| {
        return ManifestError::UnknownGuaranteeValue {
            at: at.to_owned(),
            field: format!("{full}.variant"),
            found: variant_label.clone(),
        };
    })?;

    let soundness_label = String_At(inner, "soundness", at, &format!("{full}.soundness"))?;
    let soundness: Assurance = serde_json::from_value(Value::String(soundness_label.clone())).map_err(|_| {
        return ManifestError::UnknownGuaranteeValue {
            at: at.to_owned(),
            field: format!("{full}.soundness"),
            found: soundness_label.clone(),
        };
    })?;

    let completeness_label = String_At(inner, "completeness", at, &format!("{full}.completeness"))?;
    let completeness: Assurance = serde_json::from_value(Value::String(completeness_label.clone())).map_err(|_| {
        return ManifestError::UnknownGuaranteeValue {
            at: at.to_owned(),
            field: format!("{full}.completeness"),
            found: completeness_label.clone(),
        };
    })?;

    let incremental_label = String_At(inner, "incremental", at, &format!("{full}.incremental"))?;
    let incremental: IncrementalGranularity =
        serde_json::from_value(Value::String(incremental_label.clone())).map_err(|_| {
            return ManifestError::UnknownGuaranteeValue {
                at: at.to_owned(),
                field: format!("{full}.incremental"),
                found: incremental_label.clone(),
            };
        })?;

    return Ok(Guarantee::New(variant, soundness, completeness, incremental));
}

fn Required_Capabilities_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<CapabilityRequirement>, ManifestError>
{
    let array = Array_At(object, "required_capabilities", at, "required_capabilities")?;
    let mut requirements = Vec::with_capacity(array.len());

    for (index, item) in array.iter().enumerate()
    {
        let full = format!("required_capabilities[{index}]");
        let inner = Object_At(item, at, &full)?;
        let capability = String_At(inner, "capability", at, &format!("{full}.capability"))?;
        let minimum = Guarantee_At(inner, "minimum", at, &format!("{full}.minimum"))?;

        requirements.push(CapabilityRequirement::New(CapabilityId::New(capability), minimum));
    }

    return Ok(requirements);
}

fn Evidence_Schema_Field(object: &Map<String, Value>, at: &str) -> Result<EvidenceClass, ManifestError>
{
    let label = String_At(object, "evidence_schema", at, "evidence_schema")?;

    return serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownEvidenceClass { at: at.to_owned(), found: label };
    });
}

fn Enhanced_Implementation_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<ProviderRegistration>, ManifestError>
{
    let array = Array_At(object, "enhanced_implementation", at, "enhanced_implementation")?;
    let mut registrations = Vec::with_capacity(array.len());

    for (index, item) in array.iter().enumerate()
    {
        let full = format!("enhanced_implementation[{index}]");
        let inner = Object_At(item, at, &full)?;
        let provider = String_At(inner, "provider", at, &format!("{full}.provider"))?;
        let tool_version = Version_Triple_At(inner, "tool_version", at, &format!("{full}.tool_version"))?;

        registrations.push(ProviderRegistration { provider: ProviderId::New(provider), tool_version });
    }

    return Ok(registrations);
}

fn External_Diagnostics_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<DiagnosticMapping>, ManifestError>
{
    let array = Array_At(object, "external_diagnostics", at, "external_diagnostics")?;
    let mut mappings = Vec::with_capacity(array.len());

    for (index, item) in array.iter().enumerate()
    {
        let full = format!("external_diagnostics[{index}]");
        let inner = Object_At(item, at, &full)?;
        let external_tool = String_At(inner, "external_tool", at, &format!("{full}.external_tool"))?;
        let external_code = String_At(inner, "external_code", at, &format!("{full}.external_code"))?;
        let maps_to = String_At(inner, "maps_to", at, &format!("{full}.maps_to"))?;

        mappings.push(DiagnosticMapping::New(
            ProviderId::New(external_tool),
            external_code,
            RuleId::New(maps_to),
        ));
    }

    return Ok(mappings);
}

fn Correction_And_Suppression_Field(
    object: &Map<String, Value>,
    at: &str,
) -> Result<Option<CorrectionAndSuppressionContract>, ManifestError>
{
    let Some(raw) = object.get("correction_and_suppression")
    else
    {
        return Err(ManifestError::MissingField {
            at: at.to_owned(),
            field: "correction_and_suppression".to_owned(),
        });
    };

    if raw.is_null()
    {
        return Ok(None);
    }

    let inner = Object_At(raw, at, "correction_and_suppression")?;
    let mechanical_correction_available = Bool_At(
        inner,
        "mechanical_correction_available",
        at,
        "correction_and_suppression.mechanical_correction_available",
    )?;
    let suppression_supported = Bool_At(
        inner,
        "suppression_supported",
        at,
        "correction_and_suppression.suppression_supported",
    )?;

    return Ok(Some(CorrectionAndSuppressionContract::New(
        mechanical_correction_available,
        suppression_supported,
    )));
}
