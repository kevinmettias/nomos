//! Reading an `IntegrationPackage` manifest from JSON, and refusing what it cannot resolve.
//!
//! Hand-rolled rather than built over `nomos_package::Parse_Manifest`: that shared entry
//! point refuses every `PackageKind` other than `LanguagePackage`
//! (`nomos_package::reader::Package_Kind_Field`), and its own `language_versions` and
//! `providers` fields are documented "never empty" -- meaningless for an
//! `IntegrationPackage`, which recognizes no language and registers no provider. This
//! mirrors the established pattern `nomos-rule-package`, `nomos-model-package` and
//! `nomos-tool-package` already use for every `PackageKind` other than `LanguagePackage`:
//! reuse `nomos_package`'s plain shared types (`ProtocolRange`, `PackageVersion`), never its
//! `Parse_Manifest` entry point, and declare a complete [`ManifestError`] of this crate's own.
//!
//! `Is_Absolute` and `Escapes_The_Root` are called from `nomos-materialization` rather than
//! kept here. They moved down with the intent type they judge, because the mechanism that
//! actually opens the file cannot take a declaration's word for where it may write, and two
//! copies of one check is two authorities on one question.
//!
//! Every condition is a refusal and not a skip, with two deliberate exceptions that are
//! defaults rather than skips: an intent that declares no `ownership_class` resolves to
//! [`OwnershipClass::UNDECLARED`] and one that declares no `publication_scope` resolves to
//! [`PublicationScope::UNDECLARED`], because `OD-PACKAGE-004` and `OD-PACKAGE-005` each
//! decide what an undeclared asset means and both fix the direction by the same asymmetry
//! argument. A key that is present with the wrong shape, or a label neither record names, is
//! still refused -- a default covers absence, never a malformed presence.

use crate::integration_package::IntegrationPackage;
use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_materialization::{Escapes_The_Root, Is_Absolute, MaterializationIntent, OwnedRegion, OwnershipClass, PublicationScope};
use nomos_package::{PackageVersion, ProtocolRange};
use serde_json::{Map, Value};
use std::path::Path;

/// The highest manifest schema this build can read. `OD-PACKAGE-009`.
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
    /// `package_kind` is a real [`PackageKind`], but not `IntegrationPackage`.
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
    /// `schema_version` names a manifest shape newer than this build understands.
    UnknownSchema
    {
        at: String,
        understood: u32,
        found: u32,
    },
    /// An intent's `ownership_class` is a label `OD-PACKAGE-004` does not name.
    UnknownOwnershipClass
    {
        at: String,
        field: String,
        found: String,
    },
    /// An intent's `publication_scope` is a label `OD-PACKAGE-005` does not name.
    UnknownPublicationScope
    {
        at: String,
        field: String,
        found: String,
    },
    /// An intent's `target` is anchored somewhere other than the repository root.
    AbsoluteTarget
    {
        at: String,
        field: String,
        target: String,
    },
    /// An intent's `target` climbs above the repository root.
    EscapingTarget
    {
        at: String,
        field: String,
        target: String,
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
            Self::WrongType { at, field, expected } => write!(formatter, "{at}'s `{field}` is not a {expected}."),
            Self::EmptyList { at, field } => write!(formatter, "{at}'s `{field}` names no value."),
            Self::UnknownPackageKind { at, found } =>
            {
                write!(formatter, "{at}'s `package_kind` is `{found}`, which is not a PackageKind this build defines.")
            }
            Self::WrongPackageKind { at, found } =>
            {
                write!(formatter, "{at}'s `package_kind` is `{found}`, not `IntegrationPackage`.")
            }
            Self::MalformedVersion { at, field, cause } =>
            {
                write!(formatter, "{at}'s `{field}` did not resolve to a version: {cause}.")
            }
            Self::InvertedProtocolRange { at } =>
            {
                write!(formatter, "{at}'s `protocol_range.minimum` sorts after `protocol_range.maximum`.")
            }
            Self::UnknownSchema { at, understood, found } =>
            {
                write!(formatter, "{at}'s `schema_version` is {found}, and this build only understands manifest schema {understood}.")
            }
            Self::UnknownOwnershipClass { at, field, found } =>
            {
                write!(formatter, "{at}'s `{field}` is `{found}`, which is not an ownership class OD-PACKAGE-004 names.")
            }
            Self::UnknownPublicationScope { at, field, found } =>
            {
                write!(formatter, "{at}'s `{field}` is `{found}`, which is not a publication scope OD-PACKAGE-005 names.")
            }
            Self::AbsoluteTarget { at, field, target } =>
            {
                write!(formatter, "{at}'s `{field}` is `{target}`, which is absolute rather than repository-relative.")
            }
            Self::EscapingTarget { at, field, target } =>
            {
                write!(formatter, "{at}'s `{field}` is `{target}`, which climbs above the repository root.")
            }
        };
    }
}

