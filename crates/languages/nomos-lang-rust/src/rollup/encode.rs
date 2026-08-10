//! Writing a module index out.

use super::{ModuleIndex, IndexEntry};

/// Fields in a `module` record, counting the tag.
pub(super) const MODULE_FIELDS: usize = 2;

/// Fields in a `member` record, counting the tag.
pub(super) const MEMBER_FIELDS: usize = 3;

/// Fields in an `item` record, counting the tag.
pub(super) const ITEM_FIELDS: usize = 6;

/// The canonical byte encoding of an index.
///
/// Line-oriented text by the same rules and for the same reasons as the syntax schema's:
/// tab-separated fields, `\n` terminators and never `\r\n`, written by hand in one place so
/// no dependency's field ordering can silently re-address every rollup in the store.
///
/// ```text
/// payload := module member* item*
/// module  := "module" TAB subject LF
/// member  := "member" TAB subject TAB outcome LF
/// item    := "item" TAB subject TAB ordinal TAB kind TAB visibility TAB qualified-name LF
/// outcome := "read" | "approximate" | "unreachable"
/// subject := 32 lowercase hexadecimal characters
/// ```
///
/// The `module` record is first and appears exactly once. It is what makes an index over a
/// module with no members a payload rather than the empty byte string, which the reader
/// refuses — a module nothing was read for and a module that declares nothing are two
/// answers and must not share an encoding.
///
/// **A record is exactly the fields the grammar gives it**, and one carrying more is refused
/// rather than read down to the fields this build knows. That is the same rule as the one
/// for an unrecognised record tag and it is refused for the same reason: a longer record is
/// most likely a newer schema, so the field being dropped is the one carrying what changed.
/// [`Expect_Fields`] is where it is enforced.
///
/// Nothing is escaped. `kind`, `visibility` and `qualified-name` arrive from the syntax
/// schema's unescaped fields, which are tab-free and newline-free by that schema's own
/// grammar, so an escape here would be a second encoding of bytes that already passed
/// through one.
#[must_use]
pub fn Encode_Index(index: &ModuleIndex) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("module\t");
    encoded.push_str(&index.module.Digest().to_string());
    encoded.push('\n');

    for member in &index.members
    {
        encoded.push_str("member\t");
        encoded.push_str(&member.subject.Digest().to_string());
        encoded.push('\t');
        encoded.push_str(member.outcome.Label());
        encoded.push('\n');
    }

    for item in &index.items
    {
        Encode_Entry(&mut encoded, item);
    }

    return encoded.into_bytes();
}

/// One item record: the member that declared it, then the syntax schema's own fields.
pub(super) fn Encode_Entry(encoded: &mut String, item: &IndexEntry)
{
    encoded.push_str("item\t");
    encoded.push_str(&item.member.Digest().to_string());
    encoded.push('\t');
    encoded.push_str(&item.ordinal.to_string());
    encoded.push('\t');
    encoded.push_str(&item.kind);
    encoded.push('\t');
    encoded.push_str(&item.visibility);
    encoded.push('\t');
    encoded.push_str(&item.qualified_name);
    encoded.push('\n');
}
