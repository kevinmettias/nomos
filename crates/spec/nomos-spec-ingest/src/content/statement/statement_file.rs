//! A parsed file of normative statements.

use crate::RecordedStatement;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct StatementFile
{
    pub statements: Vec<RecordedStatement>,
}
