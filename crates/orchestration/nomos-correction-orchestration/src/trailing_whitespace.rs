//! Every trailing-whitespace finding in one file, turned into a real, judgment-free
//! [`CorrectionCandidate`] that strips it -- this crate's second correction family,
//! chosen deliberately for a fix shape phantom-mirror's own single-line strike does not
//! have: a file can carry any number of trailing-whitespace findings at once, and the
//! safe fix for all of them is one edit to the one file, not one candidate per line.
//!
//! # Why this fix is judgment-free
//!
//! `Check_No_Trailing_Whitespace` (`nomos_rules::checks::formatting`) reports a line
//! whose content, with its own line ending stripped, ends in a space or a tab. Removing
//! exactly that trailing space or tab and nothing else cannot change what the line means
//! in any language this workspace recognizes -- Rust and Go both treat trailing
//! whitespace as insignificant everywhere outside a string or byte literal, and a
//! trailing space *inside* one is not what this rule reports, since the rule reads the
//! line's own raw text rather than a parsed token. The same "provably safe, nothing to
//! guess" shape `phantom_mirror`'s own module doc claims for striking a false mirror
//! claim.
//!
//! # Why one candidate covers a whole file rather than one line
//!
//! `phantom_mirror::Phantom_Claim` deliberately reports only the first finding it
//! recognizes, one candidate per run: `nomos_corrections::CorrectionPlan::New` refuses
//! two candidates touching the same path, so batching phantom-mirror's own single-line
//! strikes was never available to it without a second claim from a second file. Trailing
//! whitespace does not have that constraint -- every flagged line in one file is fixed by
//! one before/after read of that file's own content -- so [`Trailing_Whitespace_Claim`]
//! claims a whole file at once, the first real evidence this crate has for batching
//! `P40-CORRECTIONS-SECOND-FAMILY-3`'s own `done_when` asks for. What it does not attempt:
//! batching *across* files into one multi-candidate plan, or choosing between this family
//! and phantom-mirror's own claim when a tree presents both in the same run -- see
//! `crate::run`'s own module doc for what stays undecided.

use nomos_contracts::{Finding, GateCategory};
use nomos_corrections::{CandidateLabel, ChangeSet, CorrectionCandidate, CorrectionClass, Edit};
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

/// Every line in one file that a real `no-trailing-whitespace` finding named -- the
/// file's own path and how many lines it named, kept only for [`crate::run`]'s own
/// summary text: [`Candidate_For`] re-reads and re-scans the file rather than trusting
/// this count to still be true of it.
pub(crate) struct TrailingWhitespaceClaim<'a>
{
    pub path: &'a str,
    pub line_count: usize,
}

/// The first file (by path, ascending) any blocking `no-trailing-whitespace` finding in
/// `findings` names, and how many such findings name it -- or `None` if `findings` names
/// none.
#[must_use]
pub(crate) fn Trailing_Whitespace_Claim(findings: &[Finding]) -> Option<TrailingWhitespaceClaim<'_>>
{
    let mut paths: Vec<&str> = findings.iter().filter_map(File_Path_Of).collect();
    paths.sort_unstable();
    paths.dedup();
    let path = *paths.first()?;

    let line_count = findings.iter().filter(|finding| return File_Path_Of(finding) == Some(path)).count();

    return Some(TrailingWhitespaceClaim { path, line_count });
}

/// The file a blocking `no-trailing-whitespace` finding names, or `None` if `finding` is
/// not one -- not this rule's finding, or not the Blocking gate the rule always raises
/// at. `nomos_rules::checks::formatting::Finding_For_Line` files a finding's location as
/// `path:line`, so the file itself is everything before the last `:`.
fn File_Path_Of(finding: &Finding) -> Option<&str>
{
    if finding.rule.As_Str() != nomos_rules::NO_TRAILING_WHITESPACE || finding.gate != GateCategory::Blocking
    {
        return None;
    }

    let [location] = finding.locations.as_slice()
    else
    {
        return None;
    };

    let (path, _line) = location.rsplit_once(':')?;
    return Some(path);
}

