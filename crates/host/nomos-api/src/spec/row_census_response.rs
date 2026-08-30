//! [`RowCensusResponse`], carried only by [`super::table_response::TableResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Row_Census()
    {
        let census = RowCensus { lines: 10, header: 1, content: 7, separator: 1, non_separator: 8 };

        let response = RowCensusResponse::From(census);

        assert_eq!(response.lines, census.lines);
        assert_eq!(response.header, census.header);
        assert_eq!(response.content, census.content);
        assert_eq!(response.separator, census.separator);
        assert_eq!(response.non_separator, census.non_separator);
    }
}
