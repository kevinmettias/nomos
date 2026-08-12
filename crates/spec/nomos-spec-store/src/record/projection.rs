//! A record read back out as markdown, with both content addresses.

/// A record read back out as markdown, with both content addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordProjection
{
    pub node_id: String,
    pub path: String,
    pub revision: String,
    /// The markdown, rendered from the store's rows.
    pub markdown: String,
    /// What the ingested bytes hash to.
    pub source_hash: String,
    /// What this projection hashes to.
    pub projected_hash: String,
}

impl RecordProjection
{
    /// Whether the projection is the ingested bytes.
    ///
    /// Reported rather than asserted here. A document the store cannot reproduce is a real
    /// answer — a v14 record carrying a byte order mark is one — and a reader who asked for
    /// markdown should get it along with the fact, not an error instead of it.
    #[must_use]
    pub fn Matches_Source(&self) -> bool
    {
        return self.source_hash == self.projected_hash;
    }
}
