//! One section of a profile, named apart from a projection's section.

use crate::Filter;
use crate::Content;
use serde::Serialize;
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProfileSection
{
    pub title: String,
    pub content: Content,
    #[serde(default)]
    pub filter: Filter,
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub may_be_empty: bool,
}
