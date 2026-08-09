//! I7 — revision archaeology. Fingerprints only, across every archive.
//!
//! Four sets per adjacent pair, and the fourth is the reason the other three are not
//! enough. *Appeared*, *disappeared* and *changed in place* are what a diff between two
//! trees gives you. *Reappeared* — present now, absent in the revision before, present in
//! one earlier still — is not visible to any pairwise diff, and it is the strongest
//! signature of accidental loss: content does not come back on purpose.
//!
//! Nothing is unpacked and nothing is stored. A fingerprint is a path and a normalized
//! hash, which is all four sets need, and it keeps 156 MB of archives out of the store.

use crate::archive::Archive;
use crate::phases::IngestError;
use nomos_spec_model::{BlockKind, ContentHash, RowKind, Segment, Table_Rows};
use core::fmt::Write as _;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Where the v14 tree keeps the volumes the regression headline is measured over.
pub const DOMAIN_VOLUMES: &str = "01_authoring/domain_volumes/";

/// One revision, reduced to what the four sets need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisionFingerprint
{
    pub label: String,
    /// Path within the revision, with the archive's own top directory removed, against the
    /// normalized hash of the document. Normalized rather than verbatim, so a reflow
    /// nobody can see is not reported as a change in place.
    pub documents: BTreeMap<String, String>,
}

/// What became of every path between two adjacent revisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairChange
{
    pub from: String,
    pub to: String,
    pub appeared: Vec<String>,
    pub disappeared: Vec<String>,
    pub changed: Vec<String>,
    /// Absent in `from`, present in `to`, and present in some revision before `from`.
    pub reappeared: Vec<String>,
}

impl PairChange
{
    /// Names the pair and the size of each set, then a few members of each.
    ///
    /// Never a bare total: "38 disappeared" is the number that starts an argument, and the
    /// paths are what ends it.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut line = format!("{} -> {}", self.from, self.to);
        for (label, set) in [
            ("appeared", &self.appeared),
            ("disappeared", &self.disappeared),
            ("changed in place", &self.changed),
            ("reappeared", &self.reappeared),
        ]
        {
            if set.is_empty()
            {
                continue;
            }
            let named: Vec<&str> = set.iter().take(3).map(String::as_str).collect();
            let _ = write!(
                line,
                "\n  {label}: {} ({}{})",
                set.len(),
                named.join(", "),
                if set.len() > named.len() { ", …" } else { "" }
            );
        }

        return line;
    }
}

/// What a census counted over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope
{
    /// The ten volumes the block manifest covers. v15.0 has no such directory at all.
    DomainVolumes,
    /// Every markdown document in the revision, wherever it lives.
    EveryMarkdown,
}

impl Scope
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::DomainVolumes => "01_authoring/domain_volumes",
            Self::EveryMarkdown => "every markdown document",
        };
    }

    fn Covers(self, path: &str) -> bool
    {
        return match self
        {
            Self::DomainVolumes => path.contains(DOMAIN_VOLUMES),
            Self::EveryMarkdown => true,
        };
    }
}

/// Counts of content kinds in one revision, each naming its unit.
///
/// Both readings of a table and both readings of a code block, because the plan states
/// the line counts and the store answers the block counts, and D-132 says a figure that
/// disagrees is recorded rather than replaced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KindCensus
{
    pub documents: u32,
    pub documents_with_tables: u32,
    pub pipe_lines: u32,
    pub non_separator_rows: u32,
    pub content_rows: u32,
    pub fence_lines: u32,
    pub code_blocks: u32,
}

