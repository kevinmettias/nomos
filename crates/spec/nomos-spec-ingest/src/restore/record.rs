//! Writing what was recognised into the store, and finding it again by name.

use super::{
    BTreeMap, Extract_Members, IngestError, Member, NodeRow, Refuse_Collisions, RestorationReport, SpecificationStore,
    StoreError,
};
use nomos_spec_store::{DocumentPath, DocumentRevision};

mod aliases;
mod tracing;

use aliases::{Ambiguous_Names, Claim_Alias};

/// I5 — restores every family across the corpus into a store that already holds it.
///
/// Takes the whole document set rather than one volume at a time, because both things
/// that have to be unique are corpus-wide. A minted identifier is stable across databases,
/// so two members colliding is a collision wherever they live; and a bare name resolving
/// to one node is a claim about the corpus, not about a file. Restoring volume by volume
/// would make both answers depend on the order the directory was read in.
///
/// Every document must already be ingested. Restoring from text the store never saw would
/// make the source-truth gate optional for exactly the content the restoration cares
/// about, and a node whose lineage points at a block nobody hashed is a node with a
/// provenance nothing checked.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if a document is not in the store at that revision or
/// two members collide, and [`IngestError::Store`] on any store failure.
pub fn Restore_Members(
    store: &mut SpecificationStore,
    revision: &str,
    documents: &BTreeMap<String, String>,
) -> Result<RestorationReport, IngestError>
{
    let located = Located_Members(store, revision, documents)?;
    let members: Vec<Member> = located.iter().map(|(_, member)| return member.clone()).collect();
    Refuse_Collisions(&members)?;

    let mut report = RestorationReport {
        ambiguous_names: Ambiguous_Names(&members),
        ..RestorationReport::default()
    };
    for (document_uid, member) in located
    {
        Record_Member(store, document_uid, member, &mut report)?;
    }

    return Ok(report);
}

/// Every member every document declares, with the store row its document occupies.
///
/// Read in full before anything is written, because a collision is only visible across
/// documents and half a restoration is harder to undo than none.
pub(super) fn Located_Members(
    store: &mut SpecificationStore,
    revision: &str,
    documents: &BTreeMap<String, String>,
) -> Result<Vec<(i64, Member)>, IngestError>
{
    let mut located: Vec<(i64, Member)> = Vec::new();

    for (document, markdown) in documents
    {
        let document_uid = Document_Uid(store, DocumentRevision(revision), DocumentPath(document))?;
        for member in Extract_Members(DocumentPath(document), markdown)?
        {
            located.push((document_uid, member));
        }
    }

    return Ok(located);
}

/// Mints one member's node, ties it to its text, and gives it its name.
pub(super) fn Record_Member(
    store: &mut SpecificationStore,
    document_uid: i64,
    member: Member,
    report: &mut RestorationReport,
) -> Result<(), IngestError>
{
    use self::tracing::Trace_Member;

    let node_uid = store.Upsert_Node(NodeRow {
        node_id: &member.id,
        kind: member.family.Node_Kind(),
        authority: "canonical",
        representation: "record",
        title: &member.name,
    })?;

    Trace_Member(store, document_uid, &member, node_uid)?;
    Claim_Alias(store, &member, node_uid, report)?;
    report.members.push(member);

    return Ok(());
}

pub(super) fn Document_Uid(
    store: &SpecificationStore,
    revision: DocumentRevision<'_>,
    document: DocumentPath<'_>,
) -> Result<i64, IngestError>
{
    let revision = revision.0;
    let document = document.0;

    return store
        .Connection()
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            rusqlite::params![document, revision],
            |row| row.get(0),
        )
        .map_err(|cause| {
            if matches!(cause, rusqlite::Error::QueryReturnedNoRows)
            {
                return IngestError::Parse(format!(
                    concat!(
                        "{document} is not in the store at {revision}, so there is nothing for a ",
                        "restored node to trace back to",
                    ),
                    document = document,
                    revision = revision
                ));
            }

            return IngestError::Parse(format!(
                "the store could not be asked for {document} at {revision}: {cause}"
            ));
        });
}

/// Resolves an identifier or an authored name to a node.
///
/// The alias table is what makes `WorkspaceContext` answer as well as `CDM-WORKSPACECONTEXT`.
/// Without it a restored concept resolves only by the identifier this build minted, and the
/// name the corpus actually uses would find nothing.
///
/// # Errors
///
/// Returns [`IngestError::Store`] on any store failure.
pub fn Resolve_Model_Uid(store: &SpecificationStore, name: &str) -> Result<Option<i64>, IngestError>
{
    if let Some(uid) = store.Node_Uid(name)?
    {
        return Ok(Some(uid));
    }

    return Sql_Result(store
        .Connection()
        .query_row(
            "SELECT node_uid FROM node_aliases WHERE alias = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|error| {
            return match error
            {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            };
        }));
}

