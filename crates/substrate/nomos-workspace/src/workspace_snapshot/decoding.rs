//! Reading a workspace state back out of its bytes.
//!
//! Split out of `workspace_snapshot.rs` by responsibility: that file declares what a state
//! *is* and how it is written down, and this one is the whole of the inverse — every field
//! line, the schema gate, and the hex a digest is spelled in. Nothing here is public; the
//! only door is [`Decode_Snapshot`], which the type's own `Decode` calls.

use crate::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128};
use nomos_store::StoreError;
use std::collections::BTreeMap;

use super::WorkspaceSnapshot;

/// The body of [`WorkspaceSnapshot::Decode`], which is where the reading is written down.
///
/// One line at a time into a partial [`Read`], which is completed only once the payload has
/// run out — so a payload that stops before stating its variant and its configuration is
/// refused rather than completed with a guess.
pub(super) fn Decode_Snapshot(bytes: &[u8]) -> Result<WorkspaceSnapshot, StoreError>
{
    let text = core::str::from_utf8(bytes)
        .map_err(|error| return StoreError::Malformed(error.to_string()))?;
    let mut lines = text.lines();
    Expect_Schema(lines.next().unwrap_or_default())?;

    let mut read = Read {
        variant: None,
        configuration: None,
        members: BTreeMap::new(),
    };
    for line in lines
    {
        Decode_Field(line, &mut read)?;
    }

    return read.Complete();
}

/// A snapshot part-way through being decoded.
///
/// The variant and the configuration are optional here and not on [`WorkspaceSnapshot`],
/// because they arrive as fields in whatever order the payload wrote them and are only
/// required to be present once the payload has run out.
struct Read
{
    variant: Option<BuildVariant>,
    configuration: Option<ConfigurationId>,
    members: BTreeMap<String, Digest128>,
}

impl Read
{
    /// The snapshot a completed read describes, if it describes one.
    ///
    /// The variant and the configuration are required, absent rather than defaulted. A
    /// snapshot decoded with a default variant would claim to describe a build nobody
    /// configured, and every fact keyed on it would be filed under a variant that does not
    /// exist.
    fn Complete(self) -> Result<WorkspaceSnapshot, StoreError>
    {
        let (Some(variant), Some(configuration)) = (self.variant, self.configuration)
        else
        {
            return Err(StoreError::Malformed(
                "a snapshot states its variant and its configuration".to_owned(),
            ));
        };

        return Ok(WorkspaceSnapshot {
            variant,
            configuration,
            members: self.members,
        });
    }
}

/// The schema line, which has to be this schema.
///
/// Refused rather than read past, because bytes filed under a different schema decoded as
/// this one would produce a snapshot describing a tree nobody measured.
fn Expect_Schema(schema: &str) -> Result<(), StoreError>
{
    use super::SNAPSHOT_SCHEMA;

    if schema == SNAPSHOT_SCHEMA
    {
        return Ok(());
    }

    return Err(StoreError::Malformed(format!(
        "`{schema}` is not {SNAPSHOT_SCHEMA}"
    )));
}

/// One field line of a snapshot payload.
///
/// An unrecognised field is refused rather than skipped: a field this build does not know
/// is most likely a newer schema, and reading past it would decode the payload down to the
/// part that has not changed.
fn Decode_Field(line: &str, read: &mut Read) -> Result<(), StoreError>
{
    let Some((field, rest)) = line.split_once('\t')
    else
    {
        return Err(StoreError::Malformed(format!("`{line}` is not a field")));
    };

    match field
    {
        "variant" => read.variant = Some(Decode_Variant(rest)?),
        "configuration" => read.configuration = Some(Decode_Configuration(rest)?),
        "member" => Decode_Member(rest, &mut read.members)?,
        other => return Err(Unknown_Field(other)),
    }

    return Ok(());
}

fn Decode_Variant(rendered: &str) -> Result<BuildVariant, StoreError>
{
    let fields: Vec<&str> = rendered.split('\t').collect();

    let [target, profile, toolchain, features] = fields.as_slice()
    else
    {
        return Err(StoreError::Malformed(format!(
            "a variant has four fields and `{rendered}` has {}",
            fields.len()
        )));
    };

    return Ok(BuildVariant::New(
        *target,
        *profile,
        *toolchain,
        features.split(',').filter(|feature| return !feature.is_empty()),
    ));
}

/// The configuration a snapshot was taken under.
fn Decode_Configuration(rest: &str) -> Result<ConfigurationId, StoreError>
{
    let digest = Decode_Digest(rest)?;

    return Ok(ConfigurationId::From_Digest(digest));
}

/// One member: a path and the digest of what stood at it.
fn Decode_Member(rest: &str, members: &mut BTreeMap<String, Digest128>) -> Result<(), StoreError>
{
    let Some((path, content)) = rest.split_once('\t')
    else
    {
        return Err(StoreError::Malformed(format!(
            "`{rest}` is not a path and a digest"
        )));
    };
    members.insert(path.to_owned(), Decode_Digest(content)?);

    return Ok(());
}

/// A field this build does not know.
fn Unknown_Field(field: &str) -> StoreError
{
    return StoreError::Malformed(format!("`{field}` is not a snapshot field"));
}

/// The base a digest is spelled in.
const HEXADECIMAL: u32 = 16;

/// How many characters of that spelling one byte occupies.
const PER_BYTE: usize = 2;

fn Decode_Digest(hex: &str) -> Result<Digest128, StoreError>
{
    if hex.len() != Digest128::HEX_LENGTH
    {
        return Err(StoreError::Malformed(format!(
            "`{hex}` is not {} hex characters",
            Digest128::HEX_LENGTH
        )));
    }

    let mut bytes = [0_u8; Digest128::BYTE_LENGTH];

    for (index, slot) in bytes.iter_mut().enumerate()
    {
        let at = index.saturating_mul(PER_BYTE);
        let pair = hex
            .get(at..at.saturating_add(PER_BYTE))
            .ok_or_else(|| return StoreError::Malformed(format!("`{hex}` ended early")))?;
        *slot = u8::from_str_radix(pair, HEXADECIMAL)
            .map_err(|cause| return StoreError::Malformed(format!("`{pair}` is not hex: {cause}")))?;
    }

    return Ok(Digest128::From_Bytes(bytes));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The schema this build's predecessor wrote. The two payloads differ, so reading these
    /// bytes as a v1 snapshot would produce a state nobody ever encoded.
    const PREVIOUS_SCHEMA_LINE: &[u8] = b"nomos.workspace.snapshot.v0\n";

    #[test]
    fn Test_Decode_Snapshot_Should_Refuse_An_Unknown_Schema()
    {
        let refused = Decode_Snapshot(PREVIOUS_SCHEMA_LINE);

        assert!(
            matches!(refused, Err(StoreError::Malformed(_))),
            "an unknown schema is refused at the gate, not read past field by field"
        );
    }
}
