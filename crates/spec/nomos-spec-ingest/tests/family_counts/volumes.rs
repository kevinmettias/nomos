//! Reading a domain volume, and counting what its tables and its code amount to.
//!
//! Every figure here is taken through the store's own segmenter and row typing rather than
//! through a second reading written for this suite. Two implementations of "what is a table
//! row" would be two authorities on how many the corpus has.

#![allow(dead_code)]

use nomos_spec_model::{BlockKind, Segment, SourceBlock};
use nomos_spec_store::{RowScope, SpecificationStore};
use std::path::{Path, PathBuf};

pub(crate) fn Volume(corpus: &Path, stem: &str) -> String
{
    use crate::register::VOLUMES;

    let directory = corpus.join(VOLUMES);

    for path in Markdown_Files(&directory)
    {
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        if name.starts_with(stem)
        {
            return std::fs::read_to_string(&path)
                // The listing named this path a moment ago, so a read that fails now is the
                // corpus moving under the suite. Every figure defined over this volume would
                // otherwise be taken against an empty document and come out low.
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        }
    }

    // Falling out of the loop means the volume an extractor addresses by stem has been
    // renamed. A stem that matches nothing is the definition going stale, and the register
    // entry it feeds would otherwise be checked against a number nothing produced.
    panic!("no volume beginning {stem} in {}", directory.display());
}

fn Markdown_Files(directory: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(directory)
        // Every extractor reaches the corpus through this one listing. An unreadable
        // directory answered with an empty vector would turn all of them into zeroes at
        // once, so it is named here rather than once per caller.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("md"))
        .collect();
    paths.sort();

    assert!(!paths.is_empty(), "{} holds no markdown", directory.display());
    return paths;
}

/// One volume read off disk and put into the store, so a census can be taken over all of
/// them afterwards.
fn Put_Volume(store: &mut SpecificationStore, path: &Path)
{
    let markdown = std::fs::read_to_string(path)
        // Skipping an unreadable volume would drop its rows from the census, and a short
        // census reads as the corpus holding fewer table rows rather than as a file this
        // run could not open.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("?");
    let document = store
        .Put_Source_Document(name, "v14.36", &markdown)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &Segment(&markdown))
        // The census is the store's own count of what it was given, so a volume whose blocks
        // the store refused must not go on to be counted as holding none. The document name
        // is in the message because the census itself has no place for it.
        .unwrap_or_else(|error| panic!("{name}: {error}"));
}

/// The row census over every domain volume, taken through the store so the numbers come
/// from the same segmenter and the same typing the ingest uses.
pub(crate) fn Volume_Census(volumes: &Path) -> nomos_spec_store::RowCensus
{
    let mut store = SpecificationStore::In_Memory()
        .expect("an in-memory store opens over no file, so this construction has no failure path");

    for path in Markdown_Files(volumes)
    {
        Put_Volume(&mut store, &path);
    }

    return store.Row_Census(RowScope::Everything).expect("takes a census");
}

pub(crate) fn Tables(volumes: &Path) -> u32
{
    let mut tables = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            // `table.tables` is a sum across all ten volumes, so a volume that failed to read
            // contributes nothing and is indistinguishable from a volume holding no tables.
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for block in Segment(&markdown)
        {
            let highest = nomos_spec_model::Table_Rows(&block)
                .iter()
                .map(|row| return row.table_ordinal)
                .max()
                .unwrap_or(0);
            tables = tables.saturating_add(highest);
        }
    }

    return tables;
}

pub(crate) fn Fence_Lines(volumes: &Path) -> u32
{
    let mut fences = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            // The same silent subtraction, this time from `code.fence_lines`: a volume nobody
            // could open would be counted as a volume carrying no fences at all.
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for line in markdown.lines()
        {
            if line.trim_start().starts_with("```")
            {
                fences = fences.saturating_add(1);
            }
        }
    }

    return fences;
}

pub(crate) fn Code_Blocks(volumes: &Path) -> u32
{
    let mut blocks = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            // And again for `code.blocks`. The figure only means anything taken over all ten
            // volumes, so nine of them is a wrong answer rather than a partial one.
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for block in Segment(&markdown)
        {
            if block.kind == BlockKind::Code
            {
                blocks = blocks.saturating_add(1);
            }
        }
    }

    return blocks;
}

/// A heading addressed by its own text, as the document titles it.
///
/// Distinct from the volume stem rather than a second `&str`, because both are text and a
/// call site handing them over in the wrong order would compile.
pub(crate) struct TableHeading<'a>(pub(crate) &'a str);

/// Whether a block carries at least one table row.
///
/// Named rather than written into the condition below, because `Table_Rows` is a pipeline:
/// the reader would have to run it to answer the yes-or-no question being asked.
fn Carries_Rows(block: &SourceBlock) -> bool
{
    return !nomos_spec_model::Table_Rows(block).is_empty();
}

/// The pipe lines and the data rows of the one table sitting directly under a heading.
///
/// Refuses a heading carrying more than one table rather than summing them: "the table
/// under §5" would then mean something the register's sentence does not say.
pub(crate) fn Table_Under(corpus: &Path, stem: &str, heading: TableHeading<'_>) -> TableCounts
{
    let TableHeading(heading) = heading;
    let markdown = Volume(corpus, stem);
    let mut found: Vec<SourceBlock> = Vec::new();
    for block in Segment(&markdown)
    {
        let directly_under = block.heading_path.last().map(String::as_str) == Some(heading);
        if directly_under && Carries_Rows(&block)
        {
            found.push(block);
        }
    }
    assert_eq!(found.len(), 1, "{heading} in {stem} carries {} tables, not one", found.len());
    let rows = found.first().map(nomos_spec_model::Table_Rows).unwrap_or_default();
    let content = rows
        .iter()
        .filter(|row| return row.kind == nomos_spec_model::RowKind::Content)
        .count();

    return TableCounts {
        lines: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        content: u32::try_from(content).unwrap_or(u32::MAX),
    };
}

/// What one table under a heading amounts to.
///
/// Named rather than a pair. Both members are `u32` and the compiler cannot tell them
/// apart, so a call site reading the wrong position gets a number that looks right.
pub(crate) struct TableCounts
{
    pub(crate) lines: u32,
    pub(crate) content: u32,
}

/// Counted through the restoration's own reader, not a second implementation of the split.
/// Two readings of "does this cell name one model or three" would be two authorities on
/// how many models the corpus has.
pub(crate) fn Named_Models(corpus: &Path) -> u32
{
    let markdown = Volume(corpus, "02-core");
    let mut models = 0_u32;

    for block in Segment(&markdown)
    {
        if block.heading_path.last().map(String::as_str) != Some("5. Canonical domain model")
        {
            continue;
        }
        for row in nomos_spec_model::Table_Rows(&block)
            .iter()
            .filter(|row| return row.kind == nomos_spec_model::RowKind::Content)
        {
            let cell = row.cells.first().map_or("", |cell| return cell.trim());
            models = models.saturating_add(
                u32::try_from(nomos_spec_ingest::Models_In(cell).len()).unwrap_or(u32::MAX),
            );
        }
    }

    return models;
}
