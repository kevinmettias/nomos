//! The records governing this system must be in the store before the validator they
//! govern is trusted.

use nomos_spec_store::{
    AUTHORED,
    EXTERNAL,
    GOVERNING_RECORD_IDS,
    NodeRow,
    SeedReport,
    Seed_Governing_Records,
    SpecificationStore,
    SuiteAuthority,
    Table,
};
use std::path::Path;

/// One counted answer, for a query that binds nothing.
fn Counted(store: &SpecificationStore, sql: &str) -> u32
{
    return store
        .Connection()
        .query_row(sql, [], |row| return row.get(0))
        .expect("queries");
}

/// One text answer, for a query naming one node as `?1`.
fn Column(store: &SpecificationStore, sql: &str, node_id: &str) -> String
{
    return store
        .Connection()
        .query_row(sql, rusqlite::params![node_id], |row| return row.get(0))
        .expect("queries");
}

fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds");
    return store;
}

/// The fewest governing records this build accepts.
///
/// A floor, not a count, and the difference is the whole of what `OD-SPEC-007` traded. It
/// is **not raised when a record is added** — adding one costs
/// `crates/spec/nomos-spec-store/records/<ID>.record` and nothing any other record writer
/// edits, which is the point. It is lowered by a deliberate removal.
///
/// What it still buys is the one thing nothing else here can see. A record file and its
/// registration deleted *together* pass both directions of
/// `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`, because the two sets still
/// agree — they agree about a record that is gone. The literal count that stood here caught
/// that by requiring the deleter to remember a third artifact, and this keeps exactly that
/// mechanism: a deletion has to be accompanied by lowering this number, and a deleter who
/// forgets goes red.
///
/// What is given up is the ceiling, and the cost is drift. The guarantee is exact only
/// while the count sits on the floor; once the count has risen above it, a coordinated
/// deletion inside the slack is caught by nothing here. An upper bound was considered and
/// rejected in `OD-SPEC-007`: it reintroduces the shared edit on an unpredictable schedule,
/// so instead of every record writer colliding, one unforeseeable record writer in every N
/// collides with all the others — which is worse to author against than one that fires
/// always. This number may be raised for free by any item that already has this file open
/// for another reason.
const FEWEST_GOVERNING_RECORDS: usize = 32;

/// Where a record's registration lives, from this crate's manifest directory.
fn Registration_Directory() -> std::path::PathBuf
{
    return std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("records");
}

/// Where the records themselves live.
fn Record_Directory() -> std::path::PathBuf
{
    return std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/records");
}

/// Every identifier under `docs/records` that claims canonical normative authority.
///
/// Side A of the comparison this file exists for, read at test time from the directory
/// records are authored in. Side B is `GOVERNING_RECORD_IDS`, assembled from a directory of
/// registration files somebody wrote by hand. The two sides are independently authored, and
/// that — not where either of them is spelled — is what makes the comparison a check.
fn Canonical_Records() -> Vec<String>
{
    let directory = Record_Directory();
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut canonical = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        let declared = Canonical_Id(&path);

        canonical.extend(declared);
    }

    return canonical;
}

/// The identifier a file declares, if it is a record claiming canonical normative authority.
///
/// # Panics
///
/// Panics if a file claims that authority and declares no `id:`. That is a record no
/// registration can name, which is the state this whole file exists to make visible.
fn Canonical_Id(path: &Path) -> Option<String>
{
    if path.extension().is_none_or(|extension| return extension != "md")
    {
        return None;
    }
    let Ok(text) = std::fs::read_to_string(path)
    else
    {
        return None;
    };
    if !text.contains("authority: canonical-normative-record")
    {
        return None;
    }
    let Some(id) = text.lines().find_map(|line| return line.strip_prefix("id: "))
    else
    {
        panic!("{} claims canonical authority and declares no id", path.display());
    };

    return Some(id.trim().to_owned());
}

/// The two directions of a set disagreement: on side A and not side B, and the reverse.
///
/// Named rather than a pair. Both members are `Vec<String>` and the compiler cannot tell
/// them apart, so a call site that swapped them would report an unseeded record as a
/// phantom one and still build.
struct Disagreement
{
    unseeded: Vec<String>,
    phantom: Vec<String>,
}

