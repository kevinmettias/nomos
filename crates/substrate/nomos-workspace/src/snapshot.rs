//! What the workspace is, in bytes somebody else can read.

use crate::Member;
use crate::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128, SnapshotId};
use nomos_model::Content_Digest;
use nomos_store::StoreError;
use std::collections::BTreeMap;

pub const SNAPSHOT_SCHEMA: &str = "nomos.workspace.snapshot.v1";

/// A workspace state, addressable by what it contains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSnapshot
{
    variant: BuildVariant,
    configuration: ConfigurationId,
    /// Members by path. A map, so ingestion order cannot reach the encoding, and an
    /// ordered one, so iteration is the same on every machine.
    members: BTreeMap<String, Digest128>,
}

impl WorkspaceSnapshot
{
    #[must_use]
    pub fn Of(variant: BuildVariant, configuration: ConfigurationId) -> Self
    {
        return Self {
            variant,
            configuration,
            members: BTreeMap::new(),
        };
    }

    pub(crate) fn Put(&mut self, path: String, content: Digest128) -> Option<Digest128>
    {
        return self.members.insert(path, content);
    }

    pub(crate) fn Take(&mut self, path: &str) -> Option<Digest128>
    {
        return self.members.remove(path);
    }

    #[must_use]
    pub fn Content_Of(&self, path: &str) -> Option<Digest128>
    {
        return self.members.get(path).copied();
    }

    /// Every member, ordered by path.
    #[must_use]
    pub fn Members(&self) -> Vec<Member>
    {
        return self
            .members
            .iter()
            .map(|(path, content)| {
                return Member {
                    path: path.clone(),
                    content: *content,
                };
            })
            .collect();
    }

    #[must_use]
    pub fn Len(&self) -> usize
    {
        return self.members.len();
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.members.is_empty();
    }

    #[must_use]
    pub const fn Variant(&self) -> &BuildVariant
    {
        return &self.variant;
    }

    #[must_use]
    pub const fn Configuration(&self) -> ConfigurationId
    {
        return self.configuration;
    }

    /// The canonical bytes of this state.
    ///
    /// Hand-written and line-oriented, for the reasons the rest of this workspace writes
    /// its payloads that way: a derived encoder can change its field ordering in a point
    /// release and silently re-address every snapshot ever taken. It is also readable,
    /// which is what lets `tests/portable.rs` assert that a corpus root does not appear in
    /// it by looking.
    ///
    /// The generation is **not** here. These bytes are the workspace's state, and the
    /// generation is when it was reached — including it would make a file edited and
    /// edited back into a third state to analyze.
    #[must_use]
    pub fn Encode(&self) -> Vec<u8>
    {
        let mut encoded = String::new();

        encoded.push_str(SNAPSHOT_SCHEMA);
        encoded.push('\n');
        encoded.push_str("variant\t");
        encoded.push_str(&self.variant.Rendered());
        encoded.push('\n');
        encoded.push_str("configuration\t");
        encoded.push_str(&self.configuration.Digest().to_string());
        encoded.push('\n');

        for (path, content) in &self.members
        {
            encoded.push_str("member\t");
            encoded.push_str(path);
            encoded.push('\t');
            encoded.push_str(&content.to_string());
            encoded.push('\n');
        }

        return encoded.into_bytes();
    }

    /// This state's identity.
    #[must_use]
    pub fn Id(&self) -> SnapshotId
    {
        return SnapshotId::From_Digest(Content_Digest(&self.Encode()));
    }

    /// Reads a snapshot back from its bytes.
    ///
    /// The only input is the bytes. Nothing here consults a filesystem, a working
    /// directory or an environment variable, which is what makes "interpretable by a
    /// process with no access to that tree" a property of the type rather than a hope
    /// about the caller.
    pub fn Decode(bytes: &[u8]) -> Result<Self, StoreError>
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
}

/// A snapshot part-way through being decoded.
///
/// The variant and the configuration are optional here and not on [`Snapshot`], because
/// they arrive as fields in whatever order the payload wrote them and are only required to
/// be present once the payload has run out.
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

/// The configuration a snapshot was taken under.
fn Decode_Configuration(rest: &str) -> Result<ConfigurationId, StoreError>
{
    let digest = Decode_Digest(rest)?;

    return Ok(ConfigurationId::From_Digest(digest));
}

