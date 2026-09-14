//! Nothing in this workspace states its own architecture by a numeric band any more.
//!
//! `OD-RULES-020` replaced numeric bands with named zones, and `P41-ZONES-MIGRATION-3`
//! carried the migration through `README.md`, `nomos-rules`' own `ZONES` and the
//! dependency-graph tests that read them. It never touched the places a band number also
//! lived in prose, and a later item closed only the narrowest of them: a workspace member's
//! own `Cargo.toml` `description`, which every crate had opened with a sentence like
//! `"Band 40. ..."` since before zones existed.
//!
//! # Why the description check was not enough
//!
//! It was a phrase check over one line of one file per crate, and it said so. The rest of
//! the vocabulary was never measured. Measured 2026-09-14, after that check had been green
//! for some time: **113 numeric band references survived** — 52 in `Cargo.toml` comments,
//! 47 of those in the root workspace manifest alone, and 61 in Rust module documentation
//! across 55 crates. Nearly every crate in the workspace opened its `lib.rs` by announcing
//! a band.
//!
//! That is exactly the stale architectural language `AGENTS.md` names as something a reader
//! should not have to mentally translate, and it is worse than an ordinary stale comment:
//! a reader who finds `Band 41` beside a zone table has no way to tell which of the two
//! this workspace believes, and the numbers implied a total ordering `OD-RULES-020`
//! retired precisely because it was artificial.
//!
//! # What replaced them
//!
//! The zone the crate actually sits in, taken from `nomos-rules`' own `ZONES` rather than
//! retyped from memory. Two root-manifest comments turned out to describe groups spanning
//! two zones and now say so instead of asserting one; a handful of cross-references that
//! reasoned about bands as an ordering ("one band above", "a band may not depend on its own
//! band") were rewritten as what the dependency model actually declares, which is zones and
//! named same-zone edges.
//!
//! # Why records are not judged
//!
//! Because a record is a dated decision, not a description of the current tree. `docs/`
//! still carries band references and should: `OD-CONTRACTS-001` is *titled* "Band 0 admits
//! what crosses a boundary", `OD-RULES-014` has an amendment named for a band, and several
//! records reason about the ordering that was live when they were written. Rewriting them
//! would falsify the history this repository keeps records for, and `spec/`'s projection
//! inherits the same text because it is derived from them. This guard judges the two file
//! kinds a reader consults for what is true *now* -- source and manifests -- and leaves the
//! ones consulted for what was decided *then* alone.
//!
//! # Why two files are exempt
//!
//! Because they are about bands. A guard that cannot name its own subject would force the
//! two tests documenting this retirement to stop describing it, which is the vacuity this
//! repository's records are about. The exemption is a named list with a reason each, and it
//! is checked in both directions: an entry that stops carrying a band reference must be
//! removed, so the list cannot quietly outlive the thing it excuses.

use crate::bands::Repository_Root;
use std::path::{Path, PathBuf};

/// The prefix a description can no longer open with.
const STALE_PREFIX: &str = "description = \"Band ";

/// The files permitted to write a numeric band, and why each one must.
///
/// Rows are `(repository-relative path, why)`. Checked in both directions by
/// [`Test_Every_Exempt_File_Should_Still_Carry_A_Band_Reference`].
const DISCUSSES_BANDS: &[(&str, &str)] = &[
    (
        "tests/contract/tests/boundaries/band_zero.rs",
        "asserts where the Protocol zone is described, and says so in the vocabulary that \
         was retired because the record it enforces used it",
    ),
    (
        "tests/contract/tests/boundaries/manifest_bands.rs",
        "this module: it names the retired vocabulary in its documentation and feeds it to \
         its own detector as fixtures",
    ),
];

/// Whether `path` is one the exemption list names.
fn Is_Exempt(path: &str) -> bool
{
    return DISCUSSES_BANDS.iter().any(|(exempt, _)| *exempt == path);
}

/// Whether `line` writes a band number rather than merely the word.
fn Is_A_Numeric_Band(line: &str) -> bool
{
    let mut rest = line;

    while let Some(offset) = rest.find("Band ")
    {
        let Some(after) = rest.get(offset.saturating_add("Band ".len())..)
        else
        {
            return false;
        };

        if after.starts_with(|character: char| character.is_ascii_digit())
        {
            return true;
        }

        rest = after;
    }

    return false;
}

/// Every line of `text` that writes a band number, as one-based line numbers.
fn Numeric_Band_Lines(text: &str) -> Vec<usize>
{
    let mut found = Vec::new();

    for (index, line) in text.lines().enumerate()
    {
        if Is_A_Numeric_Band(line)
        {
            found.push(index.saturating_add(1));
        }
    }

    return found;
}

/// Every tracked `.rs` and `Cargo.toml` under `root`, as repository-relative paths.
fn Judged_Files(root: &Path) -> Vec<(String, PathBuf)>
{
    let mut found = Vec::new();
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
            let name = entry.file_name().to_string_lossy().into_owned();

            if path.is_dir()
            {
                if name != "target" && name != ".git"
                {
                    pending.push(path);
                }
                continue;
            }

            if name == "Cargo.toml" || name.ends_with(".rs")
            {
                let Ok(relative) = path.strip_prefix(root)
                else
                {
                    continue;
                };

                found.push((relative.to_string_lossy().replace('\\', "/"), path));
            }
        }
    }

    found.sort();

    return found;
}

