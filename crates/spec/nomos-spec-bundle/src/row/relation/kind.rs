//! A relation name, its tier, and the name of its inverse.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationType
{
    pub name: String,
    pub tier: String,
    pub inverse_of: Option<String>,
}
