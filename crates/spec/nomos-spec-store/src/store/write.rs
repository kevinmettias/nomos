//! Writing content into the schema through a caller's transaction.

mod blocks;
mod documents;
mod relations;

pub(crate) use blocks::Write_Source_Blocks;
pub(crate) use documents::{Write_Blob, Write_Node, Write_Source_Document};
pub(crate) use relations::{Inverse_Of, Write_Relation};

/// Every row a mapped query produced, or the first failure it hit.
pub(crate) fn Collected_Rows<Row>(
    rows: impl Iterator<Item = rusqlite::Result<Row>>,
) -> Result<Vec<Row>, crate::StoreError>
{
    let mut collected = Vec::new();
    for row in rows
    {
        collected.push(row?);
    }

    return Ok(collected);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The middle row of the three this test collects.
    const A_SECOND_ROW: i64 = 2;

    /// The last row of the three this test collects.
    const A_THIRD_ROW: i64 = 3;

    /// A column index no row in this test carries.
    const AN_ABSENT_COLUMN: usize = 9;

    #[test]
    fn Test_Collected_Rows_Should_Gather_Every_Row_Or_Stop_At_The_First_Failure()
    {
        let rows: Vec<rusqlite::Result<i64>> = vec![Ok(1), Ok(A_SECOND_ROW), Ok(A_THIRD_ROW)];
        assert_eq!(
            Collected_Rows(rows.into_iter()).expect("all ok"),
            vec![1, A_SECOND_ROW, A_THIRD_ROW]
        );

        let with_failure: Vec<rusqlite::Result<i64>> =
            vec![Ok(1), Err(rusqlite::Error::InvalidColumnIndex(AN_ABSENT_COLUMN)), Ok(A_THIRD_ROW)];
        assert!(Collected_Rows(with_failure.into_iter()).is_err());
    }
}