/// The comparison the guard makes, over two sets handed to it.
///
/// Extracted so that a control claiming the guard would have caught something is exercising
/// **the guard** rather than a second implementation of it written beside the first.
fn Disagreements(canonical: &[String], governing: &[&str]) -> Disagreement
{
    let unseeded: Vec<String> = canonical
        .iter()
        .filter(|id| return !governing.contains(&id.as_str()))
        .cloned()
        .collect();

    let phantom: Vec<String> = governing
        .iter()
        .filter(|id| return !canonical.iter().any(|found| return found == *id))
        .map(|id| return (*id).to_owned())
        .collect();

    return Disagreement { unseeded, phantom };
}

fn Title(store: &SpecificationStore, node_id: &str) -> Option<String>
{
    return store
        .Connection()
        .query_row(
            "SELECT title FROM nodes WHERE node_id = ?1",
            rusqlite::params![node_id],
            |row| row.get(0),
        )
        .ok();
}

#[test]
fn Test_Every_Governing_Record_Should_Resolve_By_Id()
{
    let store = Seeded();

    let missing: Vec<&str> = GOVERNING_RECORD_IDS
        .iter()
        .filter(|id| store.Node_Uid(id).expect("queries").is_none())
        .copied()
        .collect();

    assert!(missing.is_empty(), "not in the store: {missing:?}");

    // A floor rather than a count. What that keeps, what it gives up, and why there is no
    // ceiling are on `FEWEST_GOVERNING_RECORDS` above.
    assert!(
        GOVERNING_RECORD_IDS.len() >= FEWEST_GOVERNING_RECORDS,
        "{} governing record(s), and this build accepts no fewer than \
         {FEWEST_GOVERNING_RECORDS}. A record left the governing set: restore its \
         registration under crates/spec/nomos-spec-store/records/, or lower the floor in \
         the same commit that removes it. Nothing else here can see a record and its \
         registration deleted together.",
        GOVERNING_RECORD_IDS.len()
    );
}

/// Every record on disk that claims to be canonical and normative is in the store.
///
/// The direction nothing checked. `GOVERNING_RECORD_IDS` was compared against a seeded store
/// and never against `docs/records`, so a record could be written, declare itself
/// `canonical-normative-record`, be cited in commits and other records, and never reach the
/// store — which is what happened to six of them, including `OD-STORE-001` and
/// `OD-ANALYSIS-001`, two records that decide how the store and the analysis kernel behave.
///
/// `D-129` says the store holds identity and markdown is an editing surface. A governing
/// record that exists only as a file is that decision failing in the one place it is easiest
/// to check.
#[test]
fn Test_Every_Canonical_Record_On_Disk_Should_Be_Governing()
{
    let canonical = Canonical_Records();
    assert!(
        !canonical.is_empty(),
        "no canonical record was found under {}. Every assertion here iterates over that \
         set, so an empty one passes having checked nothing",
        Record_Directory().display()
    );
    let Disagreement { unseeded, phantom } = Disagreements(&canonical, GOVERNING_RECORD_IDS);

    assert!(
        unseeded.is_empty(),
        "these records claim canonical normative authority and are not seeded into the \
         store: {unseeded:?}.\n\
         Add crates/spec/nomos-spec-store/records/<ID>.record naming the file. A governing \
         record that lives only as a file is D-129 failing where it is easiest to check."
    );

    assert!(
        phantom.is_empty(),
        "the store seeds these and no file under docs/records declares them: {phantom:?}"
    );
}

/// The control this whole arrangement is measured against: the guard did not go vacuous.
///
/// Runs the real comparison with the real canonical set and the real governing set **minus
/// one element**, and asserts the missing one is reported. That is what a record written
/// without its registration looks like from here, and it is exactly `OD-SPEC-005`'s defect:
/// six records were authored as files and never declared.
///
/// It is done over the sets rather than by deleting the file because side B is assembled at
/// compile time and no `#[test]` causes a rebuild. `OD-SPEC-007` records the one-off manual
/// check that closed that gap end to end, with its result, rather than a test claiming to
/// have done it. The complement is
/// `Test_The_Governing_List_Should_Be_The_Registration_Directory`, which pins side B to the
/// registration directory as it is on disk.
#[test]
fn Test_A_Record_Whose_Registration_Is_Missing_Should_Be_Unseeded()
{
    let canonical = Canonical_Records();
    let Some(dropped) = GOVERNING_RECORD_IDS.first()
    else
    {
        panic!("nothing governs this build, so this control has nothing to remove")
    };

    let governing: Vec<&str> = GOVERNING_RECORD_IDS
        .iter()
        .filter(|id| return *id != dropped)
        .copied()
        .collect();

    let Disagreement { unseeded, phantom } = Disagreements(&canonical, &governing);

    assert_eq!(
        unseeded,
        vec![(*dropped).to_owned()],
        "a canonical record with no registration was not reported as unseeded, so the \
         guard no longer sees the thing OD-SPEC-005 exists about"
    );
    assert!(phantom.is_empty(), "removing a registration invented a phantom: {phantom:?}");
}

