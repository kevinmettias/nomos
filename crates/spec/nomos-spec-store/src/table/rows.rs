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
}
