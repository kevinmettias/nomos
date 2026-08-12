//! P1: database to export to import to export yields a byte-identical bundle.
//!
//! The property is worthless over an empty store — two empty bundles are byte-identical
//! and prove nothing. So the fixture fills every table the store knows about, and
//! [`fixpoint::Test_Every_Table_Should_Be_Exercised`] fails if one of them is ever empty.


mod common;
mod fixpoint;
mod governing_records;
mod refusals;
mod table_rows;
