//! Every claim this suite makes about the corpus, beside the root that gates them all.
//!
//! All six tests are here because all six are corpus-gated and the derivation behind
//! `tests/contract/tests/corpus_gates.rs` follows calls within one file. None of these tests
//! names `NOMOS_RUST_CORPUS`; each reaches it through [`Corpus_Or_Skip`], so moving one to a
//! sibling module would leave it gated in fact and counted by nobody.

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

/// The determinism sample: every 97th file in sorted order.
///
/// Named rather than a cap, because the count and the stride are both reported and a
/// reader has to be able to tell a spread sample from a truncated walk.
const STRIDE: usize = 97;

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
        walked.unreadable.iter().take(UNREADABLE_PATHS_SHOWN).collect::<Vec<&PathBuf>>()
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
        walked.declaring_nothing.saturating_mul(NOTHING_DECLARED_ONE_IN) < walked.read,
        "{} of the {} files read declared nothing. That is not a corpus of stubs, it is a \
         reader that stopped reading",
        walked.declaring_nothing,
        walked.read
    );
}

/// The most unreadable paths a failure names before it stops naming them: enough to see
/// whether the cause is one file or a pattern, and few enough that the message stays
/// readable in a terminal.
const UNREADABLE_PATHS_SHOWN: usize = 10;

/// At most one file in this many may declare nothing. A corpus has `mod` stubs and generated
/// stubs that genuinely declare nothing, so the ratio is not zero -- but a reader that has
/// stopped reading blows straight through it, which is the failure this bounds.
const NOTHING_DECLARED_ONE_IN: usize = 2;

/// Every recognized file reaches exactly one of the two outcomes, and every refusal is
/// named, positioned and accounted for.
///
/// # What this corpus actually refuses
///
/// Two files of 8,975 as last measured, both for the same reason: `//!` placed after
/// `use super::*;` in `xvpe-thread-pool`'s `benches/fiber/throughput_*.rs`. An inner doc
/// comment is sugar for an inner attribute, and an inner attribute must precede every other
/// item in its scope, in any position — a rule this walk enforces by refusing to parse the
/// file, and rustc rejects the same two files for the same reason. The historical case this
/// comment once named, a stray `U+FEFF` byte order mark, is not currently present in this
/// corpus; the check for it stays, because a mark reappearing would be exactly this kind of
/// damage again.
///
/// So the assertion is not a tolerance for a reader that has fallen behind the language.
/// It is that every refusal is explained: a refusal this walk cannot attribute to a known
/// kind of damage is a new fact and fails the test, whether it turns out to be a corpus
/// defect or a gap in this provider. A budget with no explanation attached is a place for
/// the second kind of failure to hide behind the first.
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
        "refusals: {} of {total} files, {} explained (byte order mark or misplaced inner \
         doc comment; the reason for each is on its own line above)",
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
/// A stray byte order mark and a misplaced inner doc comment are the two kinds of damage
/// this corpus is known to carry, so both are reported and excused. Anything else is either
/// new damage or a provider that has fallen behind the language, and the two need different
/// responses — which is why the count alone was never going to be enough.
fn Unexplained(path: &Path, failure: &str) -> Option<PathBuf>
{
    Assert_Refusal_Names_Position(path, failure);

    let source = std::fs::read_to_string(path).ok();
    let reason = Damage_Reason(source.as_deref());

    eprintln!(
        "refused {} ({}): {failure}",
        path.display(),
        reason.unwrap_or("UNEXPLAINED")
    );

    return reason.is_none().then(|| return path.to_path_buf());
}

fn Assert_Refusal_Names_Position(path: &Path, failure: &str)
{
    assert!(
        failure.starts_with("line "),
        "a refusal must say where: {} — {failure}",
        path.display()
    );
}

/// Which of the two known kinds of damage `source` carries, if either.
fn Damage_Reason(source: Option<&str>) -> Option<&'static str>
{
    let stray_mark = source.is_some_and(|source| return source.trim_start_matches('\u{feff}').contains('\u{feff}'));
    if stray_mark
    {
        return Some("stray byte order mark");
    }

    if source.is_some_and(Has_Misplaced_Inner_Documentation_Comment)
    {
        return Some("misplaced inner doc comment");
    }

    return None;
}

