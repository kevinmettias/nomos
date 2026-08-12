//! One fact a read depended on.

use crate::reader::ReadOutcome;
use crate::fact::FactKey;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency
{
    pub key: FactKey,
    pub outcome: ReadOutcome,
}
