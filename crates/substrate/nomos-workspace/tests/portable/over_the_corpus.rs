//! Every assertion this suite makes against the real Rust corpus.
//!
//! All five corpus-gated tests are here, beside [`Scale_Corpus_Or_Skip`], and that is a
//! requirement rather than a tidy arrangement. `tests/contract/src/gates.rs` resolves a test
//! to the corpora it reaches by following the calls it makes **within one file** — none of
//! these tests names `NOMOS_RUST_CORPUS` itself — so a gated test moved to a sibling module
//! would stop being counted, and the declared size of the hole in
//! `tests/contract/tests/corpus_gates.rs` would drop without any assertion being removed.

use crate::arrival::{Configuration, Fresh, Ingest, Variant, Workspace};
use crate::walk::{Assert_This_Is_That_Corpus, Corpus};
use nomos_store::{Authority, DocumentKind, DocumentStore};
use nomos_workspace::{Applied, WorkspaceSnapshot};
use std::path::{Path, PathBuf};

const SCALE_CORPUS: &str = "F:/repos/xvpe";

fn Scale_Corpus_Or_Skip() -> Option<(PathBuf, Vec<(String, String)>)>
{
    let configured = std::env::var_os("NOMOS_RUST_CORPUS");
    let root = configured
        .clone()
        .map_or_else(|| return PathBuf::from(SCALE_CORPUS), PathBuf::from);
    if !root.is_dir()
    {
        Report_The_Absence(configured.is_some(), &root);

        return None;
    }
    let members = Corpus(&root);

    Assert_This_Is_That_Corpus(&members, &root);

    return Some((root, members));
}

/// A corpus the environment named and this machine cannot read is a failure. An unconfigured
/// one that is simply not here is a skip, said out loud so a green run is not read as a
/// checked one.
fn Report_The_Absence(configured: bool, root: &Path)
{
    assert!(
        !configured,
        "NOMOS_RUST_CORPUS is set to {}, which is not a directory. A configured corpus \
         that cannot be read is a failure, not a skip",
        root.display()
    );
    eprintln!("skipped: no corpus at {}", root.display());
}

// ---------------------------------------------------------------------------------
// Portability
// ---------------------------------------------------------------------------------

