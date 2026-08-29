//! P3-ARCHAEOLOGY. I7, over every revision archive.
//!
//! The headline the plan says the archaeology must produce, produced. Where the plan's
//! figure and the archives disagree, the archives win and the plan's figure is recorded
//! with a reading of what it counted, per D-132 — the two code-block numbers here are the
//! same finding OD-SPEC-002 settled, arrived at from the other end.
//!
//! Every count names its unit and the scope it was taken over, and the scopes are not the
//! same on both sides of the last pair: v14.36 keeps ten domain volumes and v15.0 has no
//! such directory at all. That asymmetry is stated rather than smoothed, because averaging
//! it away is how "the tree was reorganised" reads as "the content is still there".

use nomos_spec_ingest::{
    Archive, Census_Kinds, Fingerprint_Revision, Label_Gaps, PairChange, RevisionFingerprint, Revisions_In, Scope, Walk_Revisions,
};
use std::path::{Path, PathBuf};

/// The revision that reorganised the tree and dropped the narrative with it.
const V15: &str = "v15.0";

const V14_LAST: &str = "v14.36";

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

fn Fingerprints(root: &Path) -> Vec<RevisionFingerprint>
{
    // The whole walk is built from this list, and an empty one makes every per-pair assertion in
    // this file pass over nothing — the silent no-op the coverage test at the bottom exists to
    // name, arrived at from the directory rather than from the archives.
    let revisions = Revisions_In(root).unwrap_or_else(|error| panic!("{error}"));

    return revisions
        .iter()
        .map(|(label, path)| {
            // One revision dropped here is indistinguishable from the thinned history the negative
            // control at the bottom constructs on purpose, so it must not happen by accident.
            let mut archive = Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));
            // There is no empty fingerprint to stand in for a refused one: a revision carrying no
            // documents reads as a revision that lost all of them, which is this file's headline.
            return Fingerprint_Revision(&mut archive, label).unwrap_or_else(|error| panic!("{error}"));
        })
        .collect();
}

fn Opened(root: &Path, label: &str) -> Archive
{
    // A listing that refused here would surface below as "v15.0 is not among the revision
    // archives", sending the reader to look for a missing zip rather than for the directory that
    // would not read.
    let revisions = Revisions_In(root).unwrap_or_else(|error| panic!("{error}"));
    let (_, path) = revisions
        .iter()
        .find(|(found, _)| return found == label)
        // Only V14_LAST and V15 are ever asked for, so a label with no archive means the set no
        // longer holds a revision every census count in this file was measured against.
        .unwrap_or_else(|| panic!("{label} is not among the revision archives"));

    // Found by label but unopenable is not the scope refusal the census tests assert. Handing back
    // something empty would make "the directory is gone" and "the zip is broken" print alike.
    return Archive::Open(path).unwrap_or_else(|error| panic!("{error}"));
}

/// The revision set itself, before anything is read out of it.
#[test]
fn Test_The_Revision_Set_Should_Be_Ordered_And_Name_Its_Gaps()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let revisions = Revisions_In(&root).expect("reads the directory");
    let labels: Vec<String> = revisions.iter().map(|(label, _)| return label.clone()).collect();

    assert_eq!(labels.len(), 13, "the revision archives are 12 v14 releases and v15.0");
    assert_eq!(labels.first().map(String::as_str), Some("v14.24"));
    assert_eq!(labels.last().map(String::as_str), Some(V15));

    // v14.26 is not in the archive set. Adjacent among the archives is not adjacent in the
    // corpus's own numbering, and the v14.25 -> v14.27 pair carries two revisions of
    // change under one heading.
    assert_eq!(Label_Gaps(&labels), vec!["v14.26".to_owned()]);
}

/// Every adjacent pair yields the four sets, and the fourth is not decoration.
#[test]
fn Test_Every_Adjacent_Pair_Should_Yield_The_Four_Sets()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let fingerprints = Fingerprints(&root);
    let walk = Walk_Revisions(&fingerprints);

    assert_eq!(walk.len(), fingerprints.len().saturating_sub(1), "one pair per adjacency");
    assert_eq!(walk.len(), 12);

    for pair in &walk
    {
        assert!(!pair.from.is_empty() && !pair.to.is_empty(), "{}", pair.Summary());
    }

    let moved: usize = walk.iter().map(|pair| return pair.changed.len()).sum();
    let gone: usize = walk.iter().map(|pair| return pair.disappeared.len()).sum();
    assert!(moved > 0, "no document changed in place across the whole history");
    assert!(gone > 0, "nothing ever disappeared, so the disappeared set proves nothing");
}

