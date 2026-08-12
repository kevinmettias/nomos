//! One store row a projection was built from.

use crate::Content;
use serde::Serialize;
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Input
{
    pub content: Content,
    pub identity: String,
    pub hash: String,
}