/// The revision archives, in version order.
///
/// Only the full-suite archives. The topic zips beside them (`nomos_v14_31_ocaml_...`)
/// carry one change apiece and are not revisions of the suite; walking them as if they
/// were would report the whole corpus as disappearing between every pair.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the directory holds no revision archive, because a
/// directory that matched nothing reads exactly like an archaeology with nothing to say.
pub fn Revisions_In(directory: &Path) -> Result<Vec<(String, PathBuf)>, IngestError>
{
    let entries = std::fs::read_dir(directory).map_err(|error| {
        return IngestError::Parse(format!("cannot read {}: {error}", directory.display()));
    })?;

    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten()
    {
        let path = entry.path();
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or_default();
        if let Some(label) = Label_Of(name)
        {
            found.push((label, path));
        }
    }

    if found.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} holds no revision archive. Refusing to report an archaeology over nothing",
            directory.display()
        )));
    }

    found.sort_by_key(|(label, _)| return Order(label));
    return Ok(found);
}

/// `nomos-spec-internal-artifacts-v14.36.zip` and `nomos-spec-v15.0.zip`, and nothing else.
fn Label_Of(name: &str) -> Option<String>
{
    let stem = name.strip_suffix(".zip")?;
    let label = stem
        .strip_prefix("nomos-spec-internal-artifacts-")
        .or_else(|| return stem.strip_prefix("nomos-spec-"))?;

    return Version(label).map(|_| return label.to_owned());
}

/// `v14.36` as a pair of numbers, so `v14.9` sorts before `v14.10`.
fn Version(label: &str) -> Option<(u32, u32)>
{
    let (major, minor) = label.strip_prefix('v')?.split_once('.')?;
    return Some((major.parse().ok()?, minor.parse().ok()?));
}

fn Order(label: &str) -> (u32, u32)
{
    return Version(label).unwrap_or((u32::MAX, u32::MAX));
}

/// Revision numbers the archive set skips.
///
/// Reported rather than ignored. Two revisions are adjacent among the archives that exist,
/// which is not the same as adjacent in the corpus's own numbering, and a pair spanning a
/// missing revision attributes two revisions' worth of change to one.
#[must_use]
pub fn Gaps(labels: &[String]) -> Vec<String>
{
    let numbered: Vec<(u32, u32)> = labels.iter().filter_map(|label| return Version(label)).collect();
    let mut missing = Vec::new();

    for pair in numbered.windows(2)
    {
        let (Some(before), Some(after)) = (pair.first(), pair.get(1))
        else
        {
            continue;
        };
        if before.0 != after.0
        {
            continue;
        }
        let mut minor = before.1.saturating_add(1);
        while minor < after.1
        {
            missing.push(format!("v{}.{minor}", before.0));
            minor = minor.saturating_add(1);
        }
    }

    return missing;
}

/// One revision's fingerprints, read without unpacking.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if an entry cannot be read as text, or if the revision
/// holds no markdown at all — an archive that fingerprints to nothing is indistinguishable
/// from one that was never opened.
pub fn Fingerprint(archive: &mut Archive, label: &str) -> Result<RevisionFingerprint, IngestError>
{
    let mut documents = BTreeMap::new();

    for entry in archive.Ending_With(".md")
    {
        let text = archive
            .Read_Text(&entry)
            .map_err(|error| return IngestError::Parse(error.to_string()))?;
        documents.insert(Within(&entry), text);
    }

    return Fingerprint_Of(label, &documents);
}

#[allow(clippy::missing_errors_doc)]
pub fn Fingerprint_Of(
    label: &str,
    documents: &BTreeMap<String, String>,
) -> Result<RevisionFingerprint, IngestError>
{
    if documents.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{label} fingerprints to no document, which reads exactly like a revision that \
             was never opened"
        )));
    }

    return Ok(RevisionFingerprint {
        label: label.to_owned(),
        documents: documents
            .iter()
            .map(|(path, text)| {
                return (path.clone(), ContentHash::Of_Normalized(text).As_Str().to_owned());
            })
            .collect(),
    });
}

/// The path inside the revision, with the archive's own top directory removed.
///
/// Without this every path differs between revisions by the version in its first segment
/// and every pair reports the whole corpus twice.
pub(crate) fn Within(entry: &str) -> String
{
    return entry.split_once('/').map_or_else(|| return entry.to_owned(), |(_, rest)| return rest.to_owned());
}

