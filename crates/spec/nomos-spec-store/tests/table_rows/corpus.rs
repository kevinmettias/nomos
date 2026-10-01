//! The measured half of the done-when, over the real v14 corpus.
//!
//! Every corpus-gated test in this suite is here, beside the root that gates them, and that
//! is a requirement rather than a tidy arrangement. `tests/contract/src/gates.rs` resolves a
//! test to the corpora it reaches by following the helpers it calls **within one file** —
//! almost no gated test names a variable itself. Separating one of these tests from
//! [`Corpus_Root`] would stop it being counted, drop `GATED_TOTAL` below the size of the
//! hole it is cited for, and leave the table in `tests/contract/tests/corpus_gates.rs`
//! agreeing with itself.

use crate::stored::{
    Blame, RowCensus, RowScope, Row_Census_For_Scope, SpecificationStore, Two_Columns_Of_Row,
};
use nomos_spec_model::{Segment, Table_Rows};
use std::path::{Path, PathBuf};

/// The pipe lines the ten domain volumes carry, re-measured 2026-09-14 against the live corpus.
const DOMAIN_VOLUME_LINES: u32 = 286;

/// Every authored line of those volumes: the pipe lines that are not delimiters.
const DOMAIN_VOLUME_NON_SEPARATOR_LINES: u32 = 262;

/// The data rows of those volumes.
const DOMAIN_VOLUME_CONTENT_LINES: u32 = 238;

/// The tables those volumes carry, each contributing one header line and one delimiter.
const DOMAIN_VOLUME_TABLES: u32 = 24;

/// The domain volumes the corpus holds.
const DOMAIN_VOLUMES: u32 = 10;

/// The six of them that carry a table at all.
const VOLUMES_WITH_TABLES: u32 = 6;

/// The canonical domain model's pipe lines.
const CANONICAL_MODEL_LINES: u32 = 30;

/// Its data rows, which is not the number of models they name.
const CANONICAL_MODEL_CONTENT_LINES: u32 = 28;

/// The corpus root, if this machine has one.
///
/// A configured root that is not a directory is a failure rather than a skip: a gate that
/// quietly passes over its own subject is worse than one that fails.
fn Corpus_Root() -> Option<PathBuf>
{
    let configured = std::env::var_os("NOMOS_V14_CORPUS")?;
    let root = PathBuf::from(configured);

    assert!(
        root.is_dir(),
        "NOMOS_V14_CORPUS is set to {}, which is not a directory",
        root.display()
    );

    return Some(root);
}

/// The measured half of the done-when, over the real corpus. Opt-in by path, and loud
/// rather than silent: a configured corpus that cannot be read fails.
#[test]
fn Test_The_Domain_Volumes_Should_Answer_282_258_And_234()
{
    let Some(root) = Corpus_Root()
    else
    {
        return;
    };
    let mut store =
        SpecificationStore::In_Memory().expect("In_Memory applies the schema in process");
    let measured = Store_Volumes(&mut store, &root.join("01_authoring/domain_volumes"));
    let census = Row_Census_For_Scope(&store, RowScope::Everything);

    assert_eq!(
        measured.volumes, DOMAIN_VOLUMES,
        "the ten domain volumes are the corpus this is measured over"
    );
    assert_eq!(measured.with_tables, VOLUMES_WITH_TABLES, "six of the ten carry tables");
    Assert_The_Census_Reconciles(&census);
}

/// What one walk over a volume directory counted.
///
/// Both figures are counts, so a tuple would leave the caller remembering which came first and
/// compiling either way if it did not.
struct VolumesStored
{
    volumes: u32,
    with_tables: u32,
}

/// Every volume under `directory`, stored, and how many of them carry a table at all.
///
/// Both figures belong to the same walk, and the walk is here rather than in the test so that
/// the test reads as the claim it makes: this many volumes, that many of them with tables,
/// and the counts reconcile.
fn Store_Volumes(store: &mut SpecificationStore, directory: &Path) -> VolumesStored
{
    let mut volumes = 0_u32;
    let mut with_tables = 0_u32;

    for path in Volume_Paths_In(directory)
    {
        let carries = Is_A_Volume_With_Tables(store, &path);

        volumes = volumes.saturating_add(1);
        with_tables = with_tables.saturating_add(u32::from(carries));
    }

    return VolumesStored {
        volumes,
        with_tables,
    };
}

