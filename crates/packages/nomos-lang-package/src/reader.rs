//! Reading a `LanguagePackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! This is `OD-PACKAGE-001`'s second step, and it is written in the discipline the
//! record names: `nomos-spec-store`'s registration reader
//! (`crates/spec/nomos-spec-store/src/registration.rs`), where every malformed input is
//! a refusal and never a skip. A missing field, a `package_kind` that is not
//! `LanguagePackage`, a version domain this format cannot parse, a provider this
//! package's crate does not know -- each is a named [`ManifestError`] variant, and none
//! of them is reached by defaulting a field or by taking the first value found and
//! ignoring a conflict.
//!
//! Unlike the registration reader, this one is JSON rather than a hand-rolled line
//! format: there is no build-script constraint here pushing the parser away from a
//! dependency, and `serde_json` is already a workspace dependency. The document is read
//! into a [`serde_json::Value`] first and then walked field by field, rather than
//! deserialized directly into typed structs in one call, so that a missing or malformed
//! field produces the specific [`ManifestError`] variant naming it -- not one generic
//! `serde_json::Error` whose text a caller would have to parse to tell "no such field"
//! from "wrong shape" apart.

use crate::known_providers::Is_Known;
use crate::language_version::RustEdition;
use crate::manifest::LanguagePackage;
use crate::protocol_range::ProtocolRange;
use crate::provider_registration::ProviderRegistration;
use crate::version::PackageVersion;
use nomos_contracts::{ContractVersion, PackageId, PackageKind, ProviderId};
use serde_json::{Map, Value};
use std::path::Path;

/// Why a manifest was refused.
///
/// Every condition here is a refusal and not a skip, the same discipline
/// `nomos_spec_store`'s registration reader states for its own error type: a malformed
/// manifest that were silently repaired or dropped a field would be a `LanguagePackage`
/// that quietly claims less, or something different, than what is on disk.
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
    /// `package_kind` is a real [`PackageKind`], but not `LanguagePackage`.
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
    /// A registered provider is not one this workspace's Rust `LanguagePackage` carries.
    UnresolvedProvider
    {
        at: String,
        provider: String,
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
            Self::MissingField { at, field } =>
            {
                write!(formatter, "{at} carries no `{field}` field.")
            }
            Self::WrongType { at, field, expected } => write!(
                formatter,
                "{at}'s `{field}` is not a {expected}."
            ),
            Self::EmptyList { at, field } =>
            {
                write!(formatter, "{at}'s `{field}` names no value.")
            }
            Self::UnknownPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, which is not a PackageKind this build \
                 defines."
            ),
            Self::WrongPackageKind { at, found } => write!(
                formatter,
                "{at}'s `package_kind` is `{found}`, not `LanguagePackage`."
            ),
            Self::MalformedVersion { at, field, cause } =>
            {
                write!(formatter, "{at}'s `{field}` did not resolve to a version: {cause}.")
            }
            Self::InvertedProtocolRange { at } => write!(
                formatter,
                "{at}'s `protocol_range.minimum` sorts after `protocol_range.maximum`."
            ),
            Self::UnresolvedProvider { at, provider } => write!(
                formatter,
                "{at} registers `{provider}`, which is not a provider this crate knows."
            ),
        };
    }
}

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
/// `at` is a caller-supplied label rather than derived from a path, so a manifest read
/// from an embedded string or a test fixture can still be refused with a meaningful
/// name.
///
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Parse_Manifest(text: &str, at: &str) -> Result<LanguagePackage, ManifestError>
{
    let value: Value = serde_json::from_str(text).map_err(|error| {
        return ManifestError::NotJson { at: at.to_owned(), cause: error.to_string() };
    })?;
    let root = Object_At(&value, at, "<root>")?;

    let package_id = String_At(root, "package_id", at, "package_id")?;
    let package_kind = Package_Kind_Field(root, at)?;
    let package_version = Version_Triple_At(root, "package_version", at, "package_version")?;
    let protocol_range = Protocol_Range_Field(root, at)?;
    let language_versions = Language_Versions_Field(root, at)?;
    let providers = Providers_Field(root, at)?;

    return Ok(LanguagePackage {
        package_id: PackageId::New(package_id),
        package_kind,
        package_version,
        protocol_range,
        language_versions,
        providers,
    });
}