/// The four sets for every adjacent pair.
///
/// `reappeared` is computed against every revision before `from`, which is why this takes
/// the whole sequence rather than two revisions: a pairwise diff cannot see it.
#[must_use]
pub fn Walk(revisions: &[RevisionFingerprint]) -> Vec<PairChange>
{
    let mut pairs = Vec::new();
    let mut seen_before: BTreeSet<&str> = BTreeSet::new();

    for window in revisions.windows(2)
    {
        let (Some(from), Some(to)) = (window.first(), window.get(1))
        else
        {
            continue;
        };

        let mut change = PairChange {
            from: from.label.clone(),
            to: to.label.clone(),
            ..PairChange::default()
        };

        for (path, hash) in &to.documents
        {
            match from.documents.get(path)
            {
                Some(previous) if previous != hash => change.changed.push(path.clone()),
                Some(_) => {}
                None =>
                {
                    if seen_before.contains(path.as_str())
                    {
                        change.reappeared.push(path.clone());
                    }
                    else
                    {
                        change.appeared.push(path.clone());
                    }
                }
            }
        }

        for path in from.documents.keys()
        {
            if !to.documents.contains_key(path)
            {
                change.disappeared.push(path.clone());
            }
        }

        pairs.push(change);
        seen_before.extend(from.documents.keys().map(String::as_str));
    }

    return pairs;
}

