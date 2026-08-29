#[derive(Debug, Default)]
pub struct SectionReport
{
    pub headings: u32,
    pub lineage_rows: u32,
    /// Sections naming a document that was never ingested, per section rather than
    /// counted.
    pub unknown_documents: Vec<String>,
}

impl SectionReport
{
    #[must_use]
    pub fn Is_Passed(&self) -> bool
    {
        return self.unknown_documents.is_empty() && self.headings > 0;
    }
}
