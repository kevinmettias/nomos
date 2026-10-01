//! A row read left to right, so a column's place is the order it is asked for rather than a
//! number typed beside the SELECT that chose it.
//!
//! The number and the query drift apart in silence. A column inserted into a SELECT renumbers
//! every column after it, and nothing in the language ties `row.get(7)` to the eighth name in a
//! string literal twenty lines up — the reader keeps compiling and starts filling the wrong
//! fields. Asking in order leaves the SELECT as the only place the order is stated, which is
//! where a reader was going to look anyway.

use rusqlite::Row;
use rusqlite::types::FromSql;

pub(crate) struct Columns<'row, 'statement>
{
    row: &'row Row<'statement>,
    next: usize,
}

impl<'row, 'statement> Columns<'row, 'statement>
{
    pub(crate) fn Of(row: &'row Row<'statement>) -> Self
    {
        return Self { row, next: 0 };
    }

    /// The next column the query names, as whatever type receives it.
    pub(crate) fn Next<Value: FromSql>(&mut self) -> rusqlite::Result<Value>
    {
        let at = self.next;

        self.next = at.saturating_add(1);
        return self.row.get(at);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use rusqlite::Connection;

    /// The first column's value in the two-column row this test reads.
    const FIRST_COLUMN: i64 = 7;

    /// The three values the second row carries, in the order the SELECT names them.
    const FIRST_OF_THREE: i64 = 10;
    const SECOND_OF_THREE: i64 = 20;
    const THIRD_OF_THREE: i64 = 30;

    #[test]
    fn Test_Of_Should_Wrap_A_Row_Starting_Before_Its_First_Column()
    {
        let connection = Connection::open_in_memory().expect("Connection::open_in_memory builds its own schema, so no file is opened");

        connection
            .query_row("SELECT 7, 'seven'", [], |row| {
                let mut columns = Columns::Of(row);
                let number: i64 = columns.Next()?;

                assert_eq!(number, FIRST_COLUMN, "Of should start reading at the row's first column");
                return Ok(());
            })
            .expect("the SELECT names the columns the closure reads");
    }

    #[test]
    fn Test_Next_Should_Advance_Past_Each_Column_It_Reads()
    {
        let connection = Connection::open_in_memory().expect("Connection::open_in_memory builds its own schema, so no file is opened");

        connection
            .query_row("SELECT 10, 20, 30", [], |row| {
                let mut columns = Columns::Of(row);
                let first: i64 = columns.Next()?;
                let second: i64 = columns.Next()?;
                let third: i64 = columns.Next()?;

                assert_eq!(
                    (first, second, third),
                    (FIRST_OF_THREE, SECOND_OF_THREE, THIRD_OF_THREE)
                );
                return Ok(());
            })
            .expect("the SELECT names the columns the closure reads");
    }
}