/// Builds the one real [`CorrectionCandidate`] for `claim`: `claim.path`'s own content
/// with every line's trailing space or tab stripped, nothing else touched. Returns the
/// candidate alongside the exact `before`/`after` text it read and computed, the same
/// pairing `phantom_mirror::Candidate_For` returns for the identical reason: a caller
/// stages against the same read this function made rather than a second, independent one
/// that could disagree with it.
///
/// # Errors
///
/// The [`FileSystemError`] `filesystem` reports if `claim.path` cannot be read.
pub(crate) fn Candidate_For<Fs: FileSystem>(root: &Path, claim: &TrailingWhitespaceClaim<'_>, filesystem: &Fs) -> Result<(CorrectionCandidate, String, String), FileSystemError>
{
    let before = filesystem.Read_To_String(&root.join(claim.path))?;
    let after = Stripped_Of_Trailing_Whitespace(&before);

    let edit = Edit::New(claim.path, Some(before.clone()), Some(after.clone()));
    let description = format!("`{}` carries trailing whitespace on {} line(s); stripping it, nothing else", claim.path, claim.line_count);
    let labels = vec![CandidateLabel::MechanicallySafe, CandidateLabel::BehaviorPreserving];

    let candidate = CorrectionCandidate::New(description, ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, labels);

    return Ok((candidate, before, after));
}

/// `text`, with every line's own trailing space or tab removed -- its line endings
/// (`\n`, `\r\n`, or none, for a final line with no trailing newline) are preserved
/// exactly, since a line ending is not whitespace this rule reports.
fn Stripped_Of_Trailing_Whitespace(text: &str) -> String
{
    let mut after = String::with_capacity(text.len());

    for line in text.split_inclusive('\n')
    {
        let (body, newline) = match line.strip_suffix('\n')
        {
            Some(body) => (body, "\n"),
            None => (line, ""),
        };
        let (content, carriage_return) = match body.strip_suffix('\r')
        {
            Some(content) => (content, "\r"),
            None => (body, ""),
        };

        after.push_str(content.trim_end_matches([' ', '\t']));
        after.push_str(carriage_return);
        after.push_str(newline);
    }

    return after;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, EvidenceClass, RuleId, SubjectId};
    use nomos_model::Content_Digest;
    use nomos_platform_std::StdFileSystem;

    /// The line number the second `a.rs` finding below names -- a fixture's own second
    /// line, rather than an unexplained `2` in the middle of a list.
    const SECOND_LINE: usize = 2;

    /// How many `no-trailing-whitespace` findings the fixtures below put on one file, and
    /// how many lines of [`TRAILING_WHITESPACE_FILE`] carry trailing whitespace.
    const FLAGGED_LINE_COUNT: usize = 2;

    #[test]
    fn Test_Trailing_Whitespace_Claim_Should_Name_The_First_File_And_Count_Its_Findings()
    {
        let findings = vec![
            Whitespace_Finding("b.rs", 1),
            Whitespace_Finding("a.rs", 1),
            Whitespace_Finding("a.rs", SECOND_LINE),
        ];

        let claim = Trailing_Whitespace_Claim(&findings).expect("two files carry a real finding");

        assert_eq!(claim.path, "a.rs");
        assert_eq!(claim.line_count, FLAGGED_LINE_COUNT);
    }

    #[test]
    fn Test_Trailing_Whitespace_Claim_Should_Be_None_For_No_Matching_Findings()
    {
        let findings = vec![];

        assert!(Trailing_Whitespace_Claim(&findings).is_none());
    }

    #[test]
    fn Test_An_Advisory_Finding_Should_Not_Be_Claimed()
    {
        let mut finding = Whitespace_Finding("a.rs", 1);
        finding.gate = GateCategory::Advisory;

        assert!(Trailing_Whitespace_Claim(&[finding]).is_none());
    }

    #[test]
    fn Test_A_Finding_From_A_Different_Rule_Should_Not_Be_Claimed()
    {
        let mut finding = Whitespace_Finding("a.rs", 1);
        finding.rule = RuleId::New(nomos_rules::NAMING_CONVENTION);

        assert!(Trailing_Whitespace_Claim(&[finding]).is_none());
    }

    #[test]
    fn Test_Stripped_Of_Trailing_Whitespace_Should_Preserve_Every_Line_Ending()
    {
        let before = "fn Clean()\r\n{\n    return; \t\n}\n no newline ";

        let after = Stripped_Of_Trailing_Whitespace(before);

        assert_eq!(after, "fn Clean()\r\n{\n    return;\n}\n no newline");
    }

    /// The file [`Test_Candidate_For_Should_Strip_Every_Flagged_Line_In_One_Read`] writes:
    /// two lines ending in a trailing space or tab, and two that do not.
    const TRAILING_WHITESPACE_FILE: &str = "fn Clean()\n{\n    return; \n}\t\n";

    #[test]
    fn Test_Candidate_For_Should_Strip_Every_Flagged_Line_In_One_Read()
    {
        let root = std::env::temp_dir().join("nomos-correction-orchestration-trailing-whitespace-candidate");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(root.join("a.rs"), TRAILING_WHITESPACE_FILE)
            .expect("the temporary root the two statements above created holds this file");

        let claim = TrailingWhitespaceClaim { path: "a.rs", line_count: FLAGGED_LINE_COUNT };
        let (candidate, before, after) = Candidate_For(&root, &claim, &StdFileSystem).expect("a real file with trailing whitespace");

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(before, TRAILING_WHITESPACE_FILE);
        assert_eq!(after, "fn Clean()\n{\n    return;\n}\n");
        assert!(candidate.Description().contains("2 line(s)"), "{}", candidate.Description());
    }

    fn Whitespace_Finding(path: &str, line: usize) -> Finding
    {
        return Finding {
            rule: RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE),
            subject: SubjectId::From_Digest(Content_Digest(path.as_bytes())),
            subject_name: format!("{path}:{line}"),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: format!("{path} line {line} ends with trailing whitespace"),
            locations: vec![format!("{path}:{line}")],
        };
    }
}