/// The result the plan expects most from this walk, and it is empty.
///
/// No path in the whole recorded history goes away and comes back. That is worth stating
/// plainly rather than leaving as an absence a reader might mistake for a detector that
/// never ran: the reappearance case is exercised against a constructed sequence in the
/// module's own tests, so the mechanism is proven and the corpus simply has none.
///
/// It also bounds what the archaeology can claim. v15.0's loss is a single reorganisation,
/// not the accumulation of quiet drops the four-set framing was built to surface — the
/// v14 history disappears exactly one document across twelve revisions.
#[test]
fn Test_No_Document_Should_Have_Disappeared_And_Come_Back()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let walk = Walk_Revisions(&Fingerprints(&root));

    let returned: Vec<&String> = walk.iter().flat_map(|pair| return &pair.reappeared).collect();
    assert!(
        returned.is_empty(),
        "the corpus does have a reappearance and this test has been asserting otherwise: \
         {returned:?}"
    );

    let lost_before_v15: usize = walk
        .iter()
        .filter(|pair| return pair.to != V15)
        .map(|pair| return pair.disappeared.len())
        .sum();
    assert_eq!(lost_before_v15, 1, "documents dropped across the whole v14 history");
}

/// The headline pair, by the numbers, each naming what it counted.
#[test]
fn Test_The_Last_Pair_Should_Carry_Its_Measured_Sizes()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let walk = Walk_Revisions(&Fingerprints(&root));
    let Some(pair) = walk.last()
    else
    {
        // An empty walk has no last pair to size, and returning instead would leave this test
        // green having compared no revision to any other.
        panic!("no pairs");
    };

    assert_eq!(pair.disappeared.len(), 2290, "markdown documents v15.0 does not carry");
    assert_eq!(pair.appeared.len(), 272, "markdown documents v15.0 introduced");
}

/// The pair the plan calls the headline.
#[test]
fn Test_The_Last_Pair_Should_Show_The_Whole_Narrative_Tree_Disappearing()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let walk = Walk_Revisions(&Fingerprints(&root));
    let Some(pair) = walk.last()
    else
    {
        // The same absence again, and without a last pair the ten-volume loss goes unasserted —
        // leaving a green test that reads as the loss still being exactly where this says it is.
        panic!("no pairs");
    };
    let volumes: Vec<&String> = pair
        .disappeared
        .iter()
        .filter(|path| return path.contains(nomos_spec_ingest::DOMAIN_VOLUMES))
        .collect();

    assert_eq!(pair.from, V14_LAST);
    assert_eq!(pair.to, V15);
    assert_eq!(
        volumes.len(),
        10,
        "the ten domain volumes are what v15.0 dropped\n{}",
        pair.Summary()
    );
    assert!(
        pair.changed.is_empty(),
        "v15.0 kept no path at its old location, so nothing can have changed in place: {:?}",
        pair.changed.iter().take(5).collect::<Vec<_>>()
    );
    assert!(!pair.appeared.is_empty(), "v15.0 added the records and nothing shows it");
}

/// The counts, each naming its unit and the scope it was taken over.
#[test]
fn Test_The_Headline_Counts_Should_Name_Their_Unit_And_Their_Scope()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let mut archive = Opened(&root, V14_LAST);
    let v14 = Census_Kinds(&mut archive, Scope::DomainVolumes).expect("counts v14.36");

    assert_eq!(v14.documents, 10, "domain volumes");
    assert_eq!(v14.documents_with_tables, 6, "of the ten, the ones carrying a table");
    assert_eq!(v14.pipe_lines, 282, "pipe lines");
    assert_eq!(v14.non_separator_rows, 258, "authored rows, header included");
    assert_eq!(v14.content_rows, 234, "data rows");
    // The plan says 6 code blocks. Six is the fence count; three is the blocks. Recorded
    // rather than corrected — OD-SPEC-002 settled it and this reproduces it from the
    // archive rather than from the working tree.
    assert_eq!(v14.fence_lines, 6, "fence lines, which is the plan's figure");
    assert_eq!(v14.code_blocks, 3, "fenced blocks, which is what the plan named");
}

/// v15.0 has no domain volumes, and the census refuses to call that zero.
///
/// A missing path is not an empty one. Reporting `0 rows` here would say the volumes
/// survived and were emptied, when what happened is that the directory is gone.
#[test]
fn Test_A_Scope_The_Revision_Does_Not_Have_Should_Be_Refused()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let mut archive = Opened(&root, V15);
    let refusal = Census_Kinds(&mut archive, Scope::DomainVolumes)
        .expect_err("v15.0 has no domain volumes and must not report zero");

    assert!(format!("{refusal}").contains("does not exist"), "{refusal}");
}

