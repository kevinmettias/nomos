//! What the workspace is, in bytes somebody else can read.

use crate::Member;
use crate::BuildVariant;
use nomos_contracts::{ConfigurationId, Digest128, SnapshotId};
use nomos_model::Content_Digest;
use nomos_store::StoreError;
use std::collections::BTreeMap;

// The inverse of `Encode`: every field line, the schema gate, and the hex a digest is
// spelled in. A sibling module rather than more of this file, because a file that both
// writes a state down and reads it back is two responsibilities that only ever share the
// schema constant.
#[path = "workspace_snapshot/decoding.rs"]
mod decoding;

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
    pub fn Length(&self) -> usize
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
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Malformed`] if `bytes` is not UTF-8, its schema line does not
    /// match, or a later line does not parse.
    pub fn Decode(bytes: &[u8]) -> Result<Self, StoreError>
    {
        return decoding::Decode_Snapshot(bytes);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The byte [`Configuration`]'s digest is filled with, distinct from
    /// [`OTHER_CONFIGURATION_BYTE`] so two configurations in the same test are never
    /// mistaken for one.
    const CONFIGURATION_BYTE: u8 = 0x7c;

    /// The byte a second, distinguishable configuration is filled with.
    const OTHER_CONFIGURATION_BYTE: u8 = 0x7d;

    /// How many paths the count below writes before it counts them: `a.rs` and `b.rs`, one
    /// `Put` each.
    const PATHS_PUT: usize = 2;

    #[test]
    fn Test_Encode_Should_Produce_Bytes_That_Round_Trip_Byte_For_Byte()
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
    fn Test_Put_Should_Make_The_Encoding_Independent_Of_Insertion_Order()
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

        for altered in Altered_Snapshots()
        {
            assert_ne!(base.Id(), altered.Id(), "{altered:?} must not share the base's identity");
        }
    }

    /// The base snapshot's members, taken under a different variant, and again under a
    /// different configuration. Two named alterations rather than one, because the identity
    /// must move for either change on its own.
    fn Altered_Snapshots() -> Vec<WorkspaceSnapshot>
    {
        let variant = BuildVariant::New("x86_64-unknown-linux-gnu", "release", "1.85", ["analysis"]);
        let mut other_variant = WorkspaceSnapshot::Of(variant, Configuration());
        let mut other_configuration = WorkspaceSnapshot::Of(
            Variant(),
            ConfigurationId::From_Digest(Digest128::From_Bytes([OTHER_CONFIGURATION_BYTE; Digest128::BYTE_LENGTH])),
        );
        for snapshot in [&mut other_variant, &mut other_configuration]
        {
            snapshot.Put("src/lib.rs".to_owned(), Content_Digest(b"pub fn a() {}"));
            snapshot.Put("src/main.rs".to_owned(), Content_Digest(b"fn main() {}"));
        }

        return vec![other_variant, other_configuration];
    }

    /// A decoder that defaulted would produce a snapshot describing a build nobody
    /// configured, and every fact keyed on it would be filed under a variant that does not
    /// exist.
    #[test]
    fn Test_Decode_Should_Refuse_Bytes_That_Are_Not_A_Valid_Snapshot()
    {
        for bytes in Malformed_Snapshot_Bytes()
        {
            assert!(
                WorkspaceSnapshot::Decode(bytes).is_err(),
                "{:?} is not a snapshot",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    /// Every way a byte string fails to be a snapshot: not UTF-8 in spirit, the wrong
    /// schema, a field that does not parse, a field this build does not know, and a
    /// schema with members but no variant — the one that would have defaulted.
    fn Malformed_Snapshot_Bytes() -> Vec<&'static [u8]>
    {
        return vec![
            &b"not a snapshot"[..],
            b"nomos.workspace.snapshot.v0\n",
            b"nomos.workspace.snapshot.v1\nmember\ta.rs\tnot-a-digest\n",
            b"nomos.workspace.snapshot.v1\nunknown\tfield\n",
            b"nomos.workspace.snapshot.v1\nmember\ta.rs\t0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a\n",
        ];
    }

    /// The positive control for the test above: a decoder that refused everything would
    /// pass it while nothing could ever be read back.
    #[test]
    fn Test_Is_Empty_Should_Be_True_For_A_Snapshot_With_Nothing_Decoded()
    {
        let empty = WorkspaceSnapshot::Of(Variant(), Configuration());

        let decoded = WorkspaceSnapshot::Decode(&empty.Encode()).expect("an empty state is a state");

        assert!(decoded.Is_Empty());
        assert_eq!(decoded.Variant(), &Variant());
    }

    #[test]
    fn Test_Of_Should_Start_Completely_Blank()
    {
        let snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());

        assert!(snapshot.Is_Empty());
        assert_eq!(snapshot.Length(), 0);
    }

    #[test]
    fn Test_Take_Should_Remove_A_Path_And_Return_What_It_Held()
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        let digest = Content_Digest(b"pub fn a() {}");
        snapshot.Put("src/a.rs".to_owned(), digest);

        assert_eq!(snapshot.Take("src/a.rs"), Some(digest));
        assert_eq!(snapshot.Take("src/a.rs"), None, "a second take finds nothing left");
    }

    #[test]
    fn Test_Content_Of_Should_Find_What_Was_Written_At_A_Path()
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        let digest = Content_Digest(b"pub fn a() {}");
        snapshot.Put("src/a.rs".to_owned(), digest);

        assert_eq!(snapshot.Content_Of("src/a.rs"), Some(digest));
        assert_eq!(snapshot.Content_Of("src/missing.rs"), None);
    }

    #[test]
    fn Test_Length_Should_Count_How_Many_Paths_Are_Held()
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        assert_eq!(snapshot.Length(), 0);

        snapshot.Put("a.rs".to_owned(), Content_Digest(b"one"));
        snapshot.Put("b.rs".to_owned(), Content_Digest(b"two"));

        assert_eq!(snapshot.Length(), PATHS_PUT);
    }

    #[test]
    fn Test_Variant_Should_Report_The_Build_It_Was_Taken_Under()
    {
        let snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());

        assert_eq!(snapshot.Variant(), &Variant());
    }

    #[test]
    fn Test_Id_Should_Change_When_A_Path_Is_Added()
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        let before = snapshot.Id();

        snapshot.Put("a.rs".to_owned(), Content_Digest(b"content"));

        assert_ne!(snapshot.Id(), before);
    }

    fn Variant() -> BuildVariant
    {
        return BuildVariant::New("x86_64-unknown-linux-gnu", "dev", "1.85", ["analysis"]);
    }

    fn Configuration() -> ConfigurationId
    {
        return ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_BYTE; Digest128::BYTE_LENGTH]));
    }

    fn Populated() -> WorkspaceSnapshot
    {
        let mut snapshot = WorkspaceSnapshot::Of(Variant(), Configuration());
        snapshot.Put("src/lib.rs".to_owned(), Content_Digest(b"pub fn a() {}"));
        snapshot.Put("src/main.rs".to_owned(), Content_Digest(b"fn main() {}"));

        return snapshot;
    }
}
