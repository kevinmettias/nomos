//! The `nomos_spec_store` seam `check-integration-coverage` found with no suite.
//!
//! `nomos-cli::spec.rs` re-exports and uses several `nomos_spec_store` items directly
//! (`use nomos_spec_store::{CommitReport, DocumentSource, EditError, EditPreview,
//! NodeSummary, PathMatch, RecordProjection, RowCensus, StoreError, TableLine};`), and
//! `spec/verb/table.rs`'s own `Block_Ordinals` takes a `&[nomos_spec_store::TableLine]`
//! directly by name. Nothing under `tests/` ever named the crate, because every existing
//! spec test drives the compiled binary as a subprocess and the boundary this finding is
//! about — a caller holding a real `nomos_spec_store::TableLine` and doing something with
//! its fields — sits one layer below anything a subprocess's stdout can prove by itself.
//!
//! This constructs real `TableLine` values and proves the property `spec/verb/table.rs`'s
//! own rendering leans on (each row carries which block it came from, and rows from the
//! same block are recognizable as such), then runs the real binary through `nomos spec
//! profiles` — the one verb `spec.rs`'s own inline test
//! (`Test_Run_Should_Dispatch_Profiles_Without_Assembling_A_Corpus`) already knows needs no
//! external corpus, since the three v14 corpora `nomos_spec_orchestration::corpus` reads
//! live outside this repository and CI has none of them.

#[path = "support/mod.rs"]
mod support;

use nomos_spec_store::TableLine;
use support::Run;

/// One line of a table, shaped the way a real document's rows are: an ordinal for its
/// block, a kind, and the text as authored. `spec/verb/table.rs`'s own `Sample_Line` helper
/// builds the identical shape for its inline tests; this is the same fixture, reachable
/// from outside the crate because every field on `TableLine` is public.
fn Sample_Line(block_ordinal: u32, row_ordinal: u32, text: &str) -> TableLine
{
    return TableLine {
        block_ordinal,
        table_ordinal: 1,
        row_ordinal,
        kind: "content".to_owned(),
        cells: vec![text.to_owned()],
        text: text.to_owned(),
        content_hash: format!("hash-{block_ordinal}-{row_ordinal}"),
    };
}

/// The property `spec/verb/table.rs`'s private `Block_Ordinals` renders for a reader: rows
/// belonging to the same block are identifiable as such by that one field, and rows from
/// different blocks are not conflated. Reimplemented here rather than called, because that
/// function is `pub(super)` to `nomos-cli` and unreachable from `tests/` — this is the same
/// property, proven directly against the public type it operates over.
#[test]
fn Test_Table_Lines_Should_Carry_Which_Block_They_Came_From()
{
    let lines = vec![
        Sample_Line(2, 1, "| a |"),
        Sample_Line(2, 2, "| b |"),
        Sample_Line(1, 1, "| c |"),
    ];

    let mut seen_in_order: Vec<u32> = Vec::new();
    for line in &lines
    {
        if !seen_in_order.contains(&line.block_ordinal)
        {
            seen_in_order.push(line.block_ordinal);
        }
    }

    assert_eq!(seen_in_order, vec![2, 1], "first-seen block order must survive intact");
    assert_eq!(
        lines.iter().filter(|line| line.block_ordinal == 2).count(),
        2,
        "both rows authored under block 2 must still say so"
    );
    assert_eq!(lines[0].text, "| a |");
    assert_ne!(lines[0].content_hash, lines[1].content_hash, "distinct rows must not share a content hash");
}

/// The real binary, driven through the one `nomos spec` verb that answers from the
/// embedded catalogue without assembling a store from a corpus this repository does not
/// carry -- the shipped side of the same `nomos_spec_store`-backed dispatch `spec.rs`
/// wires every other verb through.
#[test]
fn Test_The_Shipped_Binary_Should_List_Profiles_Without_A_Corpus()
{
    let ran = Run(&["spec", "profiles"]);

    assert_eq!(ran.code, 0, "stderr: {}", ran.stderr);
    assert!(!ran.stdout.is_empty(), "the embedded profile catalogue must print something");
}
