//! A typed edge declared in a record's front matter.

use serde::Deserialize;

/// A typed edge declared in a record's front matter.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Relation
{
    pub target: String,
    /// v14 and most of v15 spell this `type`; v15's `spec-governance` records spell it
    /// `relation`. `validate_bundle.py` reads either (`relation.get('type') or
    /// relation.get('relation')`), so one reader has to as well — accepting only the
    /// first spelling would make a whole governance suite unreadable.
    #[serde(rename = "type", alias = "relation")]
    pub relation: String,
}
