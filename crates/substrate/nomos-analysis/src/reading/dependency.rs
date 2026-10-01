//! One fact a read depended on.

use crate::ReadOutcome;
use crate::FactKey;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency
{
    pub key: FactKey,
    pub outcome: ReadOutcome,
}
