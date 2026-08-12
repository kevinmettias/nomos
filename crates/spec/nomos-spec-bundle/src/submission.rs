//! A born-structured submission, addressed by the node it is.

use serde::{Deserialize, Serialize};

/// A born-structured submission, addressed by the node it is.
///
/// `node_id` rather than `node_uid`, for the reason every other record here carries a natural
/// key: a bundle exported from a database built by importing a bundle must come back the same
/// even though the surrogates were assigned differently.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submission
{
    pub node_id: String,
    pub kind: String,
    pub form_contract_version: i64,
    pub state: String,
    pub submitted_by: String,
    pub submitted_through: String,
}
