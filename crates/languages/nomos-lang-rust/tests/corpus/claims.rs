//! Every claim this suite makes about the corpus, beside the root that gates them all.
//!
//! All six tests are here because all six are corpus-gated and the derivation behind
//! `tests/contract/tests/corpus_gates.rs` follows calls within one file. None of these tests
//! names `NOMOS_RUST_CORPUS`; each reaches it through [`Corpus_Or_Skip`], so moving one to a
//! sibling module would leave it gated in fact and counted by nobody.

use crate::soundness::Names_Checked;
use crate::walk::{Each_File, Rust_Files};
use crate::walked::{Report_The_Walk, Walk};
use nomos_lang_rust::{Read_Source, Recognition};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The corpus this provider was built to survive.
const DEFAULT_ROOT: &str = "F:/repos/xvpe";

/// The floor below which this corpus is not the corpus.
///
/// Guards every assertion in this file against passing vacuously. If the walk breaks —
/// a changed layout, a permissions failure, a skip rule that matches too much — the file
/// set comes back small or empty and every loop below iterates over nothing while
/// reporting success.
const EXPECTED_AT_LEAST: usize = 5_000;

pub(crate) struct Corpus
{
    pub(crate) root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
}

/// Locates the corpus, or explains why there is nothing to measure.
fn Corpus_Or_Skip() -> Option<Corpus>
{
    let configured = std::env::var_os("NOMOS_RUST_CORPUS");
    let root = configured
        .clone()
        .map_or_else(|| return PathBuf::from(DEFAULT_ROOT), PathBuf::from);
    if !root.is_dir()
    {
        Report_The_Absence(configured.as_deref(), &root);

        return None;
    }
    let files = Rust_Files(&root);

    assert!(
        files.len() >= EXPECTED_AT_LEAST,
        "found {} Rust files under {}, expected at least {EXPECTED_AT_LEAST}. Every \
         assertion in this file iterates over this set, so a truncated walk makes all of \
         them pass having read almost nothing",
        files.len(),
        root.display()
    );

    return Some(Corpus { root, files });
}

/// A corpus that was configured and cannot be read is a failure, not a skip.
fn Report_The_Absence(configured: Option<&std::ffi::OsStr>, root: &Path)
{
    assert!(
        configured.is_none(),
        "NOMOS_RUST_CORPUS is set to {}, which is not a directory. A configured corpus that \
         cannot be read is a failure, not a skip",
        root.display()
    );

    eprintln!(
        "skipped: no corpus at {} and NOMOS_RUST_CORPUS is unset",
        root.display()
    );
}

/// The headline. Real files, real items, and every figure stated over its denominator.
#[test]
fn Test_The_Corpus_Should_Yield_Syntax_Facts()
{
    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let walked = Walk(&corpus);
    let total = corpus.files.len();

    Report_The_Walk(&corpus, &walked);
    assert!(
        walked.unreadable.is_empty(),
        "{} of {total} files could not be read off disk: {:?}",
        walked.unreadable.len(),
        walked.unreadable.iter().take(10).collect::<Vec<&PathBuf>>()
    );
    assert!(
        walked.items > 0,
        "{total} files produced no items at all. A provider that reads everything and \
         reports nothing is broken, not satisfied"
    );
    // Not "most files have items" — a corpus has `mod` stubs and generated stubs that
    // genuinely declare nothing. What must not happen is the reverse: a provider whose
    // usual answer is nothing.
    assert!(
        walked.declaring_nothing.saturating_mul(2) < walked.read,
        "{} of the {} files read declared nothing. That is not a corpus of stubs, it is a \
         reader that stopped reading",
        walked.declaring_nothing,
        walked.read
    );
}

/// Every recognized file reaches exactly one of the two outcomes, and every refusal is
/// named, positioned and accounted for.
///
/// # What this corpus actually refuses
///
/// Seven files of 7,580, and all seven for one reason: a `U+FEFF` byte order mark at
/// byte 15, immediately after a `use super::*;` that somebody prepended to a file which
/// already began with one. A mark at offset zero is an encoding announcement and parses;
/// a mark anywhere else is not whitespace, and rustc rejects those seven files too.
///
/// So the assertion is not a tolerance for a reader that has fallen behind the language.
/// It is that every refusal is explained: a refusal this walk cannot attribute to a stray
/// mark is a new fact and fails the test, whether it turns out to be a corpus defect or a
/// gap in this provider. A budget with no explanation attached is a place for the second
/// kind of failure to hide behind the first.
#[test]
fn Test_Every_Refusal_Should_Be_Named_And_Explained()
{
    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let walked = Walk(&corpus);
    let total = corpus.files.len();
    let mut unexplained = Vec::new();

    assert_eq!(
        walked.read.saturating_add(walked.refused.len()),
        total,
        "every recognized file must reach exactly one outcome"
    );
    for (path, failure) in &walked.refused
    {
        let unaccounted = Unexplained(path, failure);

        unexplained.extend(unaccounted);
    }
    eprintln!(
        "refusals: {} of {total} files, {} explained by a stray byte order mark",
        walked.refused.len(),
        walked.refused.len().saturating_sub(unexplained.len())
    );
    assert!(
        unexplained.is_empty(),
        "{} of {total} files were refused for a reason this walk cannot account for: \
         {unexplained:#?}.\n\
         Either the corpus grew a new kind of damage or this provider has fallen behind \
         the language, and the two need different responses — which is why the count \
         alone was never going to be enough",
        unexplained.len()
    );
}

