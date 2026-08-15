//! The seeded store, the synthetic record every edit is made against, and the four-step
//! commit spelled out once.

use nomos_spec_store::{
    AUTHORED,
    DocumentSource,
    EditPreview,
    Seed_Governing_Records,
    SpecificationStore,
};

/// The one document behind a record.
pub(crate) fn Only_Document(store: &SpecificationStore, id: &str) -> DocumentSource
{
    return store
        .Documents_Behind(id, None)
        .expect("queries")
        .first()
        .cloned()
        // A seeded record with no document behind it stored identity and not bytes, so every
        // caller's round trip would then have nothing to compare a projection against. There
        // is no weaker answer available: any stand-in document would turn the comparison into
        // one this helper invented rather than one the seed made.
        .unwrap_or_else(|| panic!("{id} has no document behind it"));
}

/// How many edges join two nodes, in the direction named.
pub(crate) fn Edge_Count(store: &SpecificationStore, from: &str, to: &str) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = ?1 AND t.node_id = ?2",
            rusqlite::params![from, to],
            |row| return row.get(0),
        )
        .expect("queries");
}

/// What committing this edit to the synthetic record would change.
pub(crate) fn Previewed(store: &SpecificationStore, markdown: &str) -> EditPreview
{
    return store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(markdown, None)
        .expect("stages")
        .Preview(store)
        .expect("previews");
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
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds");
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
pub(crate) fn Commit(store: &mut SpecificationStore, markdown: &str, rename: Option<&str>) -> String
{
    let preview = store
        .Claim_For_Edit("D-900", None)
        .expect("claims")
        .Stage(markdown, rename)
        .expect("stages")
        .Preview(store)
        .expect("previews");
    let described = preview.Describe();

    store.Commit_Edit(&preview).expect("commits");

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
        .expect("reads uids");
}
