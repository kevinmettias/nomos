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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Is_Passed_Should_Require_Headings_And_No_Unknown_Documents()
    {
        let mut report = SectionReport::default();
        assert!(!report.Is_Passed());

        report.headings = 1;
        assert!(report.Is_Passed());

        report.unknown_documents.push("missing.md".to_owned());
        assert!(!report.Is_Passed());
    }
}
