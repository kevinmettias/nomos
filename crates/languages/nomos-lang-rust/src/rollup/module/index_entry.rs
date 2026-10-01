//! One declaration the index carries.

use nomos_contracts::SubjectId;
/// One declared item, attributed to the member that declared it.
///
/// `qualified_name` is the syntax schema's, unchanged: a name qualified by nesting within
/// its own file, and not a resolved path. What this adds is `member`, without which two
/// files declaring the same name are one record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexEntry
{
    pub member: SubjectId,
    pub ordinal: u32,
    pub kind: String,
    pub visibility: String,
    pub qualified_name: String,
}