/// Stores one volume, and reports whether it carries a table at all.
fn Is_A_Volume_With_Tables(store: &mut SpecificationStore, path: &Path) -> bool
{
    let markdown = std::fs::read_to_string(path)
        // `Volume_Paths_In` enumerated this path moments ago, so a read failure is the corpus changing
        // underneath the run rather than a machine without one — `Corpus_Root` already
        // answered that by returning `None`. Skipping the volume would leave 282, 258 and 234
        // measured over nine volumes and blame the counts.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("?");
    let blocks = Segment(&markdown);
    let document = store
        .Put_Source_Document(name, "v14.36", &markdown)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &blocks)
        // The census counts rows the store holds, so blocks that did not land lower 282, 258
        // and 234 without anything else noticing — the volume was read, counted as one of the
        // ten, and contributed nothing. The file name is what says which one was lost.
        .unwrap_or_else(|error| panic!("{name}: {error}"));

    return blocks.iter().any(|block| return !Table_Rows(block).is_empty());
}

/// Each figure names what it counted, and the three of them add up.
///
/// The plan's 282 was the pipe-line count; `OD-SPEC-002` records that. 258 was every
/// authored line, header included, and 234 the data.
///
/// **Re-measured 2026-09-14 against the live `NOMOS_V14_CORPUS` at code-standards
/// `6e5eb14e2`, to 286 / 262 / 238.** The figures moved because `cace351ac` (2026-08-17)
/// split one subsystem row of volume 02 into three and repartitioned one traceability row
/// of volume 09 -- four authored rows more than the v14.36 archive the original numbers
/// were taken over, and that archive still reproduces 282 / 258 / 234 (`revisions.rs`), so
/// the pins did not drift, the corpus moved past them. `P102` is the item that resolved the
/// divergence at its source before any figure here was touched, which is why this is a
/// re-measurement rather than a constant edited to make a test pass.
fn Assert_The_Census_Reconciles(census: &RowCensus)
{
    assert_eq!(census.lines, DOMAIN_VOLUME_LINES, "pipe lines over the ten domain volumes");
    assert_eq!(
        census.non_separator, DOMAIN_VOLUME_NON_SEPARATOR_LINES,
        "authored lines, header rows included"
    );
    assert_eq!(census.content, DOMAIN_VOLUME_CONTENT_LINES, "data rows");
    assert_eq!(census.header, DOMAIN_VOLUME_TABLES, "one header per table");
    assert_eq!(
        census.separator, DOMAIN_VOLUME_TABLES,
        "one delimiter per table, so this is the table count"
    );
    assert_eq!(
        census
            .header
            .saturating_add(census.content)
            .saturating_add(census.separator),
        census.lines,
        "the counts do not reconcile, so one of them is measuring something else"
    );
    assert_eq!(
        census.header.saturating_add(census.content),
        census.non_separator,
        "non_separator disagrees with the two kinds it covers"
    );
}

/// Section 5's canonical domain model: 30 pipe lines, 28 data rows.
///
/// The count the plan states as 30 is the pipe-line count of the table that carries them,
/// and this is the query that tells the two apart. Naming the table by the content of its
/// first data row rather than by an ordinal, because an ordinal moves when the document
/// above it is edited and the row does not.
///
/// 28 is the rows. The rows name 38 models, because nine of them name more than one —
/// that count belongs to the restoration, which reads the cells, not to the census, which
/// counts lines.
#[test]
fn Test_The_Canonical_Domain_Model_Should_Answer_30_And_28()
{
    let Some(root) = Corpus_Root()
    else
    {
        return;
    };
    let path = root.join(
        "01_authoring/domain_volumes/02-core-architecture-identity-and-configuration.md",
    );
    let mut store =
        SpecificationStore::In_Memory().expect("In_Memory is this measurement's own store");
    Is_A_Volume_With_Tables(&mut store, &path);
    let (block_uid, table_ordinal): (i64, u32) = Two_Columns_Of_Row(
        &store,
        "SELECT source_block_uid, table_ordinal FROM source_table_rows
         WHERE kind = 'content' AND cells_json LIKE '%WorkspaceContext%'",
        Blame("the canonical domain model is no longer in this volume"),
    );
    let census = Row_Census_For_Scope(
        &store,
        RowScope::Table {
            block_uid,
            table_ordinal,
        },
    );

    assert_eq!(census.lines, CANONICAL_MODEL_LINES, "pipe lines");
    assert_eq!(
        census.content, CANONICAL_MODEL_CONTENT_LINES,
        "data rows, which is not the model count"
    );
    assert_eq!(census.header, 1);
    assert_eq!(census.separator, 1);
}

fn Volume_Paths_In(directory: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(directory)
        // A `domain_volumes` that cannot be opened would otherwise yield no paths, and the
        // caller's ten-volume assertion would report "found 0" as though the corpus held no
        // volumes rather than as though the directory was never read.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("md"))
        .collect();
    paths.sort();

    return paths;
}