/// No manifest describes itself by a numeric band.
///
/// The original and narrowest claim, kept because it names one specific shape — the
/// opening of a `description` — that the wider check below would report without saying
/// which of the many band spellings it found.
#[test]
fn Test_No_Member_Manifest_Should_Open_Its_Description_With_A_Numeric_Band()
{
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut offending = Vec::new();
    for member in workspace.Members()
    {
        let manifest = member.root.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));

        if Opens_With_A_Numeric_Band(&text)
        {
            offending.push(member.name.clone());
        }
    }

    offending.sort();

    assert!(
        offending.is_empty(),
        "{offending:?} still describe themselves by a numeric band. Zones replaced bands \
         everywhere else this workspace states its own architecture; a manifest that still \
         opens with one is the stale label `OD-RULES-020` and `P41-ZONES-MIGRATION-3` were \
         meant to retire, left where nothing read it back."
    );
}

/// No source file or manifest anywhere carries a numeric band.
///
/// The wide claim. The description check above covers one line per crate; this covers the
/// comments and module documentation where the other 113 references actually were.
#[test]
fn Test_No_Source_Or_Manifest_Should_Carry_A_Numeric_Band()
{
    let root = Repository_Root();
    let mut offending = Vec::new();

    for (relative, path) in Judged_Files(&root)
    {
        if Is_Exempt(&relative)
        {
            continue;
        }

        let Ok(text) = std::fs::read_to_string(&path)
        else
        {
            continue;
        };

        for line in Numeric_Band_Lines(&text)
        {
            offending.push(format!("{relative}:{line}"));
        }
    }

    offending.sort();

    assert!(
        offending.is_empty(),
        "{} numeric band references survive:\n  {}\n\n\
         `OD-RULES-020` replaced bands with named zones because the total ordering was \
         artificial. A band number left in a comment or a module doc states an architecture \
         this workspace stopped believing in, beside a zone table that contradicts it, and \
         a reader has no way to tell which one is current. Name the zone the crate actually \
         sits in — `nomos-rules`' `ZONES` is the authority — or delete the reference if the \
         number was all it carried. If the file genuinely needs to discuss bands, add it to \
         DISCUSSES_BANDS with the reason.",
        offending.len(),
        offending.join("\n  ")
    );
}

/// The exemption list does not outlive what it excuses.
///
/// An exempt file that no longer writes a band reference does not need excusing, and a row
/// left behind would silently widen the hole the next time that file changed.
#[test]
fn Test_Every_Exempt_File_Should_Still_Carry_A_Band_Reference()
{
    let root = Repository_Root();
    let mut stale = Vec::new();

    for (relative, _) in DISCUSSES_BANDS
    {
        let path = root.join(relative);
        let Ok(text) = std::fs::read_to_string(&path)
        else
        {
            stale.push(format!("{relative} (does not exist)"));
            continue;
        };

        if Numeric_Band_Lines(&text).is_empty()
        {
            stale.push(format!("{relative} (carries no band reference)"));
        }
    }

    assert!(
        stale.is_empty(),
        "DISCUSSES_BANDS excuses files that no longer need it: {stale:?}.\n\
         Drop the rows. An exemption nothing uses is a hole waiting for the next edit to \
         that file, and this list is checked in both directions so it cannot quietly \
         outlive its subject."
    );
}

/// Whether `manifest_text`'s `description` line opens with `STALE_PREFIX` followed by a
/// digit — a real band number, not merely the word appearing somewhere in prose.
fn Opens_With_A_Numeric_Band(manifest_text: &str) -> bool
{
    return manifest_text
        .lines()
        .any(|line| line.starts_with(STALE_PREFIX) && line[STALE_PREFIX.len()..].starts_with(|character: char| character.is_ascii_digit()));
}

#[cfg(test)]
mod tests
{
    use super::{Is_A_Numeric_Band, Opens_With_A_Numeric_Band};

    #[test]
    fn Test_A_Numeric_Band_Opening_Should_Be_Detected()
    {
        assert!(Opens_With_A_Numeric_Band("description = \"Band 40. Composes something.\"\n"));
    }

    #[test]
    fn Test_A_Lettered_Band_Opening_Should_Still_Be_Detected()
    {
        assert!(Opens_With_A_Numeric_Band("description = \"Band 1p. Port traits.\"\n"));
    }

    #[test]
    fn Test_A_Cleaned_Description_Should_Not_Be_Detected()
    {
        assert!(!Opens_With_A_Numeric_Band("description = \"Composes something.\"\n"));
    }

    #[test]
    fn Test_The_Word_Band_Elsewhere_In_Prose_Should_Not_Be_Detected()
    {
        assert!(!Opens_With_A_Numeric_Band(
            "description = \"Reads a Band 40 crate's own facts.\"\n"
        ));
    }

    #[test]
    fn Test_A_Band_Written_Anywhere_In_A_Line_Should_Be_Detected()
    {
        assert!(Is_A_Numeric_Band("//! Band 92 - host. What this workspace serves."));
        assert!(Is_A_Numeric_Band("    # Band 23 - the contract."));
        assert!(Is_A_Numeric_Band("//! one band above, and Band 41 is where it sits."));
    }

    #[test]
    fn Test_The_Word_Band_Without_A_Number_Should_Not_Be_Detected()
    {
        assert!(!Is_A_Numeric_Band("//! Zone: Host. What this workspace serves."));
        assert!(!Is_A_Numeric_Band("//! a Band of brothers, unnumbered."));
        assert!(!Is_A_Numeric_Band("//! the band table is README.md"));
    }
}
