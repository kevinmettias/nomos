//! A parsed file of normative statements.

use crate::RecordedStatement;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct File
{
    pub statements: Vec<RecordedStatement>,
}