/// The other direction: a registration nothing on disk declares.
#[test]
fn Test_A_Registration_With_No_Record_Should_Be_A_Phantom()
{
    let canonical = Canonical_Records();
    let invented = "OD-INVENTED-404";

    let mut governing: Vec<&str> = GOVERNING_RECORD_IDS.to_vec();
    governing.push(invented);

    let Disagreement { unseeded, phantom } = Disagreements(&canonical, &governing);

    assert_eq!(phantom, vec![invented.to_owned()]);
    assert!(unseeded.is_empty(), "inventing a registration unseeded a record: {unseeded:?}");
}

/// `GOVERNING_RECORD_IDS` is the registration directory, and nothing else.
///
/// The vacuity boundary from the other side. `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`
/// compares side B against `docs/records`; this compares side B against the directory it is
/// supposed to have been assembled from, read at test time. Delete
/// `records/OD-FOO-001.record` and rebuild, and this goes red before that one does, naming
/// the file.
///
/// What it does **not** prove is that the build script reads `records/` rather than
/// `docs/records`: in a consistent tree both produce the same identifiers. That is guarded
/// structurally in `src/registration/mod.rs` and textually by
/// `Test_The_Build_Script_Should_Not_Enumerate_The_Record_Directory`.
#[test]
fn Test_The_Governing_List_Should_Be_The_Registration_Directory()
{
    let directory = Registration_Directory();
    let stems = Registered_Stems(&directory);
    assert!(
        !stems.is_empty(),
        "no registration was found under {}, so this compared nothing",
        directory.display()
    );
    let Disagreement {
        unseeded: unregistered,
        phantom: undeclared,
    } = Disagreements(&stems, GOVERNING_RECORD_IDS);

    assert!(
        unregistered.is_empty() && undeclared.is_empty(),
        "the governing list and the registration directory disagree. Registered and not \
         governing: {unregistered:?}. Governing and not registered: {undeclared:?}.\n\
         The list is generated from that directory, so a disagreement means the build ran \
         against a different one."
    );
}

/// Every registration stem in a directory, read at test time.
fn Registered_Stems(directory: &Path) -> Vec<String>
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut stems = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        let stem = Registration_Stem(&path);

        stems.extend(stem);
    }

    return stems;
}

/// The stem of a registration file, or `None` for anything else in the directory.
fn Registration_Stem(path: &Path) -> Option<String>
{
    if path.extension().is_none_or(|extension| return extension != "record")
    {
        return None;
    }

    return path
        .file_stem()
        .and_then(|stem| return stem.to_str())
        .map(str::to_owned);
}

/// What the refused derivation would check, exhibited rather than argued.
///
/// `OD-LEDGER-007` declined to derive `GOVERNING_RECORD_IDS` from `docs/records` because
/// the guard above would then compare that directory against itself. This is that
/// comparison, made with both sides taken from `docs/records` — including a record nobody
/// registered — and it comes back clean. It asserts nothing about the arrangement that is
/// actually in place; it exists so that the next person who proposes globbing `docs/records`
/// finds a test that already says what would happen.
#[test]
fn Test_Comparing_The_Directory_Against_Itself_Would_Check_Nothing()
{
    let mut would_be_canonical = Canonical_Records();
    would_be_canonical.push("OD-NOBODY-DECLARED-THIS-001".to_owned());

    // Side B, as the refused derivation would have produced it: read off the very directory
    // side A was read from.
    let would_be_governing: Vec<&str> = would_be_canonical
        .iter()
        .map(|id| return id.as_str())
        .collect();

    let Disagreement {
        unseeded: would_be_unseeded,
        phantom: would_be_phantom,
    } = Disagreements(&would_be_canonical, &would_be_governing);

    assert!(
        would_be_unseeded.is_empty() && would_be_phantom.is_empty(),
        "a directory compared against itself disagreed with itself, which would mean this \
         demonstration is no longer demonstrating the failure OD-LEDGER-007 refused"
    );
}