/// Whether `source` places a `//!` or `#![...]` inner doc comment or attribute after a real
/// item already began the file.
///
/// Invalid in any position under Rust's own grammar — inner attributes must precede every
/// other item in their scope — so a file shaped this way is refused by rustc as well as by
/// this provider; it is not this provider falling behind the language. A textual scan rather
/// than a second parse, at the same level of precision the byte-order-mark check above
/// already uses: comments and blank lines do not count as "an item began", anything else
/// does.
fn Has_Misplaced_Inner_Documentation_Comment(source: &str) -> bool
{
    let mut seen_item = false;

    for line in source.lines()
    {
        if Line_Is_Misplaced_Inner_Documentation_Comment(line, &mut seen_item)
        {
            return true;
        }
    }

    return false;
}

/// One line's contribution to the scan: whether it is itself an inner doc comment or
/// attribute arriving after a real item already began the file, and — if it is neither —
/// whether it is the item that would make a later one misplaced.
fn Line_Is_Misplaced_Inner_Documentation_Comment(line: &str, seen_item: &mut bool) -> bool
{
    let trimmed = line.trim_start();

    if trimmed.starts_with("//!") || trimmed.starts_with("#![")
    {
        return *seen_item;
    }
    if trimmed.is_empty() || trimmed.starts_with("//")
    {
        return false;
    }

    *seen_item = true;

    return false;
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

    let counted = Count_Checked_Names(&corpus);

    eprintln!(
        "soundness: {} names checked across {} files",
        counted.checked, counted.files
    );
    assert!(
        counted.checked > NAMES_TO_PROVE_THE_CORPUS_WAS_READ,
        "only {} names were checked across {} files; that is too few for this corpus to have \
         been read",
        counted.checked,
        counted.files
    );
}

/// How many identifiers were checked for membership in the file they came from, and
/// across how many files — the whole-corpus counterpart to the sample `tests/guarantee.rs`
/// checks the same way.
///
/// Named fields rather than a pair, because the two counts are a `u64` and a `usize` and a
/// caller reading `counted.checked` cannot mistake it for the file count the way a tuple
/// whose second element happens to be the smaller number invites.
struct SoundnessCount
{
    checked: u64,
    files: usize,
}

fn Count_Checked_Names(corpus: &Corpus) -> SoundnessCount
{
    use crate::soundness::Names_Checked;

    let mut count = SoundnessCount {
        checked: 0,
        files: 0,
    };

    for path in &corpus.files
    {
        let Some(names) = Names_Checked(path)
        else
        {
            continue;
        };

        count.checked = count.checked.saturating_add(names);
        count.files = count.files.saturating_add(1);
    }

    return count;
}

/// The floor below which a whole-corpus soundness pass has measured too little to be
/// evidence of anything: a corpus this size routinely yields more names than this, so
/// falling under it means the walk read almost nothing rather than that the corpus shrank.
const NAMES_TO_PROVE_THE_CORPUS_WAS_READ: u64 = 10_000;

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
        compared >= FILES_THE_STRIDE_MUST_REACH,
        "only {compared} files were compared; the stride sampled almost nothing"
    );
}

/// The floor the stride has to reach for this to be a sample of the corpus rather than of
/// nothing. Stated over the files actually compared rather than over the corpus size the
/// stride was computed from, so a corpus that shrank cannot make the sample look adequate.
const FILES_THE_STRIDE_MUST_REACH: usize = 50;

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

    let skipped = Skipped_Extensions(&corpus);
    let total: usize = skipped.values().copied().sum();

    eprintln!(
        "recognition: {} Rust files read, {total} files skipped across {} extensions",
        corpus.files.len(),
        skipped.len()
    );
    Assert_Skips_Are_Real(&skipped);
}

/// Every extension this walk recognized as not-Rust, and how many files it skipped for
/// each — the evidence that recognition discriminates rather than accepting everything.
fn Skipped_Extensions(corpus: &Corpus) -> BTreeMap<String, usize>
{
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();

    Each_File(&corpus.root, |_, name| {
        if let Recognition::Unrecognized { extension } = Recognition::Of_Path(name)
        {
            let label = extension.unwrap_or_else(|| return "<none>".to_owned());
            let seen = skipped.entry(label).or_insert(0);
            *seen = seen.saturating_add(1);
        }
    });

    return skipped;
}

fn Assert_Skips_Are_Real(skipped: &BTreeMap<String, usize>)
{
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
