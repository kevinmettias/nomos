//! Writing what was recognised into the store, and finding it again by name.

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

use super::{
    BTreeMap, Extract_Members, IngestError, Member, NodeRow, Origin, Refuse_Collisions, RestorationReport,
    SpecificationStore, StoreError,
};
use nomos_spec_store::{DocumentPath, DocumentRevision};

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

/// A row's address in the store.
///
/// Three numbers that only mean anything together, and carrying them as one value is what
/// keeps the row tracer inside the argument budget.
#[derive(Clone, Copy)]
pub(super) struct RowAt
{
    document_uid: i64,
    block_ordinal: u32,
    row_ordinal: u32,
}

/// Ties a node to the text it was minted from.
pub(super) fn Trace_Member(
    store: &mut SpecificationStore,
    document_uid: i64,
    member: &Member,
    node_uid: i64,
) -> Result<(), IngestError>
{
    match member.origin
    {
        Origin::Block { ordinal } => Dispose_Block(store, document_uid, ordinal, node_uid)?,
        Origin::Row {
            block_ordinal,
            row_ordinal,
        } =>
        {
            let at = RowAt {
                document_uid,
                block_ordinal,
                row_ordinal,
            };
            Trace_Row(store, at, member, node_uid)?;
        }
    }

    return Ok(());
}

/// Ties a node to the one row it was minted from.
///
/// A row the store does not hold is refused rather than skipped: the node would stand with
/// no text behind it, and "which row did this come from" is the question the restoration
/// exists to answer.
pub(super) fn Trace_Row(
    store: &mut SpecificationStore,
    at: RowAt,
    member: &Member,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let found = store.Table_Row_Uid(at.document_uid, at.block_ordinal, at.row_ordinal)?;
    let Some(row_uid) = found
    else
    {
        return Err(IngestError::Parse(format!(
            "{} block {} row {} is not in the store, so {} would trace to nothing",
            member.document, at.block_ordinal, at.row_ordinal, member.id
        )));
    };
    store.Put_Row_Lineage(row_uid, "preserved-verbatim", Some(node_uid))?;

    return Ok(());
}

/// Points an alias at the node, unless the name is contested.
///
/// A name more than one restored member claims is left unclaimed, and one another authority
/// already owns is reported and left where it is — a restoration may not repoint a name it
/// did not mint.
pub(super) fn Claim_Alias(
    store: &mut SpecificationStore,
    member: &Member,
    node_uid: i64,
    report: &mut RestorationReport,
) -> Result<(), IngestError>
{
    let Some(alias) = &member.alias
    else
    {
        return Ok(());
    };
    if report.ambiguous_names.contains(alias)
    {
        return Ok(());
    }

    if !Alias_Resolves_To(store, alias, node_uid)?
    {
        report.contested_aliases.push(alias.clone());
    }

    return Ok(());
}

/// Names claimed by more than one restored member.
///
/// Reported and withheld rather than resolved by a rule such as "the domain model wins".
/// Two nodes really do carry the name; picking one is an answer the corpus does not give.
pub(super) fn Ambiguous_Names(members: &[Member]) -> Vec<String>
{
    let mut claims: BTreeMap<&str, u32> = BTreeMap::new();
    for alias in members.iter().filter_map(|member| return member.alias.as_deref())
    {
        let counter = claims.entry(alias).or_insert(0);
        *counter = counter.saturating_add(1);
    }

    return claims
        .into_iter()
        .filter(|(_, claimed)| return *claimed > 1)
        .map(|(alias, _)| return alias.to_owned())
        .collect();
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
                    "{document} is not in the store at {revision}, so there is nothing for a \
                     restored node to trace back to"
                ));
            }

            return IngestError::Parse(format!(
                "the store could not be asked for {document} at {revision}: {cause}"
            ));
        });
}

pub(super) fn Dispose_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let disposed = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, node_uid],
    );
    Sql_Result(disposed)?;

    return Ok(());
}