/// A field this build does not know.
fn Unknown_Field(field: &str) -> StoreError
{
    return StoreError::Malformed(format!("`{field}` is not a snapshot field"));
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

    fn Variant() -> BuildVariant
    {
        return BuildVariant::New("x86_64-unknown-linux-gnu", "dev", "1.85", ["analysis"]);
    }

    fn Configuration() -> ConfigurationId
    {
        return ConfigurationId::From_Digest(Digest128::From_Bytes([0x7c; 16]));
    }

    fn Populated() -> WorkspaceSnapshot
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        snapshot.Put("src/lib.rs".to_owned(), Content_Digest(b"pub fn a() {}"));
        snapshot.Put("src/main.rs".to_owned(), Content_Digest(b"fn main() {}"));

        return snapshot;
    }

    #[test]
    fn Test_A_Snapshot_Should_Survive_A_Round_Trip()
    {
        let snapshot = Populated();

        let decoded = WorkspaceSnapshot::Decode(&snapshot.Encode()).expect("it round-trips");

        assert_eq!(decoded, snapshot);
        assert_eq!(decoded.Id(), snapshot.Id());
        assert_eq!(decoded.Encode(), snapshot.Encode(), "byte-identical");
    }

    /// The property that makes the snapshot a state rather than a log entry: insertion
    /// order cannot reach the bytes.
    #[test]
    fn Test_Insertion_Order_Should_Not_Reach_The_Encoding()
    {
        let mut forwards = WorkspaceSnapshot::Of(Variant(), Configuration());
        let mut backwards = WorkspaceSnapshot::Of(Variant(), Configuration());

        let members = [
            ("z.rs", "last"),
            ("a.rs", "first"),
            ("m.rs", "middle"),
            ("b.rs", "second"),
        ];

        for (path, content) in members
        {
            forwards.Put((*path).to_owned(), Content_Digest(content.as_bytes()));
        }
        for (path, content) in members.iter().rev()
        {
            backwards.Put((*path).to_owned(), Content_Digest(content.as_bytes()));
        }

        assert_eq!(forwards.Encode(), backwards.Encode());
        assert_eq!(forwards.Id(), backwards.Id());
    }

    /// The negative control. If the encoding ignored members, the test above would pass
    /// over two snapshots that both said nothing.
    #[test]
    fn Test_Different_Members_Should_Be_Different_Snapshots()
    {
        let one = Populated();
        let mut other = Populated();
        other.Put("src/lib.rs".to_owned(), Content_Digest(b"pub fn b() {}"));

        assert_ne!(one.Id(), other.Id());

        let mut fewer = Populated();
        fewer.Take("src/main.rs");
        assert_ne!(one.Id(), fewer.Id());
    }

    /// The variant and the configuration are part of the state, not labels on it. One tree
    /// built two ways is two things to measure.
    #[test]
    fn Test_The_Variant_And_Configuration_Should_Reach_The_Identity()
    {
        let base = Populated();

        let variant = BuildVariant::New("x86_64-unknown-linux-gnu", "release", "1.85", ["analysis"]);
        let mut other_variant = WorkspaceSnapshot::Of(variant, Configuration());
        let mut other_configuration = WorkspaceSnapshot::Of(
            Variant(),
            ConfigurationId::From_Digest(Digest128::From_Bytes([0x7d; 16])),
        );
        for snapshot in [&mut other_variant, &mut other_configuration]
        {
            snapshot.Put("src/lib.rs".to_owned(), Content_Digest(b"pub fn a() {}"));
            snapshot.Put("src/main.rs".to_owned(), Content_Digest(b"fn main() {}"));
        }

        assert_ne!(base.Id(), other_variant.Id());
        assert_ne!(base.Id(), other_configuration.Id());
    }

    /// A decoder that defaulted would produce a snapshot describing a build nobody
    /// configured, and every fact keyed on it would be filed under a variant that does not
    /// exist.
    #[test]
    fn Test_Bytes_That_Are_Not_A_Snapshot_Should_Be_Refused()
    {
        for bytes in [
            &b"not a snapshot"[..],
            b"nomos.workspace.snapshot.v0\n",
            b"nomos.workspace.snapshot.v1\nmember\ta.rs\tnot-a-digest\n",
            b"nomos.workspace.snapshot.v1\nunknown\tfield\n",
            // Schema and members, and no variant. The one that would have defaulted.
            b"nomos.workspace.snapshot.v1\nmember\ta.rs\t0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a\n",
        ]
        {
            assert!(
                WorkspaceSnapshot::Decode(bytes).is_err(),
                "{:?} is not a snapshot",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    /// The positive control for the test above: a decoder that refused everything would
    /// pass it while nothing could ever be read back.
    #[test]
    fn Test_A_Snapshot_With_No_Members_Should_Still_Decode()
    {
        let empty = WorkspaceSnapshot::Of(Variant(), Configuration());

        let decoded = WorkspaceSnapshot::Decode(&empty.Encode()).expect("an empty state is a state");

        assert!(decoded.Is_Empty());
        assert_eq!(decoded.Variant(), &Variant());
    }
}
