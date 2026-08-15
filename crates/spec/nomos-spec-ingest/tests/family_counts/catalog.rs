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
        // The caller has already asserted NOMOS_V14_CORPUS is a directory, so a catalog that
        // is not under it means the configured root is not the v14 corpus. The path is in the
        // message because that is the only way to tell which tree was pointed at.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    // `catalog.entities` is defined as what the ingest's own parser reads out of the catalog,
    // so a parser that refuses the file is the definition failing rather than a count that
    // happens to be unavailable. The parse error already names what it could not read.
    let entities = Parse_Catalog(&text).unwrap_or_else(|error| panic!("{error}"));

    return u32::try_from(entities.len()).unwrap_or(u32::MAX);
}

pub(crate) fn V15_Records(archives: &Path) -> u32
{
    use crate::register::V15;

    let archive =
        // `V15` names one archive, and `v15.records` was measured over that one. An archive
        // root missing it is the wrong root, and the listing that follows would count records
        // from a revision the register never measured.
        Archive::Open(&archives.join(V15)).unwrap_or_else(|error| panic!("{error}"));
    let matched = archive
        .Listing()
        .Ending_With(".md")
        .iter()
        .filter(|entry| return entry.contains("/records/"))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}