/// The build script's input directory is the registration directory.
///
/// **This is a textual check and it is nothing more than that.** It reads `build.rs` and
/// `src/registration/mod.rs` as text and asserts that no directory enumeration in either of
/// them is applied to a path built from `docs/records`. It cannot detect a rewrite that
/// reaches the same directory by another spelling, and it is not evidence about behaviour.
///
/// It exists because the regression it watches for — "simplifying" the generator into
/// globbing `docs/records`, the exact move `OD-LEDGER-007` refused — is invisible to every
/// other test in the tree: in a consistent tree the two directories yield the same
/// identifiers, so everything still passes while the guard checks nothing. The behavioural
/// half of the guard is `Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given` in
/// `src/registration/mod.rs`, which hands the reader a directory of one and catches it if it
/// returns more.
#[test]
fn Test_The_Build_Script_Should_Not_Enumerate_The_Record_Directory()
{
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    for file in ["build.rs", "src/registration/mod.rs"]
    {
        let path = crate_root.join(file);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        for (ordinal, after) in text.split("read_dir(").enumerate().skip(1)
        {
            let arguments = after.split(')').next().unwrap_or(after);

            assert!(
                !arguments.contains("docs/records") && !arguments.contains("RECORD_DIRECTORY"),
                "{file}: directory enumeration number {ordinal} is applied to \
                 `{arguments}`, which names the record directory. Deriving the governing \
                 list from docs/records makes \
                 Test_Every_Canonical_Record_On_Disk_Should_Be_Governing compare that \
                 directory against itself and pass having checked nothing — see \
                 OD-LEDGER-007 and OD-SPEC-007."
            );
        }
    }
}

/// The negative control. Without it the assertion above would pass on a store that
/// contains everything for some other reason.
#[test]
fn Test_An_Unseeded_Store_Should_Hold_None_Of_Them()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    for id in GOVERNING_RECORD_IDS
    {
        assert!(
            store.Node_Uid(id).expect("queries").is_none(),
            "{id} appeared in a store nobody seeded"
        );
    }
}

/// D-129 supersedes ADR-DOC-001. The edge has to be in the store, not only in the prose,
/// or "what superseded this?" is a question only a person reading markdown can answer.
#[test]
fn Test_Adr_Doc_001_Should_Carry_An_Explicit_Supersession_Edge()
{
    let store = Seeded();
    let forward = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         WHERE f.node_id = 'D-129' AND r.relation_type = 'supersedes'
           AND t.node_id = 'ADR-DOC-001'",
    );
    let inverse = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         WHERE f.node_id = 'ADR-DOC-001' AND r.relation_type = 'superseded_by'
           AND t.node_id = 'D-129'",
    );

    assert!(
        store.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the superseded record must exist for the edge to mean anything"
    );
    assert_eq!(forward, 1, "D-129 does not supersede ADR-DOC-001");
    assert_eq!(inverse, 1, "the inverse edge is missing, so the fact is only half recorded");
}

/// ADR-DOC-001 is in the store by identity, not by content — its prose is restored from
/// the v14 corpus in Phase 3. Recording that as a placeholder rather than inventing a
/// body is the difference between a gap and filler.
#[test]
fn Test_The_Superseded_Record_Should_Be_A_Visible_Placeholder()
{
    let store = Seeded();
    let authority = Column(&store, "SELECT authority FROM nodes WHERE node_id = ?1", "ADR-DOC-001");
    let bodies = Counted(
        &store,
        "SELECT count(*) FROM source_documents WHERE path LIKE '%ADR-DOC-001%'",
    );

    assert_eq!(authority, EXTERNAL);
    assert_eq!(
        bodies, 0,
        "a placeholder with a body would be filler wearing a record's clothes"
    );
}

/// A real record arriving later must fill the placeholder in rather than being ignored.
#[test]
fn Test_A_Real_Record_Should_Replace_A_Placeholder_But_Not_A_Real_One()
{
    let mut store = Seeded();
    store
        .Upsert_Node(NodeRow {
            node_id: "ADR-DOC-001",
            kind: "decision",
            authority: "canonical-normative-record",
            representation: "document",
            title: "Markdown is the canonical authored documentation format",
        })
        .expect("upgrades");
    store
        .Upsert_Node(NodeRow {
            node_id: "D-129",
            kind: "decision",
            authority: "commentary",
            representation: "document",
            title: "Something else",
        })
        .expect("attempts to restate");

    assert_eq!(
        Title(&store, "ADR-DOC-001").as_deref(),
        Some("Markdown is the canonical authored documentation format")
    );
    assert_ne!(
        Title(&store, "D-129").as_deref(),
        Some("Something else"),
        "a later pass overwrote a record its author already wrote"
    );
}

