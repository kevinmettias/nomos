//! Counting table rows, each number its own measurement.
//!
//! Every count the regression report carries is a line count under a looser definition
//! than the thing it names — 282 pipe lines for 258 non-separator rows, 30 pipe lines for
//! 28 domain models. The fix is not a better definition; it is a census where each number
//! comes from its own aggregate over a typed column, so no caller has to subtract one
//! figure from another and hope the two were measured the same way.
//!
//! The aggregates share one scan rather than one query each. That is a change in how the
//! numbers are fetched and not in what they mean: `non_separator` is still counted, never
//! derived from `lines` and `separator`, which is the property this module exists to keep.

use crate::RowCensus;
use crate::RowScope;
use crate::StoreError;
use rusqlite::Connection;

/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub fn Counted_Rows(connection: &Connection, scope: RowScope) -> Result<RowCensus, StoreError>
{
    use crate::read::columns::Columns;

    return Ok(connection.query_row(
        scope.Statement(),
        rusqlite::params_from_iter(scope.Arguments()),
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(RowCensus {
                lines: columns.Next()?,
                header: columns.Next()?,
                content: columns.Next()?,
                separator: columns.Next()?,
                non_separator: columns.Next()?,
            });
        },
    )?);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::RowKind;

    /// A fourth kind must not be addable while the census keeps quiet about it: `lines`
    /// would keep rising with nothing saying where the difference went.
    #[test]
    fn Test_The_Census_Should_Carry_A_Field_For_Every_Kind()
    {
        assert_eq!(
            RowKind::All().len(),
            3,
            "a row kind was added; RowCensus has no field for it"
        );
    }

    #[test]
    fn Test_Counted_Rows_Should_Read_The_Scoped_Census_From_The_Connection()
    {
        let mut store = crate::SpecificationStore::In_Memory().expect("opens");
        let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";
        let uid = store.Put_Source_Document("a.md", "v1", markdown).expect("writes");
        store
            .Put_Source_Blocks(uid, &nomos_spec_model::Segment(markdown))
            .expect("writes");

        let census = Counted_Rows(store.Connection(), RowScope::Everything).expect("counts");

        assert_eq!(census.lines, 3);
        assert_eq!(census.header, 1);
        assert_eq!(census.content, 1);
        assert_eq!(census.separator, 1);
    }
}
