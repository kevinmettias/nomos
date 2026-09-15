//! What ingesting the statements found.

use crate::Divergence as StatementDivergence;
#[derive(Debug, Default)]
pub struct Report
{
    pub ingested: u32,
    pub divergences: Vec<StatementDivergence>,
    pub non_canonical_text: Vec<String>,
}

impl Report
{
    #[must_use]
    pub fn Is_Passing(&self) -> bool
    {
        return self.divergences.is_empty()
            && self.non_canonical_text.is_empty()
            && self.ingested > 0;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Is_Passing_Should_Require_Ingestion_With_No_Divergence_Or_Non_Canonical_Text()
    {
        let clean = Report_Fixture(Vec::new(), Vec::new());
        assert!(clean.Is_Passing());

        let empty = Report::default();
        assert!(!empty.Is_Passing(), "ingesting nothing is not a pass");

        let diverged = Report_Fixture(
            vec![StatementDivergence {
                id: "AGT-001".to_owned(),
                recorded: "sha256:0".to_owned(),
                recomputed: "sha256:1".to_owned(),
                text_is_canonical: true,
            }],
            Vec::new(),
        );
        assert!(!diverged.Is_Passing());

        let non_canonical = Report_Fixture(Vec::new(), vec!["AGT-002".to_owned()]);
        assert!(!non_canonical.Is_Passing());
    }

    fn Report_Fixture(divergences: Vec<StatementDivergence>, non_canonical_text: Vec<String>) -> Report
    {
        return Report {
            ingested: 1,
            divergences,
            non_canonical_text,
        };
    }
}
