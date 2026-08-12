//! The bundle format this build writes, and the schema the corpus was read at.

use serde::{Deserialize, Serialize};

/// The bundle format this build writes and is willing to read.
pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header
{
    pub format: u32,
    pub schema_version: u32,
}