/// Re-ingesting a document must not renumber its blocks.
///
/// `uid` is what every lineage and omission row points at, so a write that deletes the
/// block row and inserts a replacement takes the dispositions with it. Seeding twice is
/// what exposed this; the assertion is on the uids rather than on the counts because a
/// count survives a renumbering that destroys every reference.
#[test]
fn Test_Rewriting_A_Document_Should_Not_Renumber_Its_Blocks()
{
    let mut store = Seeded();
    let before = Block_Uids(&store);

    Seed_Governing_Records(&mut store).expect("seeds again");

    assert!(!before.is_empty(), "no blocks, so this test proved nothing");
    assert_eq!(before, Block_Uids(&store), "the blocks were reinserted under new uids");
}

fn Block_Uids(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_blocks ORDER BY document_uid, ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// Seeding twice is seeding once.
#[test]
fn Test_Seeding_Should_Be_Idempotent()
{
    let mut store = Seeded();
    let before: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    Seed_Governing_Records(&mut store).expect("seeds again");

    let after: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    assert_eq!(before, after);
}

/// The records are present as content, not only as identity. Without blocks there is
/// nothing for the preservation rules to examine and every one of them reports clean
/// having looked at nothing.
///
/// The count comparison here has one subject and it is the one the message names: the store
/// made one source document per governing record. It used to have a second, incidental one
/// — `RECORDS` and `GOVERNING_RECORD_IDS` were two lists kept by hand and this caught them
/// disagreeing in length. They are now generated from one directory and cannot, so that
/// subject is impossible by construction rather than checked. The remaining hazard, two
/// registrations naming one record file, is a refusal in the reader.
#[test]
fn Test_The_Records_Should_Be_Present_As_Disposed_Content()
{
    let store = Seeded();
    let empty = Counted(
        &store,
        "SELECT count(*) FROM source_documents d
         WHERE NOT EXISTS (SELECT 1 FROM source_blocks b WHERE b.document_uid = d.uid)",
    );
    let undisposed = Counted(
        &store,
        "SELECT count(*) FROM source_blocks b
         WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
    );

    assert_eq!(
        store.Count(Table::SourceDocuments).expect("counts") as usize,
        GOVERNING_RECORD_IDS.len(),
        "one source document per governing record, or a record reached the store as an \
         identity with no content behind it"
    );
    assert!(store.Count(Table::SourceBlocks).expect("counts") >= 40);
    assert!(store.Count(Table::SourceHeadings).expect("counts") >= 16);
    assert_eq!(empty, 0, "{empty} record(s) segmented to nothing");
    assert_eq!(undisposed, 0, "{undisposed} seeded block(s) have no disposition");
}

#[test]
fn Test_Authored_Content_Should_Be_Distinguishable_From_An_Ingested_Revision()
{
    let store = Seeded();
    let revisions = Counted(
        &store,
        &format!("SELECT count(*) FROM source_documents WHERE revision <> '{AUTHORED}'"),
    );

    assert_eq!(revisions, 0, "a governing record claims to come from a corpus revision");
}

/// A carriage return in an embedded record would change every hash it produces, so the
/// same commit would validate on one machine and not on another. `.gitattributes` pins
/// it; this is what checks the pin held.
#[test]
fn Test_No_Governing_Record_Should_Carry_A_Carriage_Return()
{
    let store = Seeded();
    let offenders = Blocks_Carrying_A_Carriage_Return(&store);

    assert!(offenders.is_empty(), "CRLF reached the store from {offenders:?}");
    assert!(
        store.Count(Table::SourceBlocks).expect("counts") > 0,
        "no blocks were examined, so this found nothing by looking at nothing"
    );
}

/// The documents holding a block with a carriage return in it.
fn Blocks_Carrying_A_Carriage_Return(store: &SpecificationStore) -> Vec<String>
{
    let mut statement = store
        .Connection()
        .prepare("SELECT path, text FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid")
        .expect("prepares");

    return statement
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let text: String = row.get(1)?;

            return Ok((path, text));
        })
        .expect("queries")
        .filter_map(Result::ok)
        .filter(|(_, text)| return text.contains('\r'))
        .map(|(path, _)| return path)
        .collect();
}