/// Whether the alias now resolves to this node.
///
/// `false` where something else already owns it. Reported rather than ignored: an alias
/// silently pointing at another node makes "resolve by name" answer confidently and
/// wrongly, which is worse than not resolving at all.
pub(super) fn Alias_Resolves_To(store: &SpecificationStore, alias: &str, node_uid: i64) -> Result<bool, IngestError>
{
    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
        rusqlite::params![alias, node_uid],
    );
    Sql_Result(inserted)?;

    let selected_owner = store.Connection().query_row(
        "SELECT node_uid FROM node_aliases WHERE alias = ?1",
        rusqlite::params![alias],
        |row| row.get(0),
    );
    let owner: i64 = Sql_Result(selected_owner)?;

    return Ok(owner == node_uid);
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
    fn Test_Trace_Member_Should_Dispatch_By_Origin_Kind()
    {
        let LocatedMemberWithNode { mut store, document_uid, member: row_member, node_uid: node_for_row } =
            A_Located_Member_With_Its_Node();

        Trace_Member(&mut store, document_uid, &row_member, node_for_row).expect("traces the row");

        let traced_row: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage WHERE source_table_row_uid IS NOT NULL AND target_node_uid = ?1",
                rusqlite::params![node_for_row],
                |row| row.get(0),
            )
            .expect("counts");
        assert_eq!(traced_row, 1);

        let block_member = Member {
            origin: Origin::Block { ordinal: 1 },
            ..row_member
        };
        let node_for_block = store
            .Upsert_Node(NodeRow {
                node_id: "APX-D-TEST",
                kind: "schema",
                authority: "canonical",
                representation: "record",
                title: "Test",
            })
            .expect("mints");

        Trace_Member(&mut store, document_uid, &block_member, node_for_block).expect("traces the block");

        let traced_block: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage WHERE source_block_uid IS NOT NULL AND target_node_uid = ?1",
                rusqlite::params![node_for_block],
                |row| row.get(0),
            )
            .expect("counts");
        assert_eq!(traced_block, 1);
    }

    #[test]
    fn Test_Trace_Row_Should_Point_The_Row_At_Its_Node_Or_Refuse_A_Missing_One()
    {
        let Core { mut store, documents } = Store_With_Core();
        let located = Located_Members(&mut store, "v14.36", &documents).expect("locates");
        let (document_uid, member) = located.into_iter().next().expect("one member");
        let Origin::Row {
            block_ordinal,
            row_ordinal,
        } = member.origin
        else
        {
            panic!("the domain model member must trace to a row");
        };
        let node_uid = store
            .Upsert_Node(NodeRow {
                node_id: &member.id,
                kind: "concept",
                authority: "canonical",
                representation: "record",
                title: &member.name,
            })
            .expect("mints");
        let at = RowAt {
            document_uid,
            block_ordinal,
            row_ordinal,
        };

        Trace_Row(&mut store, at, &member, node_uid).expect("traces");

        let row_uid = store
            .Table_Row_Uid(document_uid, block_ordinal, row_ordinal)
            .expect("looks up")
            .expect("row exists");
        let traced: Option<i64> = store
            .Connection()
            .query_row(
                "SELECT target_node_uid FROM lineage WHERE source_table_row_uid = ?1",
                rusqlite::params![row_uid],
                |row| row.get(0),
            )
            .expect("reads lineage");
        assert_eq!(traced, Some(node_uid));

        let missing = RowAt {
            document_uid,
            block_ordinal,
            row_ordinal: row_ordinal + 100,
        };
        let refusal =
            Trace_Row(&mut store, missing, &member, node_uid).expect_err("must refuse a row that is not in the store");
        assert!(format!("{refusal}").contains("is not in the store"), "{refusal}");
    }

    #[test]
    fn Test_Claim_Alias_Should_Point_The_Alias_At_The_Node_Unless_It_Is_Ambiguous()
    {
        let LocatedMemberWithNode { mut store, member, node_uid, .. } = A_Located_Member_With_Its_Node();
        let alias = member.alias.clone().expect("the domain model row carries an alias");

        let mut ambiguous_report = RestorationReport {
            ambiguous_names: vec![alias.clone()],
            ..RestorationReport::default()
        };
        Claim_Alias(&mut store, &member, node_uid, &mut ambiguous_report).expect("claims");
        assert!(ambiguous_report.contested_aliases.is_empty());
        let claimed: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM node_aliases WHERE alias = ?1",
                rusqlite::params![&alias],
                |row| row.get(0),
            )
            .expect("counts");
        assert_eq!(claimed, 0, "an ambiguous alias must not be claimed");

        let mut report = RestorationReport::default();
        Claim_Alias(&mut store, &member, node_uid, &mut report).expect("claims");
        assert!(report.contested_aliases.is_empty());
        let owner: i64 = store
            .Connection()
            .query_row(
                "SELECT node_uid FROM node_aliases WHERE alias = ?1",
                rusqlite::params![&alias],
                |row| row.get(0),
            )
            .expect("reads the alias");
        assert_eq!(owner, node_uid);
    }

    #[test]
    fn Test_Ambiguous_Names_Should_Count_Aliases_Claimed_By_More_Than_One_Member()
    {
        use crate::Restored;

        let shared = Member {
            id: "CDM-X".to_owned(),
            family: Restored::CanonicalDomainModel,
            name: "X".to_owned(),
            document: "02-core.md".to_owned(),
            origin: Origin::Row {
                block_ordinal: 1,
                row_ordinal: 1,
            },
            alias: Some("Shared".to_owned()),
        };
        let other = Member {
            id: "GLS-SHARED".to_owned(),
            alias: Some("Shared".to_owned()),
            ..shared.clone()
        };
        let unique = Member {
            id: "CDM-Y".to_owned(),
            name: "Y".to_owned(),
            alias: Some("Unique".to_owned()),
            ..shared.clone()
        };

        let ambiguous = Ambiguous_Names(&[shared, other, unique]);

        assert_eq!(ambiguous, vec!["Shared".to_owned()]);
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
    fn Test_Dispose_Block_Should_Trace_A_Whole_Block_To_One_Node()
    {
        let Core { mut store, documents: _documents } = Store_With_Core();
        let document_uid = Document_Uid(&store, DocumentRevision("v14.36"), DocumentPath("02-core.md")).expect("finds");
        let node_uid = store
            .Upsert_Node(NodeRow {
                node_id: "APX-D-TEST",
                kind: "schema",
                authority: "canonical",
                representation: "record",
                title: "Test",
            })
            .expect("mints");

        Dispose_Block(&mut store, document_uid, 1, node_uid).expect("disposes");

        let traced: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage l JOIN source_blocks b ON b.uid = l.source_block_uid \
                 WHERE b.document_uid = ?1 AND b.ordinal = 1 AND l.target_node_uid = ?2",
                rusqlite::params![document_uid, node_uid],
                |row| row.get(0),
            )
            .expect("counts");
        assert_eq!(traced, 1);
    }

    #[test]
    fn Test_Alias_Resolves_To_Should_Report_False_When_Another_Node_Already_Owns_It()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let first = store
            .Upsert_Node(NodeRow {
                node_id: "CDM-A",
                kind: "concept",
                authority: "canonical",
                representation: "record",
                title: "A",
            })
            .expect("mints");
        let second = store
            .Upsert_Node(NodeRow {
                node_id: "CDM-B",
                kind: "concept",
                authority: "canonical",
                representation: "record",
                title: "B",
            })
            .expect("mints");

        assert!(Alias_Resolves_To(&store, "Shared", first).expect("resolves"));
        assert!(
            !Alias_Resolves_To(&store, "Shared", second).expect("resolves"),
            "a second node was allowed to take a claimed alias"
        );
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

    const CORE_MARKDOWN: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                                 | Model | Responsibility |\n| --- | --- |\n\
                                 | WorkspaceContext | Repository. |\n";

    /// A store already holding one ingested document, paired with the document set that produced
    /// it — named rather than a bare tuple so a caller cannot swap the two.
    struct Core
    {
        store: SpecificationStore,
        documents: BTreeMap<String, String>,
    }

    /// A store already holding one ingested document, and the document set that produced it.
    fn Store_With_Core() -> Core
    {
        use crate::Ingest_Source_Document;

        let mut store = SpecificationStore::In_Memory().expect("opens");
        Ingest_Source_Document(&mut store, "02-core.md", "v14.36", CORE_MARKDOWN).expect("ingests");

        let mut documents = BTreeMap::new();
        documents.insert("02-core.md".to_owned(), CORE_MARKDOWN.to_owned());

        return Core { store, documents };
    }

    /// A store holding one located member, minted as a node of its own, and the identity of
    /// both — named rather than a tuple so `document_uid` and `node_uid` cannot be swapped.
    struct LocatedMemberWithNode
    {
        store: SpecificationStore,
        document_uid: i64,
        member: Member,
        node_uid: i64,
    }

    /// A store holding one located member, minted as a node of its own — the setup every test
    /// that traces or claims against a member's node shares.
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
            .expect("mints");

        return LocatedMemberWithNode { store, document_uid, member, node_uid };
    }
}
