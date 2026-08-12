//! A parsed file of normative statements.

use crate::content::statement::recorded::RecordedStatement;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct StatementFile
{
    pub statements: Vec<RecordedStatement>,
}