/// The sibling records `ARC-ECOSYSTEM-001` cites, named here rather than counted.
///
/// `D-085`, `D-086`, `D-090` and `D-096` are the decisions it restates and `D-122` is the
/// one it adopts. None of them is this repository's: they belong to `xvpe-spec-seed` and
/// `ecosystem-contracts`, which `P3-SIBLINGS` ingests as non-root suites. Over a store
/// holding only this repository's records they are therefore targets nothing has ingested.
const CITED_SIBLING_RECORDS: &[&str] = &["D-085", "D-086", "D-090", "D-096", "D-122"];

/// A relation across the seam must arrive as a reported placeholder, not as a dropped edge.
///
/// `Write_Relation` inserts by selecting both endpoints out of `nodes`, so an edge to an
/// identifier no node holds matches nothing and writes nothing — silently, at exit 0. What
/// stops that here is the second pass in `Seed_Governing_Records`, which mints the reference
/// first and reports it by name. This asserts the reporting, because a growing list of
/// unresolved targets is the only thing that distinguishes an edge into another suite from
/// an edge into a typo.
#[test]
fn Test_A_Relation_To_A_Sibling_Record_Should_Arrive_As_A_Reported_Placeholder()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let report = Seed_Governing_Records(&mut store).expect("seeds");

    for record in CITED_SIBLING_RECORDS
    {
        Assert_Arrived_As_A_Placeholder(&store, &report, record);
    }
}

/// One cited sibling record: named in the report, minted as external, claiming no suite.
fn Assert_Arrived_As_A_Placeholder(store: &SpecificationStore, report: &SeedReport, record: &str)
{
    assert!(
        report.references.iter().any(|reference| return reference == record),
        "{record} is cited across the seam and the seed report does not name it, so an edge \
         into a sibling suite is indistinguishable from an edge into nothing"
    );

    let authority = Column(store, "SELECT authority FROM nodes WHERE node_id = ?1", record);

    assert_eq!(authority, EXTERNAL, "{record}");
    assert_eq!(
        store.Suite_Of(record).expect("queries"),
        None,
        "{record} claims a suite in a store no suite was ingested into"
    );
}

/// The other half: a placeholder is the sibling's own node once the sibling claims it.
///
/// `Ingest_Sibling_Suite` upserts each record under its declared identifier and assigns it
/// the suite, and that only lands because `Write_Node` updates a node in place where its
/// authority is external. The two halves live in different crates and neither states the
/// property they compose to, so this is where "which suite decided this first" stops being a
/// reading of two functions and becomes a query.
///
/// The suite is created here rather than ingested from an archive on purpose: the archives
/// are outside this repository and a test that cannot find its corpus passes. This one has
/// no corpus to miss.
#[test]
fn Test_A_Placeholder_Should_Become_The_Node_Of_The_Suite_That_Claims_It()
{
    let store = Claimed_By_The_Sibling_Suite();
    let edges = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         JOIN suites s ON s.uid = t.suite_uid
         WHERE f.node_id = 'ARC-ECOSYSTEM-001' AND r.relation_type = 'relates-to'
           AND t.node_id = 'D-090' AND s.authority_root = 0",
    );

    assert_eq!(
        store.Suite_Of("D-090").expect("queries"),
        Some(("xvpe-spec-seed".to_owned(), SuiteAuthority::Sibling)),
        "D-090 resolved to no sibling suite, so the edge to it says nothing about whose \
         decision it is"
    );
    assert_eq!(
        Title(&store, "D-090").as_deref(),
        Some("Reuse alone does not justify platform ownership"),
        "the placeholder kept its own name, so the sibling's record never arrived"
    );
    assert_eq!(
        edges, 1,
        "the cross-suite edge does not join a non-root suite, which is the whole distinction \
         P3-SIBLINGS bought"
    );
}

/// A seeded store in which the sibling suite has claimed the `D-090` placeholder as its own.
///
/// The suite is created here rather than ingested from an archive on purpose: the archives
/// are outside this repository and a test that cannot find its corpus passes. This one has
/// no corpus to miss.
fn Claimed_By_The_Sibling_Suite() -> SpecificationStore
{
    let mut store = Seeded();
    let suite = store
        .Put_Suite("xvpe-spec-seed", "XVPE specification seed", SuiteAuthority::Sibling)
        .expect("records the sibling suite");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "D-090",
            kind: "decision",
            authority: "canonical-normative-record",
            representation: "document",
            title: "Reuse alone does not justify platform ownership",
        })
        .expect("claims the placeholder");

    store.Assign_Suite(node, suite).expect("assigns");

    return store;
}
