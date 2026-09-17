//! The seeded store, the synthetic record every edit is made against, and the four-step
//! commit spelled out once.

use nomos_spec_store::{
    AUTHORED,
    DocumentSource,
    EditPreview,
    FromNodeId,
    Seed_Governing_Records,
    SpecificationStore,
    ToNodeId,
};

/// The one document behind a record.
pub(crate) fn Only_Document(store: &SpecificationStore, id: &str) -> DocumentSource
{
    return store
        .Documents_Behind(id, None)
        .expect("the seed stores a source document with every record it writes, so this id has a row")
        .first()
        .cloned()
        // A seeded record with no document behind it stored identity and not bytes, so every
        // caller's round trip would then have nothing to compare a projection against. There
        // is no weaker answer available: any stand-in document would turn the comparison into
        // one this helper invented rather than one the seed made.
        .unwrap_or_else(|| panic!("{id} has no document behind it"));
}

/// How many edges join two nodes, in the direction named.
///
/// The two positions are the crate's own node-identifier types rather than bare `&str`, so that
/// a caller cannot hand the target where the source belongs and reverse the edge it is asking
/// about. They accept a `&str` here because a sibling module of this test still calls with two
/// of them.
pub(crate) fn Edge_Count<'a>(
    store: &SpecificationStore,
    from: impl Into<FromNodeId<'a>>,
    to: impl Into<ToNodeId<'a>>,
) -> u32
{
    let from: FromNodeId<'a> = from.into();
    let to: ToNodeId<'a> = to.into();
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = ?1 AND t.node_id = ?2",
            rusqlite::params![from.0, to.0],
            |row| return row.get(0),
        )
        .expect("relations and nodes are both in the schema the store opened with, so the count runs");
}

/// What committing this edit to the synthetic record would change.
pub(crate) fn Preview_For_Markdown(store: &SpecificationStore, markdown: &str) -> EditPreview
{
    return store
        .Claim_For_Edit("D-900", None)
        .expect("D-900 was written through the door by With_Synthetic, so the claim names it")
        .Stage(markdown, None)
        .expect("the claim above came back editable, so replacing its body is allowed")
        .Preview(store)
        .expect("previewing a staged edit writes nothing, so the open store can answer for it");
}

/// The synthetic record with its two sections in the other order and nothing else changed.
pub(crate) fn Swapped() -> String
{
    return SYNTHETIC.replace(
        "## Decision\n\nFirst paragraph.\n\n## Rationale\n\nSecond paragraph.\n",
        "## Rationale\n\nSecond paragraph.\n\n## Decision\n\nFirst paragraph.\n",
    );
}

/// A record written through the door, so the edit tests do not depend on the shape of any
/// particular governing record.
pub(crate) const SYNTHETIC: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                         status: accepted\nversion: 1\n\
                         authority: canonical-normative-record\ntags:\n  - testing\n\
                         relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                         # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                         ## Rationale\n\nSecond paragraph.\n";

pub(crate) const SYNTHETIC_PATH: &str = "docs/records/D-900-a-synthetic-record.md";

pub(crate) fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() applies this crate's schema in process, so opening a fresh store cannot fail");
    Seed_Governing_Records(&mut store)
        .expect("the governing records are compiled into the crate, so seeding them cannot fail");
    return store;
}

pub(crate) fn With_Synthetic() -> SpecificationStore
{
    let mut store = Seeded();
    store
        .Put_Record(SYNTHETIC_PATH, AUTHORED, SYNTHETIC)
        .expect("writes the synthetic record");
    return store;
}

/// Commits an edit through all four steps, so no test spells the sequence out twice.
pub(crate) fn Commit_Staged_Edit(store: &mut SpecificationStore, markdown: &str, rename: Option<&str>) -> String
{
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("Commit_Staged_Edit is only called with D-900, which With_Synthetic wrote before this")
        .Stage(markdown, rename)
        .expect("the claim above is editable, so the replacement body this caller passed is staged")
        .Preview(store)
        .expect("the stage above built an edit over the open store, so there is one to describe");
    let described = preview.Describe();

    store
        .Commit_Edit(&preview)
        .expect("every caller of Commit_Staged_Edit stages a canonical edit, so the transaction writes");

    return described;
}

pub(crate) fn Block_Uids(store: &SpecificationStore, path: &str) -> Vec<i64>
{
    return store
        .Connection()
        .prepare(
            "SELECT b.uid FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.path = ?1 ORDER BY b.ordinal",
        )
        .and_then(|mut statement| {
            return statement
                .query_map(rusqlite::params![path], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("source_blocks holds one row per block of every seeded document, so this path has uids");
}
