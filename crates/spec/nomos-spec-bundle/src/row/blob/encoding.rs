//! How a blob's bytes are spelled in the bundle.

use serde::{Deserialize, Serialize};

/// How a blob's bytes are spelled in the bundle.
///
/// Chosen from the bytes alone, so a re-export picks the same arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobEncoding
{
    Utf8,
    Base64,
}
