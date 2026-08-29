//! A decision the submission needs that nobody has taken yet.

use serde::{Deserialize, Serialize};

/// A decision the submission needs that nobody has taken yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionGap
{
    pub node_id: String,
    pub ordinal: i64,
    pub question: String,
    pub blocks: String,
    pub severity: String,
    pub closed_by: Option<String>,
}
