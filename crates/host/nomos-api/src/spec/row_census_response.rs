//! [`RowCensusResponse`], carried only by [`super::spec_table_response::SpecTableResponse`].

use nomos_spec_store::RowCensus;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::RowCensus`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct RowCensusResponse
{
    /// Every pipe line, whatever it turned out to be.
    pub lines: u32,
    pub header: u32,
    pub content: u32,
    pub separator: u32,
    /// Authored lines: header and content together.
    pub non_separator: u32,
}

impl RowCensusResponse
{
    pub(crate) fn From(census: RowCensus) -> Self
    {
        return Self {
            lines: census.lines,
            header: census.header,
            content: census.content,
            separator: census.separator,
            non_separator: census.non_separator,
        };
    }
}
