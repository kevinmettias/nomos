//! Column-level completeness: every column of the store reaches the bundle.
//!
//! The row-count guard cannot see this failure. Add a column to a table, teach nothing to
//! export it, and the counts still agree on both sides — the bundle carries exactly as
//! many records as the store has rows, each one quietly missing a field. Round-tripping it
//! is a fixpoint too, because the second export drops the same column the first did.
//!
//! So each column declares how it is carried, and the declaration is checked three ways:
//! the schema must hold no column the declaration omits, the declaration must name no
//! column the schema lacks, and a named record field must actually exist on the record.
//! The third is what stops the declaration from being satisfied by typing the column's
//! name into a list.

use crate::BundleError;
use crate::model::Record;
use nomos_spec_store::Table;
use rusqlite::Connection;
use std::collections::BTreeSet;

/// How one column of the store reaches the bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Carried
{
    /// A join surrogate. Withheld on purpose: two databases built from one bundle assign
    /// different surrogates and must still be the same corpus.
    Surrogate,
    /// The record field that carries it, which may spell it differently — a bundle
    /// carries `blob_sha256` where the store carries `blob_uid`, because a bundle
    /// addresses content and a database addresses rows.
    Field(&'static str),
}

struct Coverage
{
    table: &'static str,
    columns: &'static [(&'static str, Carried)],
}

const COVERAGE: &[Coverage] = &[
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

/// # Errors
///
/// Returns [`BundleError::UncoveredColumn`] for a column the exporter does not carry,
/// [`BundleError::PhantomColumn`] for a declaration the schema does not have, and
/// [`BundleError::Uncarried`] for a declared field no record of that table holds.
pub(crate) fn Assert_Columns_Covered(
    connection: &Connection,
    records: &[Record],
) -> Result<(), BundleError>
{
    for table in Table::All()
    {
        let name = table.Name();
        let declared = COVERAGE
            .iter()
            .find(|coverage| coverage.table == name)
            .map_or(&[] as &[(&str, Carried)], |coverage| coverage.columns);

        let present = Schema_Columns(connection, name)?;
        Compare(name, &present, declared)?;
        Assert_Fields_Exist(name, declared, records)?;
    }

    return Ok(());
}

/// Both directions. A missing declaration is a column nobody exports; a stale one is a
/// declaration that stopped describing anything and would go on satisfying the guard.
fn Compare(
    table: &str,
    schema: &[String],
    declared: &[(&str, Carried)],
) -> Result<(), BundleError>
{
    let named: BTreeSet<&str> = declared.iter().map(|(column, _)| return *column).collect();

    for column in schema
    {
        if !named.contains(column.as_str())
        {
            return Err(BundleError::UncoveredColumn {
                table: table.to_owned(),
                column: column.clone(),
            });
        }
    }

    let present: BTreeSet<&str> = schema.iter().map(String::as_str).collect();
    for column in named
    {
        if !present.contains(column)
        {
            return Err(BundleError::PhantomColumn {
                table: table.to_owned(),
                column: column.to_owned(),
            });
        }
    }

    return Ok(());
}

/// The declaration names a field; the record has to have it.
///
/// Checked against the records being exported rather than a constructed sample, so it is
/// the real serialization that answers. A table with no rows cannot answer at all and is
/// skipped — the round-trip fixture holds at least one row of every table precisely so
/// that this is exercised rather than skipped everywhere.
fn Assert_Fields_Exist(
    table: &str,
    declared: &[(&str, Carried)],
    records: &[Record],
) -> Result<(), BundleError>
{
    let Some(sample) = records.iter().find(|record| return record.Table() == table)
    else
    {
        return Ok(());
    };

    let fields = Fields(sample)?;
    for (column, carried) in declared
    {
        let Carried::Field(field) = carried
        else
        {
            continue;
        };

        if !fields.contains(*field)
        {
            return Err(BundleError::Uncarried {
                table: table.to_owned(),
                column: (*column).to_owned(),
                field: (*field).to_owned(),
            });
        }
    }

    return Ok(());
}

fn Fields(record: &Record) -> Result<BTreeSet<String>, BundleError>
{
    let value = serde_json::to_value(record)?;
    let Some(serde_json::Value::Object(payload)) = value.get("record")
    else
    {
        return Err(BundleError::Malformed(format!(
            "a {} record does not serialize as a payload object",
            record.Table()
        )));
    };

    return Ok(payload.keys().cloned().collect());
}

fn Schema_Columns(connection: &Connection, table: &str) -> Result<Vec<String>, BundleError>
{
    // `table` comes from `Table::All()`, which is an enum, so this is not a hole through
    // which arbitrary SQL reaches the database.
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = statement
        .query_map([], |row| return row.get::<_, String>(1))?
        .collect::<Result<Vec<String>, _>>()?;

    if names.is_empty()
    {
        return Err(BundleError::Malformed(format!(
            "the store has no table named {table}, so its columns cannot be checked"
        )));
    }

    return Ok(names);
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Schema(columns: &[&str]) -> Vec<String>
    {
        return columns.iter().map(|column| return (*column).to_owned()).collect();
    }

    #[test]
    fn Test_A_Fully_Declared_Table_Should_Pass()
    {
        let declared = &[("uid", Carried::Surrogate), ("path", Carried::Field("path"))];

        assert!(Compare("t", &Schema(&["uid", "path"]), declared).is_ok());
    }

    /// The failure the guard exists for: a column joins the schema and nothing carries it.
    #[test]
    fn Test_An_Undeclared_Column_Should_Be_Refused()
    {
        let declared = &[("uid", Carried::Surrogate)];

        let refusal = Compare("t", &Schema(&["uid", "note"]), declared)
            .expect_err("an undeclared column must be refused");

        assert!(
            matches!(refusal, BundleError::UncoveredColumn { ref column, .. } if column == "note"),
            "{refusal}"
        );
    }

    /// A declaration that stopped describing anything would otherwise go on passing.
    #[test]
    fn Test_A_Declaration_The_Schema_Dropped_Should_Be_Refused()
    {
        let declared = &[("uid", Carried::Surrogate), ("gone", Carried::Field("gone"))];

        let refusal = Compare("t", &Schema(&["uid"]), declared)
            .expect_err("a stale declaration must be refused");

        assert!(
            matches!(refusal, BundleError::PhantomColumn { ref column, .. } if column == "gone"),
            "{refusal}"
        );
    }

    #[test]
    fn Test_A_Declared_Field_The_Record_Lacks_Should_Be_Refused()
    {
        let record = Record::Blob(crate::model::Blob {
            sha256: "sha256:aa".to_owned(),
            byte_length: 2,
            encoding: crate::model::BlobEncoding::Utf8,
            content: "hi".to_owned(),
        });
        let declared = &[("sha256", Carried::Field("digest"))];

        let refusal = Assert_Fields_Exist("blobs", declared, std::slice::from_ref(&record))
            .expect_err("a field the record does not have must be refused");

        assert!(
            matches!(refusal, BundleError::Uncarried { ref field, .. } if field == "digest"),
            "{refusal}"
        );
        assert!(
            Assert_Fields_Exist("blobs", &[("sha256", Carried::Field("sha256"))], &[record])
                .is_ok(),
            "the real field must satisfy it, or the check proves nothing"
        );
    }

    /// Every table the store knows about declares its columns somewhere.
    #[test]
    fn Test_Every_Table_Should_Have_A_Coverage_Entry()
    {
        for table in Table::All()
        {
            assert!(
                COVERAGE.iter().any(|coverage| coverage.table == table.Name()),
                "{} declares no columns, so every one of them is uncovered",
                table.Name()
            );
        }
    }
}