/// Reads and resolves a manifest from a file.
///
/// # Errors
///
/// Returns [`ManifestError`] on every malformed input; see the enum for the named cases.
pub fn Read_Manifest(path: &Path) -> Result<IntegrationPackage, ManifestError>
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
pub fn Parse_Manifest(text: &str, at: &str) -> Result<IntegrationPackage, ManifestError>
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
    let intents = Intents_Field(root, at)?;

    return Ok(IntegrationPackage {
        package_id: PackageId::New(package_id),
        package_kind,
        package_version,
        protocol_range,
        intents,
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

    return As_String(value, at, full);
}

/// One key's value as a string when the key is present, and `None` when it is absent --
/// the one shape the two defaulted fields need, where absence and a wrong type are
/// different answers.
fn Optional_String_At(object: &Map<String, Value>, key: &str, at: &str, full: &str) -> Result<Option<String>, ManifestError>
{
    let Some(value) = object.get(key)
    else
    {
        return Ok(None);
    };

    return As_String(value, at, full).map(Some);
}

fn As_String(value: &Value, at: &str, full: &str) -> Result<String, ManifestError>
{
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

/// `package_kind`, resolved and checked against [`PackageKind::IntegrationPackage`].
fn Package_Kind_Field(object: &Map<String, Value>, at: &str) -> Result<PackageKind, ManifestError>
{
    let label = String_At(object, "package_kind", at, "package_kind")?;
    let parsed: PackageKind = serde_json::from_value(Value::String(label.clone())).map_err(|_| {
        return ManifestError::UnknownPackageKind { at: at.to_owned(), found: label.clone() };
    })?;

    if parsed != PackageKind::IntegrationPackage
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

/// `intents`, resolved to at least one [`MaterializationIntent`].
fn Intents_Field(object: &Map<String, Value>, at: &str) -> Result<Vec<MaterializationIntent>, ManifestError>
{
    let raw = Field_At(object, "intents", at, "intents")?;
    let array = raw.as_array().ok_or_else(|| {
        return ManifestError::WrongType { at: at.to_owned(), field: "intents".to_owned(), expected: "array".to_owned() };
    })?;
    if array.is_empty()
    {
        return Err(ManifestError::EmptyList { at: at.to_owned(), field: "intents".to_owned() });
    }

    let mut intents = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate()
    {
        let full = format!("intents[{index}]");

        intents.push(Intent_At(item, at, &full)?);
    }

    return Ok(intents);
}

/// One entry of `intents`: its surface and source as spelled, its target checked to be
/// repository-relative, and its two classifications resolved or defaulted.
fn Intent_At(value: &Value, at: &str, full: &str) -> Result<MaterializationIntent, ManifestError>
{
    let inner = Object_At(value, at, full)?;
    let surface = String_At(inner, "surface", at, &format!("{full}.surface"))?;
    let source = String_At(inner, "source", at, &format!("{full}.source"))?;
    let target = Target_Field(inner, at, full)?;
    let ownership_class = Ownership_Class_Field(inner, at, full)?;
    let publication_scope = Publication_Scope_Field(inner, at, full)?;
    let owned_region = Owned_Region_Field(inner, at, full)?;

    return Ok(MaterializationIntent { surface, source, target, ownership_class, publication_scope, owned_region });
}

/// One intent's `owned_region`: absent for every class but `Composed`, and both markers
/// required when it is present.
///
/// Absence is not defaulted to a region, because there is no safe region to invent -- a
/// guessed marker pair would either find nothing or find the wrong thing. Whether a given
/// class may carry one at all is `nomos-materialization`'s judgment and not this reader's:
/// that is a rule about what a write may do, and this crate performs no write.
fn Owned_Region_Field(object: &Map<String, Value>, at: &str, full: &str) -> Result<Option<OwnedRegion>, ManifestError>
{
    let field = format!("{full}.owned_region");
    let Some(raw) = object.get("owned_region")
    else
    {
        return Ok(None);
    };
    let inner = Object_At(raw, at, &field)?;
    let opening_marker = String_At(inner, "opening_marker", at, &format!("{field}.opening_marker"))?;
    let closing_marker = String_At(inner, "closing_marker", at, &format!("{field}.closing_marker"))?;

    return Ok(Some(OwnedRegion { opening_marker, closing_marker }));
}

/// One intent's `target`, refused if absolute or if it climbs above the repository root.
fn Target_Field(object: &Map<String, Value>, at: &str, full: &str) -> Result<String, ManifestError>
{
    let field = format!("{full}.target");
    let found = String_At(object, "target", at, &field)?;

    if Is_Absolute(&found)
    {
        return Err(ManifestError::AbsoluteTarget { at: at.to_owned(), field, target: found });
    }
    if Escapes_The_Root(&found)
    {
        return Err(ManifestError::EscapingTarget { at: at.to_owned(), field, target: found });
    }

    return Ok(found);
}

/// One intent's `ownership_class`: [`OwnershipClass::UNDECLARED`] when the key is absent,
/// refused when it is present and names none of `OD-PACKAGE-004`'s three classes.
fn Ownership_Class_Field(object: &Map<String, Value>, at: &str, full: &str) -> Result<OwnershipClass, ManifestError>
{
    let field = format!("{full}.ownership_class");
    let Some(label) = Optional_String_At(object, "ownership_class", at, &field)?
    else
    {
        return Ok(OwnershipClass::UNDECLARED);
    };

    return OwnershipClass::Of_Label(&label).ok_or_else(|| {
        return ManifestError::UnknownOwnershipClass { at: at.to_owned(), field, found: label };
    });
}

/// One intent's `publication_scope`: [`PublicationScope::UNDECLARED`] when the key is
/// absent, refused when it is present and names none of `OD-PACKAGE-005`'s three scopes.
fn Publication_Scope_Field(object: &Map<String, Value>, at: &str, full: &str) -> Result<PublicationScope, ManifestError>
{
    let field = format!("{full}.publication_scope");
    let Some(label) = Optional_String_At(object, "publication_scope", at, &field)?
    else
    {
        return Ok(PublicationScope::UNDECLARED);
    };

    return PublicationScope::Of_Label(&label).ok_or_else(|| {
        return ManifestError::UnknownPublicationScope { at: at.to_owned(), field, found: label };
    });
}