pub(super) fn Sql_Result<Value>(result: rusqlite::Result<Value>) -> Result<Value, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Restore_Members_Should_Produce_A_Report_Naming_Every_Member()
    {
        let Core { mut store, documents } = Store_With_Core();

        let report = Restore_Members(&mut store, "v14.36", &documents).expect("restores");

        assert_eq!(report.members.len(), 1);
        assert_eq!(
            report.members.first().map(|member| member.id.as_str()),
            Some("CDM-WORKSPACECONTEXT")
        );
        assert!(report.ambiguous_names.is_empty());
        assert!(report.contested_aliases.is_empty());
    }

    #[test]
    fn Test_Located_Members_Should_Pair_Each_Member_With_The_Document_It_Came_From()
    {
        let Core { mut store, documents } = Store_With_Core();

        let located = Located_Members(&mut store, "v14.36", &documents).expect("locates");

        assert_eq!(located.len(), 1);
        let (document_uid, member) = located.first().expect("the assertion above confirms exactly one located member");
        assert_eq!(member.id, "CDM-WORKSPACECONTEXT");
        assert!(*document_uid > 0);
    }

    #[test]
    fn Test_Record_Member_Should_Upsert_A_Node_Trace_It_And_Append_It_To_The_Report()
    {
        let Core { mut store, documents } = Store_With_Core();
        let located = Located_Members(&mut store, "v14.36", &documents).expect("locates");
        let (document_uid, member) = located.into_iter().next().expect("one member");
        let mut report = RestorationReport::default();

        Record_Member(&mut store, document_uid, member.clone(), &mut report).expect("records");

        assert_eq!(report.members, vec![member]);
        assert!(store.Node_Uid("CDM-WORKSPACECONTEXT").expect("looks up").is_some());
    }

    #[test]
    fn Test_Document_Uid_Should_Find_The_Row_For_A_Known_Revision_And_Refuse_An_Unknown_One()
    {
        let Core { store, documents: _documents } = Store_With_Core();

        let uid = Document_Uid(&store, DocumentRevision("v14.36"), DocumentPath("02-core.md")).expect("finds");
        assert!(uid > 0);

        let refusal =
            Document_Uid(&store, DocumentRevision("v14.36"), DocumentPath("no-such.md")).expect_err("must refuse");
        assert!(format!("{refusal}").contains("is not in the store"), "{refusal}");
    }

    #[test]
    fn Test_Resolve_Model_Uid_Should_Answer_By_Identifier_Or_By_The_Corpus_Name()
    {
        let Core { mut store, documents } = Store_With_Core();
        Restore_Members(&mut store, "v14.36", &documents).expect("restores");

        assert!(Resolve_Model_Uid(&store, "CDM-WORKSPACECONTEXT").expect("resolves").is_some());
        assert!(Resolve_Model_Uid(&store, "WorkspaceContext").expect("resolves").is_some());
        assert!(Resolve_Model_Uid(&store, "NoSuchModel").expect("resolves").is_none());
    }

    #[test]
    fn Test_Sql_Result_Should_Wrap_A_Failure_As_A_Store_Error()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        let failure: rusqlite::Result<i64> =
            store.Connection().query_row("SELECT * FROM no_such_table", [], |row| row.get(0));

        let wrapped = Sql_Result(failure).expect_err("must wrap the failure");
        assert!(matches!(wrapped, IngestError::Store(StoreError::Sql(_))));

        let ok = Sql_Result(Ok::<i64, rusqlite::Error>(42)).expect("passes through Ok");
        assert_eq!(ok, 42);
    }
}

#[cfg(test)]
const CORE_MARKDOWN: &str = concat!(
    "# Core\n\n## 5. Canonical domain model\n\n",
    "| Model | Responsibility |\n| --- | --- |\n",
    "| WorkspaceContext | Repository. |\n",
);

/// A store already holding one ingested document, paired with the document set that produced
/// it — named rather than a bare tuple so a caller cannot swap the two.
#[cfg(test)]
struct Core
{
    store: SpecificationStore,
    documents: BTreeMap<String, String>,
}

/// A store holding one located member, minted as a node of its own, and the identity of
/// both — named rather than a tuple so `document_uid` and `node_uid` cannot be swapped.
#[cfg(test)]
struct LocatedMemberWithNode
{
    store: SpecificationStore,
    document_uid: i64,
    member: Member,
    node_uid: i64,
}

/// A store holding one located member, minted as a node of its own — the setup the tests in
/// `tracing` and `aliases` share.
///
/// It sits at module scope rather than inside `tests` because all three test modules need it,
/// and a private item here is visible to every one of them.
#[cfg(test)]
fn A_Located_Member_With_Its_Node() -> LocatedMemberWithNode
{
    let Core { mut store, documents } = Store_With_Core();
    let located = Located_Members(&mut store, "v14.36", &documents).expect("locates");
    let (document_uid, member) = located.into_iter().next().expect("one member");
    let node_uid = store
        .Upsert_Node(NodeRow {
            node_id: &member.id,
            kind: "concept",
            authority: "canonical",
            representation: "record",
            title: &member.name,
        })
        .expect("the located member's own identifier is not in the store, so the upsert writes a row");

    return LocatedMemberWithNode { store, document_uid, member, node_uid };
}

/// A store already holding one ingested document, and the document set that produced it.
#[cfg(test)]
fn Store_With_Core() -> Core
{
    use crate::Ingest_Source_Document;

    let mut store = SpecificationStore::In_Memory().expect("opens");
    Ingest_Source_Document(&mut store, "02-core.md", "v14.36", CORE_MARKDOWN).expect("ingests");

    let mut documents = BTreeMap::new();
    documents.insert("02-core.md".to_owned(), CORE_MARKDOWN.to_owned());

    return Core { store, documents };
}