/// The property, stated the way it actually fails.
///
/// A snapshot becomes unportable by recording where it was taken — an absolute path, a
/// drive letter, the corpus root as a prefix. So the assertion looks for exactly that in
/// the bytes, over a corpus rooted at a Windows drive, which is the case most likely to
/// leak one.
#[test]
fn Test_A_Snapshot_Of_The_Real_Corpus_Should_Name_Nothing_Outside_Itself()
{
    let Some((root, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };
    let mut workspace = Fresh();
    Ingest(&mut workspace, &members, 512);
    let encoded = workspace.Snapshot().Encode();

    eprintln!(
        "{}: {} members, {} bytes of snapshot, identity {}",
        root.display(),
        workspace.Snapshot().Length(),
        encoded.len(),
        workspace.Id()
    );
    assert_eq!(
        workspace.Snapshot().Length(),
        members.len(),
        "every file in the corpus is a member"
    );
    Assert_Names_Nothing_Outside_Itself(&encoded, &root);
}

/// The three ways a snapshot records where it was taken: the corpus root, a drive letter, and
/// a member path that is not workspace-relative.
fn Assert_Names_Nothing_Outside_Itself(encoded: &[u8], root: &Path)
{
    let text = String::from_utf8(encoded.to_vec()).expect("the encoding is UTF-8");
    let root_text = root.to_string_lossy().replace('\\', "/").to_lowercase();

    assert!(
        !text.contains(&root_text),
        "the snapshot records {root_text}, so it describes where it was taken rather than \
         what was there"
    );
    assert!(
        !text.contains("f:/") && !text.contains("c:/"),
        "the snapshot carries a drive letter"
    );
    for line in text.lines().filter(|line| return line.starts_with("member\t"))
    {
        let path = line.split('\t').nth(1).unwrap_or_default();
        assert!(
            !path.starts_with('/') && !path.contains(".."),
            "`{path}` is not workspace-relative"
        );
    }
}

/// Interpretable by a process with no access to that tree.
///
/// The tree is not merely unused here — it is unreachable. The decode takes bytes and
/// nothing else, and every query afterwards is answered from what it decoded. If the
/// snapshot needed the corpus for anything, this would be where it found out.
#[test]
fn Test_A_Snapshot_Should_Be_Interpretable_Without_The_Tree()
{
    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut workspace = Fresh();
    Ingest(&mut workspace, &members, 512);
    let encoded = workspace.Snapshot().Encode();
    let elsewhere = WorkspaceSnapshot::Decode(&encoded).expect("bytes are all it needs");

    Assert_Decoded_Answers_As_The_Original(&elsewhere, &workspace, &encoded);
    Assert_Every_Member_Survived(&elsewhere, &workspace);
}

/// Every question the workspace answers, asked of the copy that came from bytes alone.
fn Assert_Decoded_Answers_As_The_Original(
    elsewhere: &WorkspaceSnapshot,
    workspace: &Workspace,
    encoded: &[u8],
)
{
    assert_eq!(elsewhere.Id(), workspace.Id());
    assert_eq!(elsewhere.Length(), workspace.Snapshot().Length());
    assert_eq!(elsewhere.Variant(), &Variant());
    assert_eq!(elsewhere.Configuration(), Configuration());
    assert_eq!(
        elsewhere.Encode(),
        encoded,
        "and re-encoding what it read is byte-identical, so the reading lost nothing"
    );
}

/// Every member, answered from the decoded bytes. Not a sample: the claim is about the whole
/// snapshot, and a spot check would pass for a decoder that dropped a suffix.
fn Assert_Every_Member_Survived(elsewhere: &WorkspaceSnapshot, workspace: &Workspace)
{
    let here = workspace.Snapshot().Members();
    let there = elsewhere.Members();

    assert_eq!(here, there, "every member survives the crossing");

    // The positive control. If `Members` returned nothing, every assertion above would
    // hold over two empty snapshots.
    assert!(
        here.len() >= 5_000,
        "{} members is not this corpus",
        here.len()
    );
}

/// The store is the other half of "no access to that tree": a document store holding the
/// snapshot can be read by something that never saw the corpus.
#[test]
fn Test_A_Recorded_Snapshot_Should_Be_Readable_From_The_Store_Alone()
{
    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut workspace = Fresh();
    Ingest(&mut workspace, &members, 512);
    let mut store = DocumentStore::For(Workspace::Authority());
    workspace.Record(&mut store).expect("the store admits an observed measurement");

    let from_store = The_One_Recorded_Snapshot(&store);

    assert_eq!(from_store.Id(), workspace.Id());
    assert_eq!(from_store.Length(), members.len());
    Assert_The_Index_Still_Derives(&mut store, &workspace);
}

/// The workspace state read back out of the store by something that never saw the corpus.
fn The_One_Recorded_Snapshot(store: &DocumentStore) -> WorkspaceSnapshot
{
    let recorded: Vec<Vec<u8>> = store
        .Documents()
        .values()
        .filter(|document| return document.kind == DocumentKind::Fact)
        .map(|document| return document.bytes.clone())
        .collect();

    assert_eq!(recorded.len(), 1, "one workspace state was recorded");

    return recorded
        .first()
        .map(|bytes| return WorkspaceSnapshot::Decode(bytes))
        .expect("the document is there")
        .expect("and it is a workspace snapshot");
}

/// The index has to survive it. A workspace state filed under `DocumentKind::Commit` would be
/// decoded as a commit manifest and break the index for every document in the store — which is
/// the behaviour that decides a kind, and the reason a workspace state does not get one of its
/// own. OD-STORE-001.
///
/// And the commit it arrived in is reachable under the state it was taken against, which is
/// what makes "read the store, find the workspace" a question the index can answer without
/// knowing this crate's schema string.
fn Assert_The_Index_Still_Derives(store: &mut DocumentStore, workspace: &Workspace)
{
    assert!(store.Index().is_ok(), "the store's index still derives");
    assert_eq!(
        store.Authority(),
        Authority::Observed,
        "a measurement of a tree is observed, not authored"
    );
    assert_eq!(
        store.Index().expect("indexes").Commits_Under(workspace.Id()).len(),
        1,
        "the workspace's own commit is not filed under the state it recorded"
    );
}

// ---------------------------------------------------------------------------------
// Ingestion order
// ---------------------------------------------------------------------------------

/// 100 permutations, byte-identical.
///
/// The corpus arrives in 100 different orders, batched into change sets that straddle
/// different files each time, and every run must produce the same snapshot bytes, the same
/// identity, and the same answer to every member query.
///
/// The permutation is a deterministic shuffle rather than a random one. A random order
/// that failed once would be a failure nobody could reproduce, and this test exists to
/// produce a reproducible one. [`crate::permutation`] holds the shuffle and the proof that
/// it is one.
///
/// # What this is evidence for
///
/// [`nomos_workspace::SnapshotSerialization`], and specifically its
/// [`nomos_contracts::DeterminismStrength::State`] half — a hundred arrival orders
/// reaching one encoding is exactly the claim that the sequence inputs arrived in cannot
/// reach the output. The declaration did not exist when this was written; the property
/// did, which is the gap `P9-DETERMINISM` closed.
///
/// Gated on the scale corpus, so it does not run in CI. The same declaration is checked
/// over an in-repository fixture in `tests/integration/tests/determinism/`, including
/// the `CrossBinary` half that this test does not reach at all — a hundred permutations
/// inside one process say nothing about what a different build of this analyzer would
/// encode.
#[test]
fn Test_A_Hundred_Ingestion_Orders_Should_Yield_Byte_Identical_Queries()
{
    use crate::permutation::Taken;

    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let Taken { bytes, members, .. } = Permuted_A_Hundred_Ways(&members);

    eprintln!("permutations: 100 orders, {members} members, {} identical bytes", bytes.len());

    // The vacuity guard. If the corpus had one member — or none — every permutation would
    // be the same permutation and this would have asserted nothing.
    assert!(
        members >= 5_000,
        "{members} members cannot meaningfully be permuted a hundred ways"
    );
}

/// Snapshots `members` a hundred times, in a hundred deterministic orders, asserting each
/// against the first as it goes. Returns the first snapshot taken, once all hundred agreed
/// with it.
fn Permuted_A_Hundred_Ways(members: &[(String, String)]) -> crate::permutation::Taken
{
    use crate::permutation::{Snapshot_Of_One_Order, Taken};

    let mut baseline: Option<Taken> = None;
    for permutation in 0..100_u32
    {
        let taken = Snapshot_Of_One_Order(members, permutation);

        crate::permutation::Assert_Same_As_The_First(&mut baseline, taken, permutation);
    }

    return baseline.expect("a hundred permutations ran");
}

// ---------------------------------------------------------------------------------
// The door, over the real corpus
// ---------------------------------------------------------------------------------

/// Re-ingesting an unchanged corpus advances nothing.
///
/// The workspace-level form of the property the fact store makes at the fact level: a
/// checkout that lands where you already were is not a change, and treating it as one
/// would invalidate every fact in the store to arrive back at the same answer.
#[test]
fn Test_Re_Ingesting_The_Corpus_Should_Advance_Nothing()
{
    use nomos_contracts::GenerationId;

    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let outcome = Ingested_Twice(&members);

    assert_eq!(outcome.after_second, outcome.after_first);
    assert_eq!(outcome.identity_after_second, outcome.identity);
    assert_eq!(outcome.redundant, members.len());
    // The positive control. The first ingestion must have advanced, or "nothing advanced"
    // is a statement about a workspace that never did anything.
    assert!(
        outcome.after_first > GenerationId::INITIAL,
        "the first ingestion advanced nothing either"
    );
}

/// What a fresh workspace's generation and identity were after the corpus was ingested
/// once, and what they became after [`Re_Ingest`] applied the same corpus a second time.
struct ReIngestion
{
    after_first: nomos_contracts::GenerationId,
    identity: nomos_contracts::SnapshotId,
    after_second: nomos_contracts::GenerationId,
    identity_after_second: nomos_contracts::SnapshotId,
    redundant: usize,
}

fn Ingested_Twice(members: &[(String, String)]) -> ReIngestion
{
    let mut workspace = Fresh();
    Ingest(&mut workspace, members, 512);

    let after_first = workspace.Generation();
    let identity = workspace.Id();
    let redundant = Re_Ingest(&mut workspace, members);

    return ReIngestion {
        after_first,
        identity,
        after_second: workspace.Generation(),
        identity_after_second: workspace.Id(),
        redundant,
    };
}

/// The same corpus applied a second time, and how many paths came back redundant.
fn Re_Ingest(workspace: &mut Workspace, members: &[(String, String)]) -> usize
{
    let mut redundant = 0_usize;
    for batch in members.chunks(512)
    {
        let effects = Apply_Again(workspace, batch);

        redundant = redundant.saturating_add(effects);
    }

    eprintln!(
        "re-ingestion: {redundant} paths, all redundant, generation still {}",
        workspace.Generation().Raw()
    );

    return redundant;
}

/// One batch of what is already there, which must land as `Applied::Unchanged`.
fn Apply_Again(workspace: &mut Workspace, batch: &[(String, String)]) -> usize
{
    let set = crate::arrival::Change_Set(batch);
    let applied = workspace.Apply(&set).expect("applies");

    assert!(
        matches!(applied, Applied::Unchanged { .. }),
        "a checkout of what is already there is not a change"
    );

    return applied.Effects().len();
}
