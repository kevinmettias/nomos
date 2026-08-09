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
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if path.is_dir()
            {
                if !NOT_SOURCE.contains(&name.as_ref())
                {
                    pending.push(path);
                }
                continue;
            }

            if path.extension().is_some_and(|extension| return extension == "rs")
            {
                paths.push(path);
            }
        }
    }

    paths.sort();

    let mut members = Vec::new();
    for path in paths
    {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // Read as bytes and render lossily rather than requiring UTF-8. A file this
        // workspace cannot decode is still a member of it, and skipping it would make the
        // snapshot describe a tree that is missing files nobody was told about.
        let Ok(bytes) = std::fs::read(&path)
        else
        {
            continue;
        };
        members.push((relative, String::from_utf8_lossy(&bytes).into_owned()));
    }

    return members;
}

fn Scale_Corpus_Or_Skip() -> Option<(PathBuf, Vec<(String, String)>)>
{
    let configured = std::env::var_os("NOMOS_RUST_CORPUS");
    let root = configured
        .clone()
        .map_or_else(|| return PathBuf::from(SCALE_CORPUS), PathBuf::from);

    if !root.is_dir()
    {
        assert!(
            configured.is_none(),
            "NOMOS_RUST_CORPUS is set to {}, which is not a directory. A configured corpus \
             that cannot be read is a failure, not a skip",
            root.display()
        );
        eprintln!("skipped: no corpus at {}", root.display());

        return None;
    }

    let members = Corpus(&root);

    assert!(
        members.len() >= 5_000,
        "found {} Rust files under {}, which is not this corpus. Every assertion below \
         iterates over this set, so a truncated walk makes all of them pass having read \
         almost nothing",
        members.len(),
        root.display()
    );

    return Some((root, members));
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
        let mut set = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
        for (path, content) in batch
        {
            set = set.Present(path.clone(), content.clone());
        }

        workspace
            .Apply(&set)
            .expect("every path in the corpus is workspace-relative");
    }
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
    let text = String::from_utf8(encoded.clone()).expect("the encoding is UTF-8");

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

    assert_eq!(elsewhere.Id(), workspace.Id());
    assert_eq!(elsewhere.Len(), workspace.Snapshot().Len());
    assert_eq!(elsewhere.Variant(), &Variant());
    assert_eq!(elsewhere.Configuration(), Configuration());
    assert_eq!(
        elsewhere.Encode(),
        encoded,
        "and re-encoding what it read is byte-identical, so the reading lost nothing"
    );

    // Every member, answered from the decoded bytes. Not a sample: the claim is about the
    // whole snapshot, and a spot check would pass for a decoder that dropped a suffix.
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

    let recorded: Vec<Vec<u8>> = store
        .Documents()
        .values()
        .filter(|document| return document.kind == DocumentKind::Fact)
        .map(|document| return document.bytes.clone())
        .collect();

    assert_eq!(recorded.len(), 1, "one workspace state was recorded");

    let from_store = recorded
        .first()
        .map(|bytes| return WorkspaceSnapshot::Decode(bytes))
        .expect("the document is there")
        .expect("and it is a workspace snapshot");

    assert_eq!(from_store.Id(), workspace.Id());
    assert_eq!(from_store.Len(), members.len());

    // The index has to survive it. A workspace state filed under DocumentKind::Snapshot
    // would be decoded as a store manifest and break the index for every document in it.
    assert!(store.Index().is_ok(), "the store's index still derives");
    assert_eq!(
        store.Authority(),
        Authority::Observed,
        "a measurement of a tree is observed, not authored"
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
#[test]
fn Test_A_Hundred_Ingestion_Orders_Should_Yield_Byte_Identical_Queries()
{
    let Some((_, members)) = Scale_Corpus_Or_Skip()
    else
    {
        return;
    };

    let mut baseline: Option<(Vec<u8>, usize)> = None;

    for permutation in 0..100_u32
    {
        let ordered = Permuted(&members, permutation);

        // The stride varies with the permutation too, so change-set boundaries fall
        // between different files each time. Order-independence within one set is a much
        // weaker property than order-independence across them, and only the second one is
        // what a checkout and an editor arriving in either order actually needs.
        let stride = 1_usize.saturating_add(
            usize::try_from(permutation)
                .unwrap_or(0)
                .saturating_mul(7)
                .checked_rem(511)
                .unwrap_or(0),
        );

        let mut workspace = Fresh();
        Ingest(&mut workspace, &ordered, stride);

        let encoded = workspace.Snapshot().Encode();

        match &baseline
        {
            None => baseline = Some((encoded, workspace.Snapshot().Len())),
            Some((expected, count)) =>
            {
                assert_eq!(
                    workspace.Snapshot().Len(),
                    *count,
                    "permutation {permutation} produced a different number of members"
                );
                assert!(
                    encoded == *expected,
                    "permutation {permutation} (stride {stride}) produced different bytes"
                );
            }
        }
    }

    let (bytes, count) = baseline.expect("a hundred permutations ran");

    eprintln!("permutations: 100 orders, {count} members, {} identical bytes", bytes.len());

    // The vacuity guard. If the corpus had one member — or none — every permutation would
    // be the same permutation and this would have asserted nothing.
    assert!(
        count >= 5_000,
        "{count} members cannot meaningfully be permuted a hundred ways"
    );
}

/// A deterministic reordering, distinct for each `seed`.
///
/// A multiplicative step over the index. Because the step and the length are coprime for
/// the seeds used, the walk visits every element exactly once — a shuffle that dropped or
/// repeated elements would make the test compare snapshots of different corpora and pass
/// only by accident.
fn Permuted(members: &[(String, String)], seed: u32) -> Vec<(String, String)>
{
    // A stride larger than any realistic corpus, and prime, so that stepping by it visits
    // every index before repeating for the corpus lengths this runs over. The linear scan
    // below makes the walk a permutation for any stride at all; the prime is what keeps it
    // from degenerating into the identity order.
    const STRIDE: usize = 7_919;

    let len = members.len();
    let Some(offset) = usize::try_from(seed)
        .unwrap_or(0)
        .saturating_mul(97)
        .checked_rem(len)
    else
    {
        return Vec::new();
    };

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

        assert_eq!(permuted.len(), members.len(), "seed {seed} changed the length");

        let mut sorted = permuted.clone();
        sorted.sort();
        let mut expected = members.clone();
        expected.sort();
        assert_eq!(sorted, expected, "seed {seed} lost or repeated a member");

        orders.insert(
            permuted
                .iter()
                .map(|(path, _)| return path.clone())
                .collect::<Vec<String>>(),
        );
    }

    assert!(
        orders.len() > 50,
        "a hundred seeds produced only {} distinct orders; the test above would be \
         asserting the same order against itself",
        orders.len()
    );
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

    let mut redundant = 0_usize;
    for batch in members.chunks(512)
    {
        let mut set = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
        for (path, content) in batch
        {
            set = set.Present(path.clone(), content.clone());
        }

        let applied = workspace.Apply(&set).expect("applies");
        assert!(
            matches!(applied, Applied::Unchanged { .. }),
            "a checkout of what is already there is not a change"
        );
        redundant = redundant.saturating_add(applied.Effects().len());
    }

    eprintln!(
        "re-ingestion: {} paths, all redundant, generation still {}",
        redundant,
        workspace.Generation().Raw()
    );

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
