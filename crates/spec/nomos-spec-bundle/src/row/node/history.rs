//! One event in a node's life, hash-chained to the one before it.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeHistory
{
    pub node_id: String,
    pub ordinal: i64,
    pub event: String,
    pub reason: String,
    pub previous_event_hash: Option<String>,
    pub event_hash: String,
    pub recorded_at: String,
}
