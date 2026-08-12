//! One node of the domain model, keyed by the identifier people cite.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
    pub deleted_at: Option<String>,
    /// `None` where no suite is recorded, which is not the root.
    pub suite_id: Option<String>,
}
