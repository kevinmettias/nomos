//! A record read back out as markdown, with both content addresses.

/// A record read back out as markdown, with both content addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection
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

impl Projection
{
    /// Whether the projection is the ingested bytes.
    ///
    /// Reported rather than asserted here. A document the store cannot reproduce is a real
    /// answer — a v14 record carrying a byte order mark is one — and a reader who asked for
    /// markdown should get it along with the fact, not an error instead of it.
    #[must_use]
    pub fn Is_Matching_Source(&self) -> bool
    {
        return self.source_hash == self.projected_hash;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[derive(Clone, Copy)]
    struct SourceHash<'a>(&'a str);
    #[derive(Clone, Copy)]
    struct ProjectedHash<'a>(&'a str);

    fn A_Projection(source_hash: SourceHash<'_>, projected_hash: ProjectedHash<'_>) -> Projection
    {
        return Projection {
            node_id: "D-1".to_owned(),
            path: "records/D-1.md".to_owned(),
            revision: "v1".to_owned(),
            markdown: "# D-1\n".to_owned(),
            source_hash: source_hash.0.to_owned(),
            projected_hash: projected_hash.0.to_owned(),
        };
    }

    #[test]
    fn Test_Is_Matching_Source_Should_Compare_The_Two_Content_Addresses()
    {
        assert!(A_Projection(SourceHash("sha256:same"), ProjectedHash("sha256:same")).Is_Matching_Source());
        assert!(!A_Projection(SourceHash("sha256:one"), ProjectedHash("sha256:other")).Is_Matching_Source());
    }
}
