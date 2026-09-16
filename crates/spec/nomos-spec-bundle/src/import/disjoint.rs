//! Refusing a bundle that would land on content the store already holds.

use nomos_spec_store::SpecificationStore;
use rusqlite::{Connection, OptionalExtension};

use crate::BundleError;
use crate::Bundle;
use crate::Record;

/// The store holds nothing this bundle also carries.
pub(super) fn Assert_Disjoint(store: &SpecificationStore, bundle: &Bundle) -> Result<(), BundleError>
{
    let connection = store.Connection();

    for record in bundle.Records()
    {
        let Some(identity) = Collides_With_Store(connection, record)?
        else
        {
            continue;
        };

        return Err(BundleError::Occupied {
            table: record.Table().to_owned(),
            identity,
        });
    }

    return Ok(());
}

/// What this record would arrive on top of, if the store already holds it.
fn Collides_With_Store(connection: &Connection, record: &Record) -> Result<Option<String>, BundleError>
{
    let Some(stated) = Stated_Identity(record)
    else
    {
        return Ok(None);
    };
    let held = Already_Holds_From_String_Arguments(connection, stated.sql, &stated.arguments)?;

    return Ok(held.then(|| return stated.identity));
}

/// How a record states its own identity: where a store would already hold it, what that
/// question binds, and how the identity reads in a refusal.
struct Stated<'a>
{
    sql: &'static str,
    arguments: Vec<&'a str>,
    identity: String,
}

/// The identity a record states in its own right, for the records that state one.
///
/// Only these tables are asked about. Every other table is reached through one of them — a
/// block through its document, a history entry through its node, a table row through its
/// block — so once these are disjoint a child row cannot collide either: the parent it hangs
/// from is one this import just inserted. The pass-through kinds are listed explicitly rather
/// than matched with `_`, so that a new record kind has to be thought about here instead of
/// defaulting to unchecked.
///
/// A submission's identity is the node it is, which `nodes` already collides on, so it is
/// deliberately not asked about a second time under its own table.
fn Stated_Identity(record: &Record) -> Option<Stated<'_>>
{
    use super::resolve::{Document_Key_Of, Path, Revision};

    return match record
    {
        Record::Blob(blob) => Some(By_One("SELECT 1 FROM blobs WHERE sha256 = ?1", &blob.sha256)),
        Record::SourceDocument(document) => Some(Stated {
            sql: "SELECT 1 FROM source_documents WHERE path = ?1 AND revision = ?2",
            arguments: vec![document.path.as_str(), document.revision.as_str()],
            identity: Document_Key_Of(Path(&document.path), Revision(&document.revision)),
        }),
        Record::Suite(suite) => Some(By_One("SELECT 1 FROM suites WHERE suite_id = ?1", &suite.suite_id)),
        Record::Node(node) => Some(By_One("SELECT 1 FROM nodes WHERE node_id = ?1", &node.node_id)),
        Record::NodeAlias(alias) => Some(By_One("SELECT 1 FROM node_aliases WHERE alias = ?1", &alias.alias)),
        Record::RelationType(relation_type) => Some(By_One(
            "SELECT 1 FROM relation_types WHERE name = ?1",
            &relation_type.name,
        )),
        Record::NormativeStatement(statement) => Some(By_One(
            "SELECT 1 FROM normative_statements WHERE statement_id = ?1",
            &statement.statement_id,
        )),
        Record::Submission(submission) => Some(By_One(
            "SELECT 1 FROM submissions s JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1",
            &submission.node_id,
        )),
        Record::SourceHeading(_) | Record::SourceBlock(_) | Record::SourceTableRow(_)
        | Record::NodeHistory(_) | Record::Relation(_) | Record::Lineage(_)
        | Record::Omission(_) | Record::RecordFrontMatter(_) | Record::RecordRelation(_)
        | Record::SubmissionValue(_) | Record::SubmissionGap(_) => None,
    };
}

/// A record whose identity is one column, and reads as that column's value.
fn By_One<'a>(sql: &'static str, key: &'a str) -> Stated<'a>
{
    return Stated {
        sql,
        arguments: vec![key],
        identity: key.to_owned(),
    };
}

/// `sql` is `&'static str` so that the statement cannot be built at runtime. Every collision
/// query is a literal written here, and the identity being tested is bound as an argument; the
/// type is what keeps it that way rather than a convention a later edit could quietly drop.
fn Already_Holds_From_String_Arguments(
    connection: &Connection,
    sql: &'static str,
    arguments: &[&str],
) -> Result<bool, BundleError>
{
    let bound = rusqlite::params_from_iter(arguments);

    return Ok(connection
        .query_row(sql, bound, |row| return row.get::<_, i64>(0))
        .optional()?
        .is_some());
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn A_Node() -> Record
    {
        return Record::Node(crate::Node {
            node_id: "N1".to_owned(),
            kind: "requirement".to_owned(),
            authority: "canonical".to_owned(),
            representation: "record".to_owned(),
            title: "Node One".to_owned(),
            deleted_at: None,
            suite_id: None,
        });
    }

    #[test]
    fn Test_Assert_Disjoint_Should_Accept_A_Bundle_The_Store_Does_Not_Already_Hold()
    {
        let store = SpecificationStore::In_Memory()
            .expect("an in-memory store applies the schema this build carries");
        let bundle = Bundle::New(1, vec![A_Node()])
            .expect("a record of strings, integers and enums serialises");

        assert!(Assert_Disjoint(&store, &bundle).is_ok());
    }

    #[test]
    fn Test_Assert_Disjoint_Should_Refuse_A_Node_The_Store_Already_Holds()
    {
        let store = SpecificationStore::In_Memory()
            .expect("an in-memory store applies the schema this build carries");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'requirement', 'canonical', 'record', 'Node One', NULL, NULL);",
            )
            .expect("the node this test then collides with is inserted");
        let bundle = Bundle::New(1, vec![A_Node()])
            .expect("a record of strings, integers and enums serialises");

        let refusal = Assert_Disjoint(&store, &bundle).expect_err("a collision must be refused");
        assert!(
            matches!(refusal, BundleError::Occupied { ref table, ref identity }
                if table == "nodes" && identity == "N1"),
            "{refusal}"
        );
    }
}
