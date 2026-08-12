//! What a census is taken over, and the statement each scope carries.

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
    pub(crate) const fn Statement(self) -> &'static str
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

    pub(crate) fn Arguments(self) -> Vec<i64>
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
