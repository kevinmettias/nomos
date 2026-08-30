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

    #[test]
    fn Test_Collected_Rows_Should_Gather_Every_Row_Or_Stop_At_The_First_Failure()
    {
        let rows: Vec<rusqlite::Result<i64>> = vec![Ok(1), Ok(2), Ok(3)];
        assert_eq!(Collected_Rows(rows.into_iter()).expect("all ok"), vec![1, 2, 3]);

        let with_failure: Vec<rusqlite::Result<i64>> =
            vec![Ok(1), Err(rusqlite::Error::InvalidColumnIndex(9)), Ok(3)];
        assert!(Collected_Rows(with_failure.into_iter()).is_err());
    }
}
