//! The register itself, and the shape of one entry in it.
//!
//! Both halves of this suite read it: one to check it against itself, one to check it
//! against the corpus. `VOLUMES` is here for the same reason — two modules naming the
//! volumes directory by two constants would measure two trees and call it agreement.

#![allow(dead_code)]

use serde::Deserialize;

const REGISTER: &str = include_str!("../../../../../tests/corpus/families/counts.json");

pub(crate) const V15: &str = "nomos-spec-v15.0.zip";

pub(crate) const VOLUMES: &str = "01_authoring/domain_volumes";

#[derive(Debug, Deserialize)]
pub(crate) struct Entry
{
    pub(crate) id: String,
    pub(crate) family: String,
    pub(crate) measured: u32,
    pub(crate) unit: String,
    pub(crate) definition: String,
    pub(crate) corpus: String,
    pub(crate) plan: Option<PlanFigure>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlanFigure
{
    /// Absent where the plan names the family but states no number.
    pub(crate) figure: Option<u32>,
    pub(crate) named: String,
    pub(crate) status: String,
    pub(crate) note: String,
}

pub(crate) fn Register() -> Vec<Entry>
{
    return serde_json::from_str(REGISTER).expect("the register does not parse");
}
