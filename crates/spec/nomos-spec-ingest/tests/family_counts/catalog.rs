//! The two figures taken over something other than the domain volumes.
//!
//! One reads the machine-readable catalog through the ingest's own parser; the other reads a
//! revision archive's listing without unpacking it. Neither is a heading count, which is why
//! they are not with the heading extractors.

#![allow(dead_code)]

use nomos_spec_ingest::{Archive, Parse_Catalog};
use std::path::Path;

pub(crate) fn Catalog_Entities(corpus: &Path) -> u32
{
    let path = corpus.join("02_machine/catalog/catalog.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let entities = Parse_Catalog(&text).unwrap_or_else(|error| panic!("{error}"));

    return u32::try_from(entities.len()).unwrap_or(u32::MAX);
}

pub(crate) fn V15_Records(archives: &Path) -> u32
{
    use crate::register::V15;

    let archive =
        Archive::Open(&archives.join(V15)).unwrap_or_else(|error| panic!("{error}"));
    let matched = archive
        .Listing()
        .Ending_With(".md")
        .iter()
        .filter(|entry| return entry.contains("/records/"))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}
