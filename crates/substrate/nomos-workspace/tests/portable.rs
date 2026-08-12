//! The two properties the item is done when.
//!
//! A portable snapshot over `F:/repos/xvpe` interpretable by a process with no access to
//! that tree, and 100 ingestion-order permutations that yield byte-identical queries.
//!
//! # Opt-in by path, loud when configured and unreadable
//!
//! `NOMOS_RUST_CORPUS` overrides the root. Set and unreadable is a failure; absent and
//! unconfigured returns having asserted nothing and said so. The same bargain
//! `nomos-spec-model`'s `normalizer_gate` strikes with `NOMOS_V14_CORPUS`.

use nomos_contracts::{ConfigurationId, Digest128, GenerationId};
use nomos_store::{Authority, DocumentKind, DocumentStore};
use nomos_workspace::{
    Applied, BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet, WorkspaceSnapshot,
};
use std::path::{Path, PathBuf};

const SCALE_CORPUS: &str = "F:/repos/xvpe";
const NOT_SOURCE: &[&str] = &["target", ".git"];

fn Variant() -> BuildVariant
{
    return BuildVariant::New(
        "x86_64-pc-windows-msvc",
        "dev",
        "1.85",
        ["analysis", "telemetry"],
    );
}

fn Configuration() -> ConfigurationId
{
    return ConfigurationId::From_Digest(Digest128::From_Bytes([0x2f; 16]));
}

fn Fresh() -> Workspace
{
    return Workspace::Empty(Variant(), Configuration());
}

/// Every Rust file under a root, as workspace-relative paths and contents.
///
/// Sorted, so a failure names the same file on two machines and the permutation test has a
/// stable baseline to permute away from.
fn Corpus(root: &Path) -> Vec<(String, String)>
{
    let mut paths = Rust_Files_Under(root);
    paths.sort();

    let mut members = Vec::new();
    for path in paths
    {
        // Read as bytes and render lossily rather than requiring UTF-8. A file this
        // workspace cannot decode is still a member of it, and skipping it would make the
        // snapshot describe a tree that is missing files nobody was told about.
        let Ok(bytes) = std::fs::read(&path)
        else
        {
            continue;
        };
        let relative = Relative_To(root, &path);
        members.push((relative, String::from_utf8_lossy(&bytes).into_owned()));
    }

    return members;
}

/// Every Rust file under a root, in whatever order the walk found them.
fn Rust_Files_Under(root: &Path) -> Vec<PathBuf>
{
    let mut paths = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };
        for entry in entries.flatten()
        {
            Visit(&entry.path(), &mut pending, &mut paths);
        }
    }

    return paths;
}

/// One entry: a source directory to descend into later, a Rust file to keep, or neither.
fn Visit(path: &Path, pending: &mut Vec<PathBuf>, paths: &mut Vec<PathBuf>)
{
    let Some(name) = path.file_name()
    else
    {
        return;
    };
    let name = name.to_string_lossy();
    let is_directory = path.is_dir();
    if is_directory && !NOT_SOURCE.contains(&name.as_ref())
    {
        pending.push(path.to_path_buf());
    }
    else if !is_directory && path.extension().is_some_and(|extension| return extension == "rs")
    {
        paths.push(path.to_path_buf());
    }
}

/// A path under the root, as the workspace-relative string a snapshot is allowed to record.
fn Relative_To(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
}

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

/// The guard against a walk that found a directory but almost nothing in it.
fn Assert_This_Is_That_Corpus(members: &[(String, String)], root: &Path)
{
    assert!(
        members.len() >= 5_000,
        "found {} Rust files under {}, which is not this corpus. Every assertion below \
         iterates over this set, so a truncated walk makes all of them pass having read \
         almost nothing",
        members.len(),
        root.display()
    );
}

