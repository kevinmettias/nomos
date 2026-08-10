//! A v15 artifact document, as its front matter declares it.

use crate::family::Family;
use serde::Deserialize;
/// One authored v14 artifact, reduced to what reconciliation compares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact
{
    pub id: String,
    pub family: Family,
    /// How many criteria the artifact declared. Zero for requirements and stories.
    pub criteria: usize,
    pub statement: String,
}

#[derive(Deserialize)]
pub(crate) struct ArtifactFrontMatter
{
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) statement: String,
    /// Acceptance artifacts carry a list instead of one statement. Reconciliation needs
    /// one comparable text per identifier, so the criteria are joined in declared order —
    /// which also means a criterion silently dropped from the list changes the hash.
    #[serde(default)]
    pub(crate) criteria: Vec<Criterion>,
}

#[derive(Deserialize)]
pub(crate) struct Criterion
{
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) statement: String,
}
