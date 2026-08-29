//! One typed edge between two nodes.

pub(crate) mod relation_type;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation
{
    pub from_node_id: String,
    pub relation_type: String,
    pub to_node_id: String,
}
