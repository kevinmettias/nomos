//! How each column of the store reaches the bundle, declared one column at a time.

/// How one column of the store reaches the bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Carried
{
    /// A join surrogate. Withheld on purpose: two databases built from one bundle assign
    /// different surrogates and must still be the same corpus.
    Surrogate,
    /// The record field that carries it, which may spell it differently — a bundle
    /// carries `blob_sha256` where the store carries `blob_uid`, because a bundle
    /// addresses content and a database addresses rows.
    Field(&'static str),
}

pub(super) struct Coverage
{
    pub(super) table: &'static str,
    pub(super) columns: &'static [(&'static str, Carried)],
}

pub(super) const COVERAGE: &[Coverage] = &[
    Coverage {
        table: "record_front_matter",
        columns: &[
            ("document_uid", Carried::Field("document")),
            ("node_uid", Carried::Field("node_id")),
            ("status", Carried::Field("status")),
            ("version", Carried::Field("version")),
            ("tags_json", Carried::Field("tags")),
        ],
    },
    Coverage {
        table: "record_relations",
        columns: &[
            ("uid", Carried::Surrogate),
            ("document_uid", Carried::Field("document")),
            ("ordinal", Carried::Field("ordinal")),
            ("target", Carried::Field("target")),
            ("relation", Carried::Field("relation")),
        ],
    },
    // The three tables `OD-SPEC-013` decides. They travel in the bundle because the bundle is
    // the committed durable form -- `OD-SPEC-008` makes the structured record canonical, the
    // deterministic text serialization its git-visible shape, and SQLite a derived index any
    // checkout rebuilds. A submission that did not export would be a submission that does not
    // survive a clone.
    Coverage {
        table: "submissions",
        columns: &[
            ("uid", Carried::Surrogate),
            ("node_uid", Carried::Field("node_id")),
            ("kind", Carried::Field("kind")),
            ("form_contract_version", Carried::Field("form_contract_version")),
            ("state", Carried::Field("state")),
            ("submitted_by", Carried::Field("submitted_by")),
            ("submitted_through", Carried::Field("submitted_through")),
        ],
    },
    Coverage {
        table: "submission_values",
        columns: &[
            ("uid", Carried::Surrogate),
            ("submission_uid", Carried::Field("node_id")),
            ("field", Carried::Field("field")),
            ("ordinal", Carried::Field("ordinal")),
            ("origin", Carried::Field("origin")),
            ("value", Carried::Field("value")),
            ("value_hash", Carried::Field("value_hash")),
            ("supersedes_hash", Carried::Field("supersedes_hash")),
            ("recorded_at", Carried::Field("recorded_at")),
        ],
    },
    Coverage {
        table: "submission_gaps",
        columns: &[
            ("uid", Carried::Surrogate),
            ("submission_uid", Carried::Field("node_id")),
            ("ordinal", Carried::Field("ordinal")),
            ("question", Carried::Field("question")),
            ("blocks", Carried::Field("blocks")),
            ("severity", Carried::Field("severity")),
            ("closed_by", Carried::Field("closed_by")),
        ],
    },
    Coverage {
        table: "blobs",
        columns: &[
            ("uid", Carried::Surrogate),
            ("sha256", Carried::Field("sha256")),
            ("byte_length", Carried::Field("byte_length")),
            ("content", Carried::Field("content")),
        ],
    },
    Coverage {
        table: "source_documents",
        columns: &[
            ("uid", Carried::Surrogate),
            ("path", Carried::Field("path")),
            ("revision", Carried::Field("revision")),
            ("blob_uid", Carried::Field("blob_sha256")),
        ],
    },
    Coverage {
        table: "source_headings",
        columns: &[
            ("uid", Carried::Surrogate),
            ("document_uid", Carried::Field("document")),
            ("ordinal", Carried::Field("ordinal")),
            ("depth", Carried::Field("depth")),
            ("title", Carried::Field("title")),
        ],
    },
    Coverage {
        table: "source_blocks",
        columns: &[
            ("uid", Carried::Surrogate),
            ("document_uid", Carried::Field("document")),
            ("ordinal", Carried::Field("ordinal")),
            ("kind", Carried::Field("kind")),
            ("heading_path", Carried::Field("heading_path")),
            ("text", Carried::Field("text")),
            ("content_hash", Carried::Field("content_hash")),
            ("normalized_hash", Carried::Field("normalized_hash")),
        ],
    },
    Coverage {
        table: "source_table_rows",
        columns: &[
            ("uid", Carried::Surrogate),
            ("source_block_uid", Carried::Field("block")),
            ("ordinal", Carried::Field("ordinal")),
            ("table_ordinal", Carried::Field("table_ordinal")),
            ("kind", Carried::Field("kind")),
            ("cells_json", Carried::Field("cells")),
            ("text", Carried::Field("text")),
            ("content_hash", Carried::Field("content_hash")),
            ("normalized_hash", Carried::Field("normalized_hash")),
        ],
    },
    Coverage {
        table: "suites",
        columns: &[
            ("uid", Carried::Surrogate),
            ("suite_id", Carried::Field("suite_id")),
            ("title", Carried::Field("title")),
            ("authority_root", Carried::Field("authority_root")),
        ],
    },
    Coverage {
        table: "nodes",
        columns: &[
            ("uid", Carried::Surrogate),
            ("node_id", Carried::Field("node_id")),
            ("kind", Carried::Field("kind")),
            ("authority", Carried::Field("authority")),
            ("representation", Carried::Field("representation")),
            ("title", Carried::Field("title")),
            ("deleted_at", Carried::Field("deleted_at")),
            ("suite_uid", Carried::Field("suite_id")),
        ],
    },
    Coverage {
        table: "node_aliases",
        columns: &[
            ("alias", Carried::Field("alias")),
            ("node_uid", Carried::Field("node_id")),
        ],
    },
    Coverage {
        table: "node_history",
        columns: &[
            ("uid", Carried::Surrogate),
            ("node_uid", Carried::Field("node_id")),
            ("ordinal", Carried::Field("ordinal")),
            ("event", Carried::Field("event")),
            ("reason", Carried::Field("reason")),
            ("previous_event_hash", Carried::Field("previous_event_hash")),
            ("event_hash", Carried::Field("event_hash")),
            ("recorded_at", Carried::Field("recorded_at")),
        ],
    },
    Coverage {
        table: "relation_types",
        columns: &[
            ("name", Carried::Field("name")),
            ("tier", Carried::Field("tier")),
            ("inverse_of", Carried::Field("inverse_of")),
            ("domain_kinds_json", Carried::Field("domain")),
            ("range_kinds_json", Carried::Field("range")),
            ("max_per_node", Carried::Field("max_per_node")),
        ],
    },
    Coverage {
        table: "relations",
        columns: &[
            ("uid", Carried::Surrogate),
            ("from_node_uid", Carried::Field("from_node_id")),
            ("relation_type", Carried::Field("relation_type")),
            ("to_node_uid", Carried::Field("to_node_id")),
        ],
    },
    Coverage {
        table: "normative_statements",
        columns: &[
            ("uid", Carried::Surrogate),
            ("node_uid", Carried::Field("node_id")),
            ("statement_id", Carried::Field("statement_id")),
            ("kind", Carried::Field("kind")),
            ("canonical_text", Carried::Field("canonical_text")),
            ("canonical_hash", Carried::Field("canonical_hash")),
            ("supersedes_hash", Carried::Field("supersedes_hash")),
        ],
    },
    Coverage {
        table: "lineage",
        columns: &[
            ("uid", Carried::Surrogate),
            ("source_block_uid", Carried::Field("source_block")),
            ("source_heading_uid", Carried::Field("source_heading")),
            ("source_table_row_uid", Carried::Field("source_table_row")),
            ("disposition", Carried::Field("disposition")),
            ("target_node_uid", Carried::Field("target_node_id")),
            ("target_statement", Carried::Field("target_statement_id")),
        ],
    },
    Coverage {
        table: "omissions",
        columns: &[
            ("uid", Carried::Surrogate),
            ("source_block_uid", Carried::Field("source_block")),
            ("source_heading_uid", Carried::Field("source_heading")),
            ("reason", Carried::Field("reason")),
            ("justification", Carried::Field("justification")),
            ("decision_record", Carried::Field("decision_record")),
        ],
    },
];
