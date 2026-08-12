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

use crate::columns::Columns;
use crate::store::StoreError;
use rusqlite::Connection;

/// What a census is taken over.
///
/// The three scopes are the three the report actually asks for: a corpus, a volume, and
/// one table inside one block — the last being how "28 domain models" is a query rather
/// than a number somebody counted by hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowScope
{
    Everything,
    Document(i64),
    Table
    {
        block_uid: i64,
        table_ordinal: u32,
    },
}

/// One scope's whole census statement, joined at COMPILE time.
///
/// `concat!` joins string literals into a constant, so a scope carries a finished statement
/// rather than a predicate woven into a template at runtime — there is no point at which
/// this SQL is a value the program built. The shared select list is written once here
/// instead of once per scope.
macro_rules! Census_Statement
{
    ($predicate:literal) =>
    {
        concat!(
            "SELECT count(*),
                    coalesce(sum(line.kind = 'header'), 0),
                    coalesce(sum(line.kind = 'content'), 0),
                    coalesce(sum(line.kind = 'separator'), 0),
                    coalesce(sum(line.kind <> 'separator'), 0)
             FROM source_table_rows line
             JOIN source_blocks block ON block.uid = line.source_block_uid
             WHERE ",
            $predicate
        )
    };
}

impl RowScope
{
    const fn Statement(self) -> &'static str
    {
        return match self
        {
            Self::Everything => Census_Statement!("1 = 1"),
            Self::Document(_) => Census_Statement!("block.document_uid = ?1"),
            Self::Table { .. } =>
            {
                Census_Statement!("line.source_block_uid = ?1 AND line.table_ordinal = ?2")
            },
        };
    }

    fn Arguments(self) -> Vec<i64>
    {
        return match self
        {
            Self::Everything => Vec::new(),
            Self::Document(uid) => vec![uid],
            Self::Table {
                block_uid,
                table_ordinal,
            } => vec![block_uid, i64::from(table_ordinal)],
        };
    }
}

/// The counts, each measured separately.
///
/// `lines` and `non_separator` are not derived from the other fields. They are their own
/// aggregates, so a caller reporting "282 pipe lines, 258 non-separator" is quoting two
/// measurements rather than one measurement and one subtraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowCensus
{
    /// Every pipe line, whatever it turned out to be.
    pub lines: u32,
    pub header: u32,
    pub content: u32,
    pub separator: u32,
    /// Authored lines: header and content together.
    pub non_separator: u32,
}

/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub fn Census(connection: &Connection, scope: RowScope) -> Result<RowCensus, StoreError>
{
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