/// Ingests a corpus in the order given, as one change set per batch of `stride` files.
///
/// Batched rather than one set per file, because that is what a real source does: a
/// checkout arrives as one event covering hundreds of paths. It also means the permutation
/// test permutes across change-set boundaries rather than only within one.
fn Ingest(workspace: &mut Workspace, members: &[(String, String)], stride: usize)
{
    for batch in members.chunks(stride.max(1))
    {
        let set = Change_Set(batch);

        workspace
            .Apply(&set)
            .expect("every path in the corpus is workspace-relative");
    }
}

/// One batch of files, as the single change set a checkout would arrive as.
fn Change_Set(batch: &[(String, String)]) -> WorkspaceChangeSet
{
    let mut set = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
    for (path, content) in batch
    {
        set = set.Present(path.clone(), content.clone());
    }

    return set;
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
        workspace.Snapshot().Len(),
        encoded.len(),
        workspace.Id()
    );
    assert_eq!(
        workspace.Snapshot().Len(),
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
    assert_eq!(elsewhere.Len(), workspace.Snapshot().Len());
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
    assert_eq!(from_store.Len(), members.len());
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
/// produce a reproducible one.
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
/// over an in-repository fixture in `tests/integration/tests/determinism.rs`, including
/// the `CrossBinary` half that this test does not reach at all — a hundred permutations
/// inside one process say nothing about what a different build of this analyzer would
/// encode.
#[test]
fn Test_A_Hundred_Ingestion_Orders_Should_Yield_Byte_Identical_Queries()
{
    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };
    let mut baseline: Option<Taken> = None;
    for permutation in 0..100_u32
    {
        let taken = Snapshot_Of_One_Order(&members, permutation);

        Assert_Same_As_The_First(&mut baseline, taken, permutation);
    }
    let Taken { bytes, members, .. } = baseline.expect("a hundred permutations ran");

    eprintln!("permutations: 100 orders, {members} members, {} identical bytes", bytes.len());

    // The vacuity guard. If the corpus had one member — or none — every permutation would
    // be the same permutation and this would have asserted nothing.
    assert!(
        members >= 5_000,
        "{members} members cannot meaningfully be permuted a hundred ways"
    );
}

/// What one arrival order produced, and the stride it arrived under.
struct Taken
{
    bytes: Vec<u8>,
    members: usize,
    stride: usize,
}

/// One arrival order, ingested and encoded.
///
/// The stride varies with the permutation too, so change-set boundaries fall between
/// different files each time. Order-independence within one set is a much weaker property
/// than order-independence across them, and only the second one is what a checkout and an
/// editor arriving in either order actually needs.
fn Snapshot_Of_One_Order(members: &[(String, String)], permutation: u32) -> Taken
{
    let ordered = Permuted(members, permutation);
    let stride = 1_usize.saturating_add(
        usize::try_from(permutation)
            .unwrap_or(0)
            .saturating_mul(7)
            .checked_rem(511)
            .unwrap_or(0),
    );
    let mut workspace = Fresh();

    Ingest(&mut workspace, &ordered, stride);

    return Taken {
        bytes: workspace.Snapshot().Encode(),
        members: workspace.Snapshot().Len(),
        stride,
    };
}

/// The first order to arrive becomes the baseline every later one is compared against.
fn Assert_Same_As_The_First(baseline: &mut Option<Taken>, taken: Taken, permutation: u32)
{
    let Some(first) = baseline.as_ref()
    else
    {
        *baseline = Some(taken);

        return;
    };

    assert_eq!(
        taken.members, first.members,
        "permutation {permutation} produced a different number of members"
    );
    assert!(
        taken.bytes == first.bytes,
        "permutation {permutation} (stride {}) produced different bytes",
        taken.stride
    );
}

