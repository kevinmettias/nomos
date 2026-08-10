//! A parsed file of normative statements.

use crate::recorded_statement::RecordedStatement;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct StatementFile
{
    pub statements: Vec<RecordedStatement>,
}
