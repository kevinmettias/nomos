//! What ingesting the catalog found.

#[derive(Debug, Default)]
pub struct CatalogReport
{
    pub nodes: u32,
    pub aliases: u32,
    pub relations: u32,
    /// Edges naming an identifier the catalog does not contain, listed per edge rather
    /// than summarized.
    pub dangling_edges: Vec<String>,
}
