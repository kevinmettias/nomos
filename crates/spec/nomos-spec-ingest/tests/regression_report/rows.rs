//! The two checked-in registers, and the shape of a row in each.
//!
//! Both halves of this suite read the headline register: one to check it against itself, one
//! to check it against the archives. So the row types and the readers are here, and the
//! revision labels with them — a suite where two modules named the same revision by two
//! constants would compare two different pairs and call it agreement.

#![allow(dead_code)]

use serde::Deserialize;
use std::collections::BTreeMap;

pub(crate) const HEADLINE: &str =
    include_str!("../../../../../tests/corpus/regression/headline.json");

pub(crate) const COUNTS: &str = include_str!("../../../../../tests/corpus/families/counts.json");

pub(crate) const V14_LAST: &str = "v14.36";

pub(crate) const V15: &str = "v15.0";

pub(crate) const V14_PREVIOUS: &str = "v14.35";

#[derive(Debug, Deserialize)]
pub(crate) struct Entry
{
    pub(crate) id: String,
    pub(crate) row: String,
    pub(crate) clause: String,
    pub(crate) family: Option<String>,
    #[serde(rename = "counts_entry")]
    pub(crate) quoted: Option<String>,
    pub(crate) fates: Option<Fates>,
    pub(crate) measured: BTreeMap<String, u32>,
    pub(crate) resolution: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Fates
{
    pub(crate) preserved: u32,
    pub(crate) hollowed: u32,
    pub(crate) mentioned: u32,
    pub(crate) gone: u32,
}

impl Fates
{
    pub(crate) const fn Total(&self) -> u32
    {
        return self
            .preserved
            .saturating_add(self.hollowed)
            .saturating_add(self.mentioned)
            .saturating_add(self.gone);
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Count
{
    pub(crate) id: String,
    pub(crate) measured: u32,
}

pub(crate) fn Register() -> Vec<Entry>
{
    return serde_json::from_str(HEADLINE).expect("the headline register does not parse");
}

pub(crate) fn Counts() -> Vec<Count>
{
    return serde_json::from_str(COUNTS).expect("the counts register does not parse");
}

pub(crate) fn Restored_Family_For_Label(label: &str) -> nomos_spec_ingest::Restored
{
    return nomos_spec_ingest::Restored::All()
        .iter()
        .find(|family| return family.Label() == label)
        .copied()
        // The register spells families as labels and the report enumerates them as a type,
        // so this is where the two vocabularies are checked to still be one. A label the
        // ingest no longer knows means the register names a family that has been renamed or
        // removed, and no `Restored` value would be the right one to carry on with.
        .unwrap_or_else(|| panic!("{label} is not a restored family"));
}
