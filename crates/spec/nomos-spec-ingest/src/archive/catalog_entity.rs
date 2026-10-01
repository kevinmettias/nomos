//! One catalog node as the corpus declares it.

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct CatalogEntity
{
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authority: String,
    #[serde(default)]
    pub representation: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}
