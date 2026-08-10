//! The schema, as an ordered list of migrations.
//!
//! Only tables Phase 2 writes to exist here. A table nothing writes to looks like a
//! feature in a schema dump and is not one — the sibling `KnowledgeWorkbench` measured
//! that directly and states the rule as "a schema is not a feature".

pub const MIGRATIONS: &[Migration] = &[
    Migration {
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
    },
    Migration {
        version: 2,
        name: "table-rows-as-typed-subjects",
        statements: &[
            // Every pipe line of a table block, typed. The block above stays the
            // preservation authority — it holds the verbatim text and both v14 hashes —
            // so these rows are what lets a loss report name a row by identity instead
            // of reporting a count. `kind` is what keeps the line count and the content
            // count two queries over one table rather than one number bent to fit.
            "CREATE TABLE source_table_rows (
                 uid              INTEGER PRIMARY KEY,
                 source_block_uid INTEGER NOT NULL REFERENCES source_blocks(uid),
                 ordinal          INTEGER NOT NULL,
                 table_ordinal    INTEGER NOT NULL,
                 kind             TEXT NOT NULL CHECK (kind IN ('content', 'separator')),
                 cells_json       TEXT NOT NULL,
                 text             TEXT NOT NULL,
                 content_hash     TEXT NOT NULL,
                 normalized_hash  TEXT NOT NULL,
                 UNIQUE (source_block_uid, ordinal)
             )",
            "CREATE INDEX source_table_rows_block ON source_table_rows(source_block_uid)",
            "CREATE INDEX source_table_rows_content ON source_table_rows(content_hash)",
        ],
    },
    Migration {
        version: 3,
        name: "row-lineage-and-header-rows",
        statements: &[
            // SQLite cannot widen a CHECK in place, so the row table is rebuilt. `uid` is
            // carried across explicitly rather than reassigned: it is what a lineage row
            // points at, and renumbering here would be the silent-loss shape this schema
            // exists to prevent, committed by the migration itself.
            "CREATE TABLE source_table_rows_next (
                 uid              INTEGER PRIMARY KEY,
                 source_block_uid INTEGER NOT NULL REFERENCES source_blocks(uid),
                 ordinal          INTEGER NOT NULL,
                 table_ordinal    INTEGER NOT NULL,
                 kind             TEXT NOT NULL
                                  CHECK (kind IN ('header', 'content', 'separator')),
                 cells_json       TEXT NOT NULL,
                 text             TEXT NOT NULL,
                 content_hash     TEXT NOT NULL,
                 normalized_hash  TEXT NOT NULL,
                 UNIQUE (source_block_uid, ordinal)
             )",
            "INSERT INTO source_table_rows_next
             (uid, source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
              content_hash, normalized_hash)
             SELECT uid, source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
                    content_hash, normalized_hash
             FROM source_table_rows",
            "DROP TABLE source_table_rows",
            "ALTER TABLE source_table_rows_next RENAME TO source_table_rows",
            "CREATE INDEX source_table_rows_block ON source_table_rows(source_block_uid)",
            "CREATE INDEX source_table_rows_content ON source_table_rows(content_hash)",
            // Rows written under version 2 typed a header as content, because there was no
            // header. Re-deriving here rather than leaving them is what keeps one store's
            // answer to "how many data rows" independent of when it was built. A table with
            // no delimiter yields NULL from the subquery, so the comparison is NULL and the
            // row is left alone — the same refusal to guess the typing makes.
            "UPDATE source_table_rows SET kind = 'header'
             WHERE kind = 'content'
               AND ordinal < (
                   SELECT delimiter.ordinal FROM source_table_rows delimiter
                   WHERE delimiter.source_block_uid = source_table_rows.source_block_uid
                     AND delimiter.table_ordinal = source_table_rows.table_ordinal
                     AND delimiter.kind = 'separator'
               )",
            // A concept minted from a table row must trace to that row. Pointing it at the
            // containing block instead would say the whole table produced it, and the
            // one-canonical-source property the restoration exists to establish would not
            // be expressible — thirty concepts and one block is not a lineage.
            "CREATE TABLE lineage_next (
                 uid                  INTEGER PRIMARY KEY,
                 source_block_uid     INTEGER REFERENCES source_blocks(uid),
                 source_heading_uid   INTEGER REFERENCES source_headings(uid),
                 source_table_row_uid INTEGER REFERENCES source_table_rows(uid),
                 disposition          TEXT NOT NULL,
                 target_node_uid      INTEGER REFERENCES nodes(uid),
                 target_statement     INTEGER REFERENCES normative_statements(uid),
                 CHECK (source_block_uid IS NOT NULL
                     OR source_heading_uid IS NOT NULL
                     OR source_table_row_uid IS NOT NULL)
             )",
            "INSERT INTO lineage_next
             (uid, source_block_uid, source_heading_uid, disposition, target_node_uid,
              target_statement)
             SELECT uid, source_block_uid, source_heading_uid, disposition, target_node_uid,
                    target_statement
             FROM lineage",
            "DROP TABLE lineage",
            "ALTER TABLE lineage_next RENAME TO lineage",
            // Same reasoning as version 1: SQL treats two NULLs as distinct, so a plain
            // UNIQUE never fires on the rows that carry nulls, and `INSERT OR IGNORE`
            // stops being idempotent without erroring.
            "CREATE UNIQUE INDEX lineage_unique ON lineage (
                 coalesce(source_block_uid, -1),
                 coalesce(source_heading_uid, -1),
                 coalesce(source_table_row_uid, -1),
                 disposition,
                 coalesce(target_node_uid, -1),
                 coalesce(target_statement, -1)
             )",
        ],
    },
    Migration {
        version: 4,
        name: "sibling-suites",
        statements: &[
            // A suite is whose specification this is. `authority_root` is the whole point:
            // this repository's own suite is the root, and the XVPE, KWB and ecosystem
            // seeds are not. Without it a cross-suite relation looks exactly like an
            // internal one, and a sibling's decision reads as ours.
            //
            // Its own table rather than a value on `nodes.authority`. Authority says how
            // far a statement may be trusted; rootness says whose statement it is. Merging
            // them would give one column two meanings and no way to ask either question
            // cleanly — the same collapse the deliberate non-collapses warn about.
            "CREATE TABLE suites (
                 uid            INTEGER PRIMARY KEY,
                 suite_id       TEXT NOT NULL UNIQUE,
                 title          TEXT NOT NULL,
                 authority_root INTEGER NOT NULL CHECK (authority_root IN (0, 1))
             )",
            // Nullable, and deliberately so. Every node ingested before suites existed
            // belongs to no recorded suite, and defaulting them to the root would assert
            // ownership nothing established. Unrecorded stays unrecorded and answerable.
            "ALTER TABLE nodes ADD COLUMN suite_uid INTEGER REFERENCES suites(uid)",
            "CREATE INDEX nodes_suite ON nodes(suite_uid)",
        ],
    },
    Migration {
        version: 5,
        name: "declared-front-matter",
        statements: &[
            // What a record's front matter said, as it said it. `nodes` already holds the
            // kind, the authority and the title, and `relations` already holds the edges —
            // so this table exists for the three fields nothing kept (`status`, `version`,
            // `tags`) and for one property the graph cannot have.
            //
            // That property is directedness. `relations` is completed with inverses on the
            // way in, deliberately, so that `verifies` and `verified_by` are one fact. A
            // record that is merely the *target* of a `relates-to` therefore has an
            // outgoing edge it never declared, and `relates-to` is its own inverse, so the
            // graph cannot tell which end wrote it down. Rendering front matter from the
            // graph would put edges in a file that its author did not write — which is the
            // silent-content-change this store exists to make impossible, committed by the
            // authoring surface itself. Measured, not feared: OD-LEDGER-009 declares three
            // `relates-to` edges, and rendering OD-LEDGER-001 from the graph invents one.
            "CREATE TABLE record_front_matter (
                 document_uid INTEGER PRIMARY KEY REFERENCES source_documents(uid),
                 node_uid     INTEGER NOT NULL REFERENCES nodes(uid),
                 status       TEXT NOT NULL,
                 version      INTEGER NOT NULL,
                 tags_json    TEXT NOT NULL
             )",
            // Ordered, because the order is in the file and a set is not a document.
            "CREATE TABLE record_relations (
                 uid          INTEGER PRIMARY KEY,
                 document_uid INTEGER NOT NULL REFERENCES source_documents(uid),
                 ordinal      INTEGER NOT NULL,
                 target       TEXT NOT NULL,
                 relation     TEXT NOT NULL,
                 UNIQUE (document_uid, ordinal)
             )",
            "CREATE INDEX record_relations_document ON record_relations(document_uid)",
        ],
    },
];

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
