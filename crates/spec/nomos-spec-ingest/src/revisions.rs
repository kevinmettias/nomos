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

mod labels;
mod fingerprint;
mod revision_fingerprint;
mod walk;
#[cfg(test)]
mod tests;

pub use labels::{Label_Gaps, Revisions_In};
pub(crate) use fingerprint::Fingerprint_Of;
pub use fingerprint::Fingerprint_Revision;
pub use revision_fingerprint::RevisionFingerprint;
pub use walk::Walk_Revisions;

use crate::KindCensus;
use crate::Scope;
use crate::PairChange;
use crate::Archive;
use crate::IngestError;
use nomos_spec_model::{BlockKind, ContentHash, RowKind, Segment, SourceBlock, Table_Rows};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
pub(crate) use fingerprint::Within_Revision;

/// Where the v14 tree keeps the volumes the regression headline is measured over.
pub const DOMAIN_VOLUMES: &str = "01_authoring/domain_volumes/";

/// Counts one revision's content kinds within a scope.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the scope matches no document. A revision that
/// reorganised the tree away has no such directory, and reporting that as zero rows would
/// be the same defect as a check that walks a missing path and reports clean.
pub fn Census_Kinds(archive: &mut Archive, scope: Scope) -> Result<KindCensus, IngestError>
{
    let mut census = KindCensus::default();

    for entry in archive.Listing().Ending_With(".md")
    {
        if !scope.Is_Covering(&entry)
        {
            continue;
        }

        let text = archive
            .Read_Text(&entry)
            .map_err(|error| return IngestError::Parse(error.to_string()))?;
        Count_Document(&mut census, &text);
    }
    Refuse_Empty_Scope(&census, scope)?;

    return Ok(census);
}

/// What one document adds to the census.
///
/// Fences are counted by line rather than by block, because an unclosed fence produces no
/// block at all and a revision that lost its code is exactly where that happens.
fn Count_Document(census: &mut KindCensus, text: &str)
{
    census.documents = census.documents.saturating_add(1);

    for line in text.lines()
    {
        if line.trim_start().starts_with("```")
        {
            census.fence_lines = census.fence_lines.saturating_add(1);
        }
    }

    let mut carries_a_table = false;
    for block in Segment(text)
    {
        carries_a_table |= Has_A_Table(census, &block);
    }

    if carries_a_table
    {
        census.documents_with_tables = census.documents_with_tables.saturating_add(1);
    }
}

/// What one block adds to a census, and whether it carried a table.
///
/// The table answer is returned rather than counted here, because carrying a table is a
/// fact about the document and this sees one block of it.
fn Has_A_Table(census: &mut KindCensus, block: &SourceBlock) -> bool
{
    if block.kind == BlockKind::Code
    {
        census.code_blocks = census.code_blocks.saturating_add(1);
    }

    let mut carries_a_table = false;
    for row in Table_Rows(block)
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

    return carries_a_table;
}

/// A scope that matched no document at all.
///
/// Refused rather than reported as zero. A revision that reorganised the tree away has no
/// such directory, and calling that zero rows is the same defect as a check that walks a
/// missing path and reports clean.
fn Refuse_Empty_Scope(census: &KindCensus, scope: Scope) -> Result<(), IngestError>
{
    if census.documents > 0
    {
        return Ok(());
    }

    return Err(IngestError::Parse(format!(
        "no document under {}. Refusing to report zero rows for a scope that does not \
         exist in this revision, because a missing path is not an empty one",
        scope.Label()
    )));
}

/// The four sets for one adjacent pair.
///
/// `seen_before` is every path any earlier revision held, which is what separates a path
/// arriving for the first time from one coming back.
fn Between_Revisions(
    from: &RevisionFingerprint,
    to: &RevisionFingerprint,
    seen_before: &BTreeSet<&str>,
) -> PairChange
{
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
            None => Note_Arrival(&mut change, path, seen_before),
        }
    }
    change.disappeared = Absent_From(from, to);

    return change;
}

/// A document the earlier revision of a pair did not have: new, or back after an absence.
///
/// The difference is worth keeping. A path that comes back is evidence of a
/// reorganisation, and counting it as new would hide that.
fn Note_Arrival(change: &mut PairChange, path: &str, seen_before: &BTreeSet<&str>)
{
    if seen_before.contains(path)
    {
        change.reappeared.push(path.to_owned());

        return;
    }

    change.appeared.push(path.to_owned());
}

/// The paths the earlier revision held and the later one does not.
fn Absent_From(from: &RevisionFingerprint, to: &RevisionFingerprint) -> Vec<String>
{
    return from
        .documents
        .keys()
        .filter(|path| return !to.documents.contains_key(*path))
        .cloned()
        .collect();
}

#[cfg(test)]
mod census_kinds_tests
{
    use super::*;
    use crate::archive::tests::{FixturePrefix, Zip_Fixture};

    /// The fixture markdown below opens and closes one code fence, so it carries two lines
    /// that are a fence delimiter and nothing else.
    const FENCE_LINES_IN_THE_FIXTURE: u32 = 2;

    /// And three lines that begin with a pipe: the header row, the separator row, and the one
    /// data row. A table is counted from its pipe lines, not from its columns.
    const PIPE_LINES_IN_THE_FIXTURE: u32 = 3;

    /// Two of those three are rows a reader would call content — the header and the data row.
    /// The separator row is the third pipe line and is not one of these, which is what makes
    /// the two counts differ by one rather than being the same number written twice.
    const NON_SEPARATOR_ROWS_IN_THE_FIXTURE: u32 = 2;

    #[test]
    fn Test_Census_Kinds_Should_Count_Fences_And_Tables_Within_Scope()
    {
        let mut archive = Zip_Fixture(
            FixturePrefix("nomos-spec-ingest-census-kinds"),
            "in-scope",
            &[(
                "01_authoring/domain_volumes/02-core/a.md",
                "# A\n\n```\ncode\n```\n\n| a | b |\n| -- | -- |\n| 1 | 2 |\n",
            )],
        );

        let census = Census_Kinds(&mut archive, Scope::DomainVolumes)
            .expect("the fixture's a.md sits under a domain volume, so the scope matches it");

        assert_eq!(census.documents, 1);
        assert_eq!(census.fence_lines, FENCE_LINES_IN_THE_FIXTURE);
        assert_eq!(census.documents_with_tables, 1);
        assert_eq!(census.pipe_lines, PIPE_LINES_IN_THE_FIXTURE);
        assert_eq!(census.non_separator_rows, NON_SEPARATOR_ROWS_IN_THE_FIXTURE);
        assert_eq!(census.content_rows, 1);
    }

    #[test]
    fn Test_Census_Kinds_Should_Refuse_A_Scope_That_Matches_No_Document()
    {
        let mut archive =
            Zip_Fixture(FixturePrefix("nomos-spec-ingest-census-kinds"), "out-of-scope", &[("09-reference/glossary.md", "# G\n\nText.\n")]);

        let refusal = Census_Kinds(&mut archive, Scope::DomainVolumes).expect_err("must refuse");

        assert!(matches!(refusal, IngestError::Parse(_)), "{refusal}");
    }
}