/// Counts one revision's content kinds within a scope.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the scope matches no document. A revision that
/// reorganised the tree away has no such directory, and reporting that as zero rows would
/// be the same defect as a check that walks a missing path and reports clean.
pub fn Census(archive: &mut Archive, scope: Scope) -> Result<KindCensus, IngestError>
{
    let mut census = KindCensus::default();

    for entry in archive.Ending_With(".md")
    {
        if !scope.Covers(&entry)
        {
            continue;
        }

        let text = archive
            .Read_Text(&entry)
            .map_err(|error| return IngestError::Parse(error.to_string()))?;
        census.documents = census.documents.saturating_add(1);

        for line in text.lines()
        {
            if line.trim_start().starts_with("```")
            {
                census.fence_lines = census.fence_lines.saturating_add(1);
            }
        }

        let mut carries_a_table = false;
        for block in Segment(&text)
        {
            if block.kind == BlockKind::Code
            {
                census.code_blocks = census.code_blocks.saturating_add(1);
            }

            for row in Table_Rows(&block)
            {
                carries_a_table = true;
                census.pipe_lines = census.pipe_lines.saturating_add(1);
                if row.kind != RowKind::Separator
                {
                    census.non_separator_rows = census.non_separator_rows.saturating_add(1);
                }
                if row.kind == RowKind::Content
                {
                    census.content_rows = census.content_rows.saturating_add(1);
                }
            }
        }

        if carries_a_table
        {
            census.documents_with_tables = census.documents_with_tables.saturating_add(1);
        }
    }

    if census.documents == 0
    {
        return Err(IngestError::Parse(format!(
            "no document under {}. Refusing to report zero rows for a scope that does not \
             exist in this revision, because a missing path is not an empty one",
            scope.Label()
        )));
    }

    return Ok(census);
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Revision(label: &str, documents: &[(&str, &str)]) -> RevisionFingerprint
    {
        return RevisionFingerprint {
            label: label.to_owned(),
            documents: documents
                .iter()
                .map(|(path, hash)| return ((*path).to_owned(), (*hash).to_owned()))
                .collect(),
        };
    }

    /// The set no pairwise diff can produce. A path gone and then back is not an addition,
    /// and calling it one is how an accidental restoration reads as ordinary authoring.
    #[test]
    fn Test_A_Path_That_Comes_Back_Should_Be_Reappeared_Not_Appeared()
    {
        let walk = Walk(&[
            Revision("v14.1", &[("a.md", "sha256:01")]),
            Revision("v14.2", &[]),
            Revision("v14.3", &[("a.md", "sha256:01")]),
        ]);

        assert_eq!(walk.len(), 2);
        assert_eq!(walk.first().map(|pair| pair.disappeared.clone()), Some(vec!["a.md".to_owned()]));
        assert_eq!(walk.get(1).map(|pair| pair.reappeared.clone()), Some(vec!["a.md".to_owned()]));
        assert_eq!(walk.get(1).map(|pair| pair.appeared.clone()), Some(Vec::new()));
    }

    #[test]
    fn Test_A_Path_Seen_For_The_First_Time_Should_Be_Appeared()
    {
        let walk = Walk(&[Revision("v14.1", &[]), Revision("v14.2", &[("a.md", "sha256:01")])]);

        assert_eq!(walk.first().map(|pair| pair.appeared.clone()), Some(vec!["a.md".to_owned()]));
        assert_eq!(walk.first().map(|pair| pair.reappeared.clone()), Some(Vec::new()));
    }

    #[test]
    fn Test_A_Changed_Hash_Should_Be_Changed_In_Place()
    {
        let walk = Walk(&[
            Revision("v14.1", &[("a.md", "sha256:01")]),
            Revision("v14.2", &[("a.md", "sha256:02")]),
        ]);

        assert_eq!(walk.first().map(|pair| pair.changed.clone()), Some(vec!["a.md".to_owned()]));
        assert_eq!(walk.first().map(|pair| pair.appeared.clone()), Some(Vec::new()));
        assert_eq!(walk.first().map(|pair| pair.disappeared.clone()), Some(Vec::new()));
    }

    #[test]
    fn Test_An_Unchanged_Path_Should_Be_In_No_Set()
    {
        let walk = Walk(&[
            Revision("v14.1", &[("a.md", "sha256:01")]),
            Revision("v14.2", &[("a.md", "sha256:01")]),
        ]);
        let Some(pair) = walk.first()
        else
        {
            panic!("no pair");
        };

        assert!(pair.appeared.is_empty() && pair.disappeared.is_empty());
        assert!(pair.changed.is_empty() && pair.reappeared.is_empty());
        assert_eq!(pair.Summary(), "v14.1 -> v14.2");
    }

    #[test]
    fn Test_A_Skipped_Revision_Number_Should_Be_Named()
    {
        let labels = ["v14.25", "v14.27", "v14.28"].map(str::to_owned).to_vec();

        assert_eq!(Gaps(&labels), vec!["v14.26".to_owned()]);
    }

    /// A major-version step is not a gap: v15.0 does not skip v14.37.
    #[test]
    fn Test_A_Major_Step_Should_Not_Be_Reported_As_A_Gap()
    {
        let labels = ["v14.36", "v15.0"].map(str::to_owned).to_vec();

        assert!(Gaps(&labels).is_empty());
    }

    #[test]
    fn Test_Only_Full_Suite_Archives_Should_Be_Revisions()
    {
        assert_eq!(
            Label_Of("nomos-spec-internal-artifacts-v14.36.zip"),
            Some("v14.36".to_owned())
        );
        assert_eq!(Label_Of("nomos-spec-v15.0.zip"), Some("v15.0".to_owned()));
        assert_eq!(Label_Of("nomos_v14_31_ocaml_semantic_kernel.zip"), None);
        assert_eq!(Label_Of("xvpe-spec-seed-v0.1.zip"), None);
        assert_eq!(Label_Of("nomos full game plan.txt"), None);
    }

    #[test]
    fn Test_Versions_Should_Order_Numerically_Rather_Than_As_Text()
    {
        assert!(Order("v14.9") < Order("v14.10"));
        assert!(Order("v14.36") < Order("v15.0"));
    }

    #[test]
    fn Test_The_Archive_Directory_Should_Be_Stripped_From_A_Path()
    {
        assert_eq!(
            Within("nomos-spec-internal-artifacts-v14.36/01_authoring/a.md"),
            "01_authoring/a.md"
        );
    }
}
