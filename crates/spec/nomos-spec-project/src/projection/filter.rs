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
    /// One record's declared lifecycle status, as its own front matter wrote it.
    ///
    /// Not a property of the node. A node is an identity in the graph and the graph holds no
    /// status; `status` lives in `record_front_matter`, which the authoring surface fills
    /// from the file the author wrote. `OD-SPEC-016`'s neighbours aside, the reason that
    /// matters here is that a node referenced but never authored has no front-matter row at
    /// all, and a filter on this field excludes it rather than reporting it under a status
    /// nobody declared.
    ///
    /// Honoured by `nodes` alone, because it is the only content kind that resolves to a
    /// record. Every other kind refuses it through `ProjectError::UnsupportedFilter`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
