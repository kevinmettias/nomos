//! Whose specification a set of nodes is.

use serde::{Deserialize, Serialize};

/// Whose specification a set of nodes is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suite
{
    pub suite_id: String,
    pub title: String,
    pub authority_root: bool,
}
