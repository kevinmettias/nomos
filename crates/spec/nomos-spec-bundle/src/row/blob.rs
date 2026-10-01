//! One blob's bytes, addressed by the digest of what they are.

pub(crate) mod encoding;

use serde::{Deserialize, Serialize};

use crate::Encoding;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob
{
    pub sha256: String,
    pub byte_length: i64,
    pub encoding: Encoding,
    pub content: String,
}
