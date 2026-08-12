//! What ingesting the statements found.

use crate::content::statement::divergence::StatementDivergence;
#[derive(Debug, Default)]
pub struct StatementReport
{
    pub ingested: u32,
    pub divergences: Vec<StatementDivergence>,
    pub non_canonical_text: Vec<String>,
}

impl StatementReport
{
    #[must_use]
    pub fn Passed(&self) -> bool
    {
        return self.divergences.is_empty()
            && self.non_canonical_text.is_empty()
            && self.ingested > 0;
    }
}
