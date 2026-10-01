//! A second name one node is also cited by.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias
{
    pub alias: String,
    pub node_id: String,
}
