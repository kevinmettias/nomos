//! Reading a `ModelRoutePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! The same discipline `nomos_package::reader` and `nomos_spec_store`'s registration
//! reader already hold: a missing field, a `package_kind` this crate does not read, a
//! version domain that will not parse, a `model_selection.kind` this crate does not
//! define -- each is a named [`ManifestError`] variant, never a default or a skip.

use crate::manifest::ModelRoutePackage;
use crate::model_selection::ModelSelection;
use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_package::ProtocolRange;
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
    /// A list that must name at least one value names none.
    EmptyList
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
    /// `package_kind` is a real [`PackageKind`], but neither `ModelBackendPackage` nor
    /// `AgentExecutorPackage`.
    WrongPackageKind
    {
        at: String,
        found: PackageKind,
    },
    /// A version domain's value could not be resolved to the type this format uses there.
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
    /// `model_selection.kind` names something [`ModelSelection`] does not define.
    UnknownModelSelectionKind
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
            Self::EmptyList { at, field } => write!(formatter, "{at}'s `{field}` names no value."),
            Self::WrongType { at, field, expected } =>
            {
                write!(formatter, "{at}'s `{field}` is not a {expected}.")
            }
            Self::UnknownPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, which is not a PackageKind this build defines."
            ),
            Self::WrongPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, neither `ModelBackendPackage` nor `AgentExecutorPackage`."
            ),
            Self::MalformedVersion { at, field, cause } =>
            {
                write!(formatter, "{at}'s `{field}` did not resolve to a version: {cause}.")
            }
            Self::InvertedProtocolRange { at } => write!(
                formatter,
                "{at}'s `protocol_range.minimum` sorts after `protocol_range.maximum`."
            ),
            Self::UnknownModelSelectionKind { at, found } => write!(
                formatter,
                "{at}'s `model_selection.kind` is `{found}`, which is not `opaque`, \
                 `executor_controlled` or `catalog`."
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
pub fn Read_Manifest(path: &Path) -> Result<ModelRoutePackage, ManifestError>
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
pub fn Parse_Manifest(text: &str, at: &str) -> Result<ModelRoutePackage, ManifestError>
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
    let model_selection = Model_Selection_Field(root, at)?;

    return Ok(ModelRoutePackage {
        package_id: PackageId::New(package_id),
        package_kind,
        package_version,
        protocol_range,
        model_selection,
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

fn Version_Triple_At(
    object: &Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<nomos_package::PackageVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;
    let patch = U16_At(inner, "patch", at, &format!("{full}.patch"))?;

    return Ok(nomos_package::PackageVersion::New(major, minor, patch));
}

fn Contract_Version_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<ContractVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;

    return Ok(ContractVersion::New(major, minor));
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

/// `package_kind`, resolved and checked against the two kinds this crate reads.
fn Package_Kind_Field(object: &Map<String, Value>, at: &str) -> Result<PackageKind, ManifestError>
{
    let label = String_At(object, "package_kind", at, "package_kind")?;
    let parsed: PackageKind = serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownPackageKind { at: at.to_owned(), found: label.clone() };
    })?;

    if parsed != PackageKind::ModelBackendPackage && parsed != PackageKind::AgentExecutorPackage
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

/// `model_selection`, resolved to a [`ModelSelection`].
fn Model_Selection_Field(object: &Map<String, Value>, at: &str) -> Result<ModelSelection, ManifestError>
{
    let raw = Field_At(object, "model_selection", at, "model_selection")?;
    let inner = Object_At(raw, at, "model_selection")?;
    let kind = String_At(inner, "kind", at, "model_selection.kind")?;

    return match kind.as_str()
    {
        "opaque" => Ok(ModelSelection::Opaque),
        "executor_controlled" => Ok(ModelSelection::ExecutorControlled),
        "catalog" => Ok(ModelSelection::Catalog(Catalog_Models_At(inner, at)?)),
        _ => Err(ManifestError::UnknownModelSelectionKind { at: at.to_owned(), found: kind }),
    };
}

/// `model_selection.models`, a non-empty list of raw model identifiers.
fn Catalog_Models_At(object: &Map<String, Value>, at: &str) -> Result<Vec<String>, ManifestError>
{
    let raw = Field_At(object, "models", at, "model_selection.models")?;
    let array = raw.as_array().ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: "model_selection.models".to_owned(),
            expected: "array".to_owned(),
        };
    })?;
    if array.is_empty()
    {
        return Err(ManifestError::EmptyList { at: at.to_owned(), field: "model_selection.models".to_owned() });
    }

    let mut models = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate()
    {
        let full = format!("model_selection.models[{index}]");
        let label = item.as_str().ok_or_else(|| {
            return ManifestError::WrongType { at: at.to_owned(), field: full.clone(), expected: "string".to_owned() };
        })?;

        models.push(label.to_owned());
    }

    return Ok(models);
}
