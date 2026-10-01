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
// A function could not do this. `concat!` takes literals, so each scope's predicate has to be
// joined to the shared select list where the scope is written; a `fn(&str) -> String` would
// build the statement at run time and `Statement` below could then be neither `const fn` nor a
// return of `&'static str`.
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// A document surrogate, for the scope that names one.
    const A_DOCUMENT_UID: i64 = 7;

    /// A block surrogate and the ordinal of the table inside it, for the scope that
    /// names both.
    const A_BLOCK_UID: i64 = 3;
    const A_TABLE_ORDINAL: u32 = 2;

    #[test]
    fn Test_Statement_Should_Carry_The_Right_Predicate_For_Each_Scope()
    {
        assert!(RowScope::Everything.Statement().contains("1 = 1"));
        assert!(RowScope::Document(1).Statement().contains("block.document_uid = ?1"));
        assert!(
            RowScope::Table { block_uid: 1, table_ordinal: 0 }
                .Statement()
                .contains("line.table_ordinal = ?2")
        );
    }

    #[test]
    fn Test_Arguments_Should_Bind_What_Each_Scopes_Statement_Needs()
    {
        assert_eq!(RowScope::Everything.Arguments(), Vec::<i64>::new());
        assert_eq!(RowScope::Document(A_DOCUMENT_UID).Arguments(), vec![A_DOCUMENT_UID]);
        assert_eq!(
            RowScope::Table { block_uid: A_BLOCK_UID, table_ordinal: A_TABLE_ORDINAL }
                .Arguments(),
            vec![A_BLOCK_UID, i64::from(A_TABLE_ORDINAL)]
        );
    }
}