/// A refused file this walk cannot account for, if it cannot account for it.
///
/// A stray byte order mark is the one damage this corpus is known to carry, so it is
/// reported and excused. Anything else is either new damage or a provider that has fallen
/// behind the language, and the two need different responses — which is why the count alone
/// was never going to be enough.
fn Unexplained(path: &Path, failure: &str) -> Option<PathBuf>
{
    assert!(
        failure.starts_with("line "),
        "a refusal must say where: {} — {failure}",
        path.display()
    );

    let stray_mark = std::fs::read_to_string(path)
        .is_ok_and(|source| return source.trim_start_matches('\u{feff}').contains('\u{feff}'));

    eprintln!(
        "refused {} ({}): {failure}",
        path.display(),
        if stray_mark { "stray byte order mark" } else { "UNEXPLAINED" }
    );

    return (!stray_mark).then(|| return path.to_path_buf());
}

/// Soundness, over files nobody chose to make it pass.
///
/// Every name the provider reports occurs as an identifier in the file it was read from.
/// The sample version of this lives in `tests/guarantee.rs`; this is the version where
/// the provider has not seen the input. [`crate::soundness`] holds the membership check.
#[test]
fn Test_Soundness_Should_Hold_Over_The_Whole_Corpus()
{
    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let mut checked = 0_u64;
    let mut files = 0_usize;

    for path in &corpus.files
    {
        let Some(names) = Names_Checked(path)
        else
        {
            continue;
        };

        checked = checked.saturating_add(names);
        files = files.saturating_add(1);
    }
    eprintln!("soundness: {checked} names checked across {files} files");
    assert!(
        checked > 10_000,
        "only {checked} names were checked across {files} files; that is too few for this \
         corpus to have been read"
    );
}

/// Reading is a function of the bytes, checked against the corpus rather than against a
/// sample.
///
/// A subset, and the subset is named: every 97th file in sorted order. Not a silent cap —
/// the count and the stride are reported, and the property being checked is determinism,
/// for which a spread sample of the corpus is the right instrument and the whole corpus
/// is only slower.
///
/// # What this is evidence for
///
/// [`nomos_lang_rust::SyntaxFactProduction`], the declaration this test predates by
/// several phases. It asserted the property and named nothing, so nothing said which
/// promise it was keeping, and the promise itself was not written down anywhere until
/// `P9-DETERMINISM`.
///
/// It is the strongest instrument for that declaration and the weakest one available to
/// CI, because it is gated: without `NOMOS_RUST_CORPUS` this returns early and prints
/// `ok`, which `docs/records/OD-GATE-001` measures across the suite at 68 assertions. The
/// declaration is therefore also checked over in-repository fixtures in
/// `tests/integration/tests/determinism/`, which is where a gate run actually verifies
/// it. Read the two together: this one has the scale, that one has the reach.
#[test]
fn Test_Reading_The_Corpus_Twice_Should_Reach_The_Same_Facts()
{
    const STRIDE: usize = 97;

    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let mut compared = 0_usize;
    for path in corpus.files.iter().step_by(STRIDE)
    {
        let Ok(source) = std::fs::read_to_string(path)
        else
        {
            continue;
        };

        assert_eq!(
            Read_Source(&source),
            Read_Source(&source),
            "{} read differently twice",
            path.display()
        );
        compared = compared.saturating_add(1);
    }
    eprintln!(
        "determinism: {compared} files compared, every {STRIDE}th of {}",
        corpus.files.len()
    );
    assert!(
        compared >= 50,
        "only {compared} files were compared; the stride sampled almost nothing"
    );
}

/// The corpus is what makes the completeness declaration a measurement rather than a
/// worry. If real Rust had no macros in it, `Unknown` would be pedantry.
#[test]
fn Test_The_Corpus_Should_Show_Why_Completeness_Is_Unknown()
{
    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let walked = Walk(&corpus);
    let per_file = walked
        .unexpanded
        .checked_div(u64::try_from(walked.read).unwrap_or(1))
        .unwrap_or(0);

    eprintln!(
        "completeness: {} unexpanded regions across {} files read ({per_file} per file)",
        walked.unexpanded, walked.read
    );
    assert!(
        walked.unexpanded > 0,
        "{} files and not one macro invocation. Either the corpus is unlike every Rust \
         codebase, or the count is not being taken",
        walked.read
    );
}

/// What this provider will not read, stated over the same tree.
///
/// Recognition is the reason the counts above are what they are, and a corpus walk that
/// never reports its skips is one where a provider silently claiming half the tree looks
/// identical to one claiming the right part of it.
#[test]
fn Test_Unrecognized_Files_Should_Be_Skipped_Rather_Than_Failed()
{
    let Some(corpus) = Corpus_Or_Skip()
    else
    {
        return;
    };
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    Each_File(&corpus.root, |_, name| {
        if let Recognition::Unrecognized { extension } = Recognition::Of_Path(name)
        {
            let label = extension.unwrap_or_else(|| return "<none>".to_owned());
            let seen = skipped.entry(label).or_insert(0);
            *seen = seen.saturating_add(1);
        }
    });
    let total: usize = skipped.values().copied().sum();

    eprintln!(
        "recognition: {} Rust files read, {total} files skipped across {} extensions",
        corpus.files.len(),
        skipped.len()
    );
    assert!(
        !skipped.is_empty(),
        "a real tree has files that are not Rust; finding none means recognition \
         accepted everything"
    );
    assert!(
        skipped.contains_key("toml"),
        "a Cargo workspace has manifests, and they are not Rust: {:?}",
        skipped.keys().collect::<Vec<&String>>()
    );
}