/// A deterministic reordering, distinct for each `seed`.
///
/// A multiplicative step over the index. Because the step and the length are coprime for
/// the seeds used, the walk visits every element exactly once — a shuffle that dropped or
/// repeated elements would make the test compare snapshots of different corpora and pass
/// only by accident.
/// A stride larger than any realistic corpus, and prime, so that stepping by it visits every
/// index before repeating for the corpus lengths this runs over. The linear scan in
/// `Walk_From` makes the walk a permutation for any stride at all; the prime is what keeps it
/// from degenerating into the identity order.
const STRIDE: usize = 7_919;

fn Permuted(members: &[(String, String)], seed: u32) -> Vec<(String, String)>
{
    let Some(offset) = usize::try_from(seed)
        .unwrap_or(0)
        .saturating_mul(97)
        .checked_rem(members.len())
    else
    {
        return Vec::new();
    };

    return Walk_From(members, offset);
}

/// Step by `STRIDE` from an offset, and where that lands on something already taken, scan
/// forward to the next free slot.
fn Walk_From(members: &[(String, String)], offset: usize) -> Vec<(String, String)>
{
    let len = members.len();
    let mut permuted = Vec::with_capacity(len);
    let mut taken = vec![false; len];
    let mut at = offset;
    for _ in 0..len
    {
        while taken.get(at).copied().unwrap_or(false)
        {
            at = at.saturating_add(1).checked_rem(len).unwrap_or(0);
        }
        if let (Some(member), Some(slot)) = (members.get(at), taken.get_mut(at))
        {
            permuted.push(member.clone());
            *slot = true;
        }
        at = at.saturating_add(STRIDE).checked_rem(len).unwrap_or(0);
    }

    return permuted;
}

/// The permutation itself must be a permutation, or the test above compares snapshots of
/// different corpora.
#[test]
fn Test_The_Permutation_Should_Reorder_Without_Losing_Anything()
{
    let members: Vec<(String, String)> = (0..500_u32)
        .map(|index| return (format!("src/f{index}.rs"), format!("fn f{index}() {{}}")))
        .collect();

    let mut orders = std::collections::BTreeSet::new();
    for seed in 0..100_u32
    {
        let permuted = Permuted(&members, seed);
        let order = Path_Order(&permuted);

        Assert_Holds_Every_Member(&permuted, &members, seed);
        orders.insert(order);
    }

    assert!(
        orders.len() > 50,
        "a hundred seeds produced only {} distinct orders; the test above would be \
         asserting the same order against itself",
        orders.len()
    );
}

/// The same members, the same count, in a different order — which is what a permutation is.
fn Assert_Holds_Every_Member(permuted: &[(String, String)], members: &[(String, String)], seed: u32)
{
    assert_eq!(permuted.len(), members.len(), "seed {seed} changed the length");

    let mut sorted = permuted.to_vec();
    sorted.sort();
    let mut expected = members.to_vec();
    expected.sort();

    assert_eq!(sorted, expected, "seed {seed} lost or repeated a member");
}

/// The order alone, which is what makes one permutation distinct from another.
fn Path_Order(permuted: &[(String, String)]) -> Vec<String>
{
    return permuted
        .iter()
        .map(|(path, _)| return path.clone())
        .collect();
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
    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut workspace = Fresh();
    Ingest(&mut workspace, &members, 512);

    let after_first = workspace.Generation();
    let identity = workspace.Id();
    let redundant = Re_Ingest(&mut workspace, &members);

    assert_eq!(workspace.Generation(), after_first);
    assert_eq!(workspace.Id(), identity);
    assert_eq!(redundant, members.len());
    // The positive control. The first ingestion must have advanced, or "nothing advanced"
    // is a statement about a workspace that never did anything.
    assert!(
        after_first > GenerationId::INITIAL,
        "the first ingestion advanced nothing either"
    );
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
    let set = Change_Set(batch);
    let applied = workspace.Apply(&set).expect("applies");

    assert!(
        matches!(applied, Applied::Unchanged { .. }),
        "a checkout of what is already there is not a change"
    );

    return applied.Effects().len();
}