/// What v15.0 does retain, measured over the only scope it has.
#[test]
fn Test_V15_Should_Retain_Almost_No_Table_And_No_Code_At_All()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let mut v15_archive = Opened(&root, V15);
    let v15 = Census_Kinds(&mut v15_archive, Scope::EveryMarkdown).expect("counts v15.0");
    let mut v14_archive = Opened(&root, V14_LAST);
    let v14 = Census_Kinds(&mut v14_archive, Scope::EveryMarkdown).expect("counts v14.36");

    assert!(v15.documents > 0, "v15.0 has no markdown at all, so this measured nothing");
    assert_eq!(v15.code_blocks, 0, "fenced blocks anywhere in v15.0");
    assert_eq!(v15.fence_lines, 0, "fence lines anywhere in v15.0");
    // One table survives tree-wide, in the non-negotiable-principles document: 12 pipe
    // lines, of which 1 is the header, 1 the delimiter and 10 the actors. OD-SPEC-002's
    // "retains 11 table rows" is the non-separator count and is right; it predates the
    // header kind, so the number it names is header-plus-data rather than data.
    assert_eq!(v15.documents_with_tables, 1, "documents carrying a table");
    assert_eq!(v15.pipe_lines, 12, "pipe lines anywhere in v15.0");
    assert_eq!(v15.non_separator_rows, 11, "authored rows, which is OD-SPEC-002's figure");
    assert_eq!(v15.content_rows, 10, "data rows");
    assert!(
        v14.pipe_lines > v15.pipe_lines,
        "v15.0 did not lose table content tree-wide, so the headline is wrong"
    );
    assert!(v14.code_blocks > 0, "v14.36 carried no code, so its loss proves nothing");
}

/// Fingerprinting is a read, so running it twice must produce the same answer.
#[test]
fn Test_Fingerprinting_Should_Be_Stable()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let mut first = Opened(&root, V15);
    let mut second = Opened(&root, V15);

    assert_eq!(
        Fingerprint_Revision(&mut first, V15).expect("fingerprints"),
        Fingerprint_Revision(&mut second, V15).expect("fingerprints again")
    );
}

/// A pair's summary names paths. A report that only counts cannot answer "which ones",
/// which is the only question a loss report is asked.
#[test]
fn Test_A_Pair_Summary_Should_Name_What_It_Counted()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let walk = Walk_Revisions(&Fingerprints(&root));
    let Some(pair) = walk.last()
    else
    {
        // And here an empty walk would leave the summary unread, so a report that counted without
        // naming a single path would go on passing.
        panic!("no pairs");
    };
    let summary = pair.Summary();

    assert!(summary.contains(V14_LAST) && summary.contains(V15), "{summary}");
    assert!(summary.contains("disappeared"), "{summary}");
    assert!(
        summary.contains(".md"),
        "the summary counted without naming a single path: {summary}"
    );
}

/// The whole history, as one report. Kept last because it is the expensive one, and it is
/// the one that would silently become a no-op if the revision set ever read as empty.
#[test]
fn Test_The_Archaeology_Should_Cover_Every_Revision_Exactly_Once()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let fingerprints = Fingerprints(&root);
    let walk = Walk_Revisions(&fingerprints);
    let mut visited: Vec<&str> = Vec::new();

    for pair in &walk
    {
        if visited.is_empty()
        {
            visited.push(&pair.from);
        }
        assert_eq!(
            visited.last().copied(),
            Some(pair.from.as_str()),
            "the walk skipped a revision: {}",
            pair.Summary()
        );
        visited.push(&pair.to);
    }
    assert_eq!(visited.len(), fingerprints.len());
    assert!(
        fingerprints.iter().all(|revision| return !revision.documents.is_empty()),
        "a revision fingerprinted to nothing"
    );
}

/// The negative control for the four sets, over the real archives rather than a fixture:
/// a revision removed from the middle must change what the walk reports.
#[test]
fn Test_Dropping_A_Revision_Should_Change_The_Walk()
{
    let Some(root) = Archives()
    else
    {
        return;
    };
    let full = Fingerprints(&root);
    assert!(full.len() > 3, "too few revisions to drop one");

    let thinned: Vec<RevisionFingerprint> = full
        .iter()
        .enumerate()
        .filter(|(index, _)| return *index != 1)
        .map(|(_, revision)| return revision.clone())
        .collect();

    let before = Walk_Revisions(&full);
    let after = Walk_Revisions(&thinned);

    assert_ne!(before.len(), after.len());
    assert_ne!(Shape(&before), Shape(&after), "dropping a revision changed nothing");
}

fn Shape(walk: &[PairChange]) -> Vec<(String, String, usize, usize, usize, usize)>
{
    return walk
        .iter()
        .map(|pair| {
            return (
                pair.from.clone(),
                pair.to.clone(),
                pair.appeared.len(),
                pair.disappeared.len(),
                pair.changed.len(),
                pair.reappeared.len(),
            );
        })
        .collect();
}
