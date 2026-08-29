//! What went into the store for one record.

/// What went into the store for one record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordWrite
{
    pub node_uid: i64,
    pub document_uid: i64,
    pub blocks: u32,
    pub headings: u32,
    pub relations: u32,
}
