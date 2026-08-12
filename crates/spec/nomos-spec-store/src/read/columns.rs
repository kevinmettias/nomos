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
