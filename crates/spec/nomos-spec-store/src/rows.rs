//! Counting table rows, one query per number.
//!
//! Every count the regression report carries is a line count under a looser definition
//! than the thing it names — 282 pipe lines for 258 non-separator rows, 30 pipe lines for
//! 28 domain models. The fix is not a better definition; it is a census where each number
//! comes from its own query over a typed column, so no caller has to subtract one figure
//! from another and hope the two were measured the same way.

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

impl RowScope
{
    fn Predicate(self) -> (&'static str, Vec<i64>)
    {
        return match self
        {
            Self::Everything => ("1 = 1", Vec::new()),
            Self::Document(uid) => ("block.document_uid = ?1", vec![uid]),
            Self::Table {
                block_uid,
                table_ordinal,
            } => (
                "line.source_block_uid = ?1 AND line.table_ordinal = ?2",
                vec![block_uid, i64::from(table_ordinal)],
            ),
        };
    }
}

/// The counts, each measured separately.
///
/// `lines` and `non_separator` are not derived from the other fields. They are their own
/// queries, so a caller reporting "282 pipe lines, 258 non-separator" is quoting two
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
    return Ok(RowCensus {
        lines: Count(connection, scope, "1 = 1")?,
        header: Count(connection, scope, "line.kind = 'header'")?,
        content: Count(connection, scope, "line.kind = 'content'")?,
        separator: Count(connection, scope, "line.kind = 'separator'")?,
        non_separator: Count(connection, scope, "line.kind <> 'separator'")?,
    });
}

fn Count(connection: &Connection, scope: RowScope, kind: &str) -> Result<u32, StoreError>
{
    let (predicate, arguments) = scope.Predicate();
    let sql = format!(
        "SELECT count(*) FROM source_table_rows line
         JOIN source_blocks block ON block.uid = line.source_block_uid
         WHERE ({predicate}) AND ({kind})"
    );

    return Ok(connection.query_row(&sql, rusqlite::params_from_iter(arguments), |row| {
        row.get(0)
    })?);
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
