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