/// A JSON value as an object, or a refusal naming `full` as the wrong shape.
fn Object_At<'a>(value: &'a Value, at: &str, full: &str) -> Result<&'a Map<String, Value>, ManifestError>
{
    return value.as_object().ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: full.to_owned(),
            expected: "object".to_owned(),
        };
    });
}

/// One key of an object, or a refusal naming `full` as missing.
fn Field_At<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<&'a Value, ManifestError>
{
    return object.get(key).ok_or_else(|| {
        return ManifestError::MissingField { at: at.to_owned(), field: full.to_owned() };
    });
}

/// One key's value as a string.
fn String_At(
    object: &Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<String, ManifestError>
{
    let value = Field_At(object, key, at, full)?;

    return value.as_str().map(str::to_owned).ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: full.to_owned(),
            expected: "string".to_owned(),
        };
    });
}

/// One key's value as an unsigned 16-bit integer.
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

/// One key's value as a `{major, minor, patch}` triple.
fn Version_Triple_At(
    object: &Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<PackageVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;
    let patch = U16_At(inner, "patch", at, &format!("{full}.patch"))?;

    return Ok(PackageVersion::New(major, minor, patch));
}

/// One key's value as a `{major, minor}` contract version.
fn Contract_Version_At(
    object: &Map<String, Value>,
    key: &str,
    at: &str,
    full: &str,
) -> Result<ContractVersion, ManifestError>
{
    let raw = Field_At(object, key, at, full)?;
    let inner = Object_At(raw, at, full)?;
    let major = U16_At(inner, "major", at, &format!("{full}.major"))?;
    let minor = U16_At(inner, "minor", at, &format!("{full}.minor"))?;

    return Ok(ContractVersion::New(major, minor));
}

/// `package_kind`, resolved and checked against [`PackageKind::LanguagePackage`].
fn Package_Kind_Field(object: &Map<String, Value>, at: &str) -> Result<PackageKind, ManifestError>
{
    let label = String_At(object, "package_kind", at, "package_kind")?;
    // Reuses `PackageKind`'s own `Deserialize`, rather than a second table of the
    // sixteen labels kept here -- a spelling this reader had to keep in step with
    // `nomos-contracts` by hand is exactly the drift `OD-PACKAGE-002` found and fixed
    // once already.
    let parsed: PackageKind = serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownPackageKind { at: at.to_owned(), found: label.clone() };
    })?;
    if parsed != PackageKind::LanguagePackage
    {
        return Err(ManifestError::WrongPackageKind { at: at.to_owned(), found: parsed });
    }

    return Ok(parsed);
}

/// `protocol_range`, resolved and checked for ordering.
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

/// `language_versions`, resolved to at least one [`RustEdition`].
fn Language_Versions_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<RustEdition>, ManifestError>
{
    let raw = Field_At(object, "language_versions", at, "language_versions")?;
    let array = raw.as_array().ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: "language_versions".to_owned(),
            expected: "array".to_owned(),
        };
    })?;
    if array.is_empty()
    {
        return Err(ManifestError::EmptyList {
            at: at.to_owned(),
            field: "language_versions".to_owned(),
        });
    }

    let mut editions = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate()
    {
        let full = format!("language_versions[{index}]");
        let label = item.as_str().ok_or_else(|| {
            return ManifestError::WrongType {
                at: at.to_owned(),
                field: full.clone(),
                expected: "string".to_owned(),
            };
        })?;
        let edition = RustEdition::Of_Label(label).ok_or_else(|| {
            return ManifestError::MalformedVersion {
                at: at.to_owned(),
                field: full.clone(),
                cause: format!("`{label}` is not a Rust edition this reader knows"),
            };
        })?;

        editions.push(edition);
    }

    return Ok(editions);
}

/// `providers`, resolved to at least one [`ProviderRegistration`], each naming a known
/// [`nomos_contracts::ProviderId`].
fn Providers_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<ProviderRegistration>, ManifestError>
{
    let raw = Field_At(object, "providers", at, "providers")?;
    let array = raw.as_array().ok_or_else(|| {
        return ManifestError::WrongType {
            at: at.to_owned(),
            field: "providers".to_owned(),
            expected: "array".to_owned(),
        };
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
            return Err(ManifestError::UnresolvedProvider {
                at: at.to_owned(),
                provider: provider_id,
            });
        }
        let tool_version =
            Version_Triple_At(inner, "tool_version", at, &format!("{full}.tool_version"))?;

        providers.push(ProviderRegistration {
            provider: ProviderId::New(provider_id),
            tool_version,
        });
    }

    return Ok(providers);
}
