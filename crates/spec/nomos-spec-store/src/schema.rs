//! The schema, as an ordered list of migrations.
//!
//! Only tables Phase 2 writes to exist here. A table nothing writes to looks like a
//! feature in a schema dump and is not one — the sibling `KnowledgeWorkbench` measured
//! that directly and states the rule as "a schema is not a feature".

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "source-truth-and-node-graph",
    statements: &[
        // Every source byte-stream, addressed by content. Every later claim about what
        // a revision contained resolves here.
        "CREATE TABLE blobs (
             uid          INTEGER PRIMARY KEY,
             sha256       TEXT NOT NULL UNIQUE,
             byte_length  INTEGER NOT NULL,
             content      BLOB NOT NULL
         )",
        "CREATE TABLE source_documents (
             uid          INTEGER PRIMARY KEY,
             path         TEXT NOT NULL,
             revision     TEXT NOT NULL,
             blob_uid     INTEGER NOT NULL REFERENCES blobs(uid),
             UNIQUE (path, revision)
         )",
        "CREATE TABLE source_headings (
             uid          INTEGER PRIMARY KEY,
             document_uid INTEGER NOT NULL REFERENCES source_documents(uid),
             ordinal      INTEGER NOT NULL,
             depth        INTEGER NOT NULL,
             title        TEXT NOT NULL,
             UNIQUE (document_uid, title, depth)
         )",
        "CREATE TABLE source_blocks (
             uid             INTEGER PRIMARY KEY,
             document_uid    INTEGER NOT NULL REFERENCES source_documents(uid),
             ordinal         INTEGER NOT NULL,
             kind            TEXT NOT NULL,
             heading_path    TEXT NOT NULL,
             text            TEXT NOT NULL,
             content_hash    TEXT NOT NULL,
             normalized_hash TEXT NOT NULL,
             UNIQUE (document_uid, ordinal)
         )",
        "CREATE INDEX source_blocks_content ON source_blocks(content_hash)",
        // `uid` is a join surrogate and is never exported. `node_id` is the identity
        // that travels in a bundle. Conflating them makes two databases of the same
        // corpus disagree about what a row is.
        "CREATE TABLE nodes (
             uid            INTEGER PRIMARY KEY,
             node_id        TEXT NOT NULL UNIQUE,
             kind           TEXT NOT NULL,
             authority      TEXT NOT NULL,
             representation TEXT NOT NULL,
             title          TEXT NOT NULL,
             deleted_at     TEXT
         )",
        // Legacy identifiers resolve rather than vanish.
        "CREATE TABLE node_aliases (
             alias    TEXT PRIMARY KEY,
             node_uid INTEGER NOT NULL REFERENCES nodes(uid)
         )",
        // Append-only hash chain. `reason` is NOT NULL and checked non-empty, because a
        // history entry that does not say why is a row that satisfies a schema.
        "CREATE TABLE node_history (
             uid                 INTEGER PRIMARY KEY,
             node_uid            INTEGER NOT NULL REFERENCES nodes(uid),
             ordinal             INTEGER NOT NULL,
             event               TEXT NOT NULL,
             reason              TEXT NOT NULL CHECK (length(trim(reason)) > 0),
             previous_event_hash TEXT,
             event_hash          TEXT NOT NULL,
             recorded_at         TEXT NOT NULL,
             UNIQUE (node_uid, ordinal)
         )",
        "CREATE TABLE relation_types (
             name       TEXT PRIMARY KEY,
             tier       TEXT NOT NULL,
             inverse_of TEXT REFERENCES relation_types(name)
         )",
        "CREATE TABLE relations (
             uid           INTEGER PRIMARY KEY,
             from_node_uid INTEGER NOT NULL REFERENCES nodes(uid),
             relation_type TEXT NOT NULL REFERENCES relation_types(name),
             to_node_uid   INTEGER NOT NULL REFERENCES nodes(uid),
             UNIQUE (from_node_uid, relation_type, to_node_uid)
         )",
        "CREATE TABLE normative_statements (
             uid             INTEGER PRIMARY KEY,
             node_uid        INTEGER NOT NULL REFERENCES nodes(uid),
             statement_id    TEXT NOT NULL UNIQUE,
             kind            TEXT NOT NULL,
             canonical_text  TEXT NOT NULL,
             canonical_hash  TEXT NOT NULL,
             supersedes_hash TEXT
         )",
        // What became of each source block. NSV-PRESERVE-002 and -006 are queries over
        // this table and `omissions`.
        "CREATE TABLE lineage (
             uid                INTEGER PRIMARY KEY,
             source_block_uid   INTEGER REFERENCES source_blocks(uid),
             source_heading_uid INTEGER REFERENCES source_headings(uid),
             disposition        TEXT NOT NULL,
             target_node_uid    INTEGER REFERENCES nodes(uid),
             target_statement   INTEGER REFERENCES normative_statements(uid),
             CHECK (source_block_uid IS NOT NULL OR source_heading_uid IS NOT NULL)
         )",
        // Not a plain UNIQUE. SQL treats two NULLs as distinct, so a UNIQUE over these
        // columns never fires while any of them is null -- which is every row that
        // records a disposition and nothing else. `INSERT OR IGNORE` then inserts a
        // duplicate every time, and re-ingest stops being idempotent without erroring.
        "CREATE UNIQUE INDEX lineage_unique ON lineage (
             coalesce(source_block_uid, -1),
             coalesce(source_heading_uid, -1),
             disposition,
             coalesce(target_node_uid, -1),
             coalesce(target_statement, -1)
         )",
        // A dropped block is recorded, justified and pointed at a decision. This is the
        // only sanctioned way for content to leave, and it is itself validated.
        "CREATE TABLE omissions (
             uid                INTEGER PRIMARY KEY,
             source_block_uid   INTEGER REFERENCES source_blocks(uid),
             source_heading_uid INTEGER REFERENCES source_headings(uid),
             reason             TEXT NOT NULL CHECK (length(trim(reason)) > 0),
             justification      TEXT NOT NULL CHECK (length(trim(justification)) > 0),
             decision_record    TEXT NOT NULL,
             CHECK (source_block_uid IS NOT NULL OR source_heading_uid IS NOT NULL)
         )",
    ],
}];

pub struct Migration
{
    pub version: u32,
    pub name: &'static str,
    pub statements: &'static [&'static str],
}

#[must_use]
pub fn Latest_Version() -> u32
{
    return MIGRATIONS.last().map_or(0, |migration| migration.version);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Versions_Should_Be_Dense_And_Ascending()
    {
        for (index, migration) in MIGRATIONS.iter().enumerate()
        {
            let expected = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
            assert_eq!(migration.version, expected, "{} is out of order", migration.name);
        }
    }

    #[test]
    fn Test_No_Migration_Should_Be_Empty()
    {
        for migration in MIGRATIONS
        {
            assert!(!migration.statements.is_empty(), "{} does nothing", migration.name);
        }
    }
}
