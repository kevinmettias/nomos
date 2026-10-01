//! Reading a `ToolPackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! Hand-rolled rather than built over `nomos_package::Parse_Manifest`: that shared entry
//! point refuses every `PackageKind` other than `LanguagePackage`
//! (`nomos_package::reader::Package_Kind_Field`), and its own `PackageManifest::
//! language_versions` is documented "never empty" -- meaningless for a `ToolProvider`
//! manifest, which has no language versions at all. This mirrors the established pattern
//! `nomos-rule-package` and `nomos-model-package` already use for every `PackageKind`
//! other than `LanguagePackage`: reuse `nomos_package`'s plain shared types
//! (`ProtocolRange`, `PackageVersion`), never its `Parse_Manifest` entry point, and declare
//! a complete [`ManifestError`] of this crate's own -- the same variant shapes as
//! `nomos_package::ManifestError`, but its own type. A missing field, a `package_kind`
//! this crate does not read, a version domain that will not parse, a provider the
//! allowlist does not name, a `family` label `OD-CAPABILITY-013` does not close the
//! vocabulary at -- each is a named [`ManifestError`] variant, never a default or a skip.

use crate::family::Family;
use crate::known_providers::Is_Known;
use crate::tool_package::ToolPackage;
use crate::tool_provider_registration::ToolProviderRegistration;
use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
use nomos_package::{PackageVersion, ProtocolRange};
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
    /// A list that must name at least one value names none.
    EmptyList
    {
        at: String,
        field: String,
    },
    /// `package_kind` names something [`PackageKind`] does not define at all.
    UnknownPackageKind
    {
        at: String,
        found: String,
    },
    /// `package_kind` is a real [`PackageKind`], but not `ToolProvider`.
    WrongPackageKind
    {
        at: String,
        found: PackageKind,
    },
    /// A version domain's value could not be resolved to the type this format uses
    /// there. Also carries a `providers[n].family` label matching none of
    /// `OD-CAPABILITY-013`'s twelve closed FAMILY names, the same shape
    /// `nomos-lang-rust-package`'s reader already reuses for a `language_versions` label
    /// its own `RustEdition::Of_Label` cannot resolve.
    MalformedVersion
    {
        at: String,
        field: String,
        cause: String,
    },
    /// A registered provider is not one this crate's `KNOWN_PROVIDERS` names.
    UnresolvedProvider
    {
        at: String,
        provider: String,
    },
    /// `protocol_range.minimum` sorts after `protocol_range.maximum`.
    InvertedProtocolRange
    {
        at: String,
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
            Self::EmptyList { at, field } => write!(formatter, "{at}'s `{field}` names no value."),
            Self::UnknownPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, which is not a PackageKind this build defines."
            ),
            Self::WrongPackageKind { at, found } =>
            {
                write!(formatter, "{at}'s `package_kind` is `{found}`, not `ToolProvider`.")
            }
            Self::MalformedVersion { at, field, cause } =>
            {
                write!(formatter, "{at}'s `{field}` did not resolve to a version: {cause}.")
            }
            Self::UnresolvedProvider { at, provider } => write!(
                formatter,
                "{at} registers `{provider}`, which is not a provider this crate knows."
            ),
            Self::InvertedProtocolRange { at } => write!(
                formatter,
                "{at}'s `protocol_range.minimum` sorts after `protocol_range.maximum`."
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
pub fn Read_Manifest(path: &Path) -> Result<ToolPackage, ManifestError>
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
pub fn Parse_Manifest(text: &str, at: &str) -> Result<ToolPackage, ManifestError>
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
    let providers = Providers_Field(root, at)?;

    return Ok(ToolPackage {
        package_id: PackageId::New(package_id),
        package_kind,
        package_version,
        protocol_range,
        providers,
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

fn Version_Triple_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<PackageVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;
    let patch = U16_At(inner, "patch", at, &format!("{full}.patch"))?;

    return Ok(PackageVersion::New(major, minor, patch));
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

/// `package_kind`, resolved and checked against [`PackageKind::ToolProvider`].
fn Package_Kind_Field(object: &Map<String, Value>, at: &str) -> Result<PackageKind, ManifestError>
{
    let label = String_At(object, "package_kind", at, "package_kind")?;
    let parsed: PackageKind = serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownPackageKind { at: at.to_owned(), found: label.clone() };
    })?;

    if parsed != PackageKind::ToolProvider
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

/// `providers`, resolved to at least one [`ToolProviderRegistration`], each naming a
/// provider [`crate::known_providers::KNOWN_PROVIDERS`] carries and a `family` one of
/// `OD-CAPABILITY-013`'s twelve closed FAMILY names.
fn Providers_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<ToolProviderRegistration>, ManifestError>
{
    let raw = Field_At(object, "providers", at, "providers")?;
    let array = raw.as_array().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: "providers".to_owned(), expected: "array".to_owned() };
    })?;
    if array.is_empty()
    {
        return Err(ManifestError::EmptyList { at: at.to_owned(), field: "providers".to_owned() });
    }

    let mut providers = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate()
    {
        let full = format!("providers[{index}]");
        let inner = Object_At(item, at, &full)?;
        let provider_id = String_At(inner, "provider_id", at, &format!("{full}.provider_id"))?;
        if !Is_Known(&provider_id)
        {
            return Err(ManifestError::UnresolvedProvider { at: at.to_owned(), provider: provider_id });
        }
        let tool_version = Version_Triple_At(inner, "tool_version", at, &format!("{full}.tool_version"))?;
        let family = Family_At(inner, at, &full)?;

        providers.push(ToolProviderRegistration { provider: ProviderId::New(provider_id), tool_version, family });
    }

    return Ok(providers);
}

/// One entry's `family` label, resolved against [`Family::Of_Label`] the same way
/// `nomos-lang-rust-package`'s reader resolves a `language_versions` label against
/// `RustEdition::Of_Label` -- the identical [`ManifestError::MalformedVersion`] shape for
/// a label matching none of the closed vocabulary.
fn Family_At(object: &Map<String, Value>, at: &str, full: &str) -> Result<Family, ManifestError>
{
    let field = format!("{full}.family");
    let label = String_At(object, "family", at, &field)?;

    return Family::Of_Label(&label).ok_or_else(|| {
        return ManifestError::MalformedVersion {
            at: at.to_owned(),
            field,
            cause: format!("`{label}` is not one of the twelve FAMILY names OD-CAPABILITY-013 closes the vocabulary at"),
        };
    });
}
