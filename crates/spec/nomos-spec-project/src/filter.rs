//! Which rows a section admits.

use serde::Serialize;
use serde::Deserialize;
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Filter
{
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier_prefix: Option<String>,
    /// One node, by the identity it is addressed as.
    ///
    /// Apart from `identifier_prefix` because a prefix is not an identity. `FEAT-1` as a
    /// prefix also selects `FEAT-12`, and on relations it narrows only the end the edge
    /// starts from — so a subject asked for its relations would be shown the ones it
    /// declares and not the ones declared about it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}
