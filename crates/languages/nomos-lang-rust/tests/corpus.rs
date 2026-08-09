//! The provider against a corpus it did not write.
//!
//! Everything in `tests/guarantee.rs` is a sample chosen by the person asserting the
//! property, which is the weakest possible evidence that a reader works: the samples were
//! written by somebody who already knew what the reader did. This file points it at
//! `F:/repos/xvpe` — several thousand files of somebody else's Rust, none of it written
//! with this provider in mind.
//!
//! # Every number here comes with what it was measured over
//!
//! A corpus report that says "3 files failed" and not "3 of 7604" is a number nobody can
//! calibrate, and one that says "0 findings" without saying how many files it opened is
//! the shape of a check that walked nothing and reported clean. Every assertion below
//! either names its denominator or fails.
//!
//! # Opt-in by path, and loud when configured and unreadable
//!
//! `NOMOS_RUST_CORPUS` overrides the root. When it is set and unreadable the tests fail,
//! because a gate that quietly skips its own subject is worse than one that fails. When
//! it is unset and the default root is absent — any machine that is not this one — the
//! tests return, having asserted nothing and said so. This is the same bargain
//! `nomos-spec-model`'s `normalizer_gate` strikes with `NOMOS_V14_CORPUS`.
//!
//! Reading these files is not a dependency on the sibling workspace. D-130 governs what
//! Nomos may *build against*, and this crate builds against nothing there; it reads text.

use nomos_lang_rust::{Read_Source, Reading, Recognition};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The corpus this provider was built to survive.
const DEFAULT_ROOT: &str = "F:/repos/xvpe";

/// Directories that are not somebody's source.
///
/// `target` is compiler output — reading it would inflate every count here with generated
/// code and measure this provider against rustc's formatting rather than against a person's.
/// `.git` is object storage that happens to sit in the tree.
const NOT_SOURCE: &[&str] = &["target", ".git"];

/// The floor below which this corpus is not the corpus.
///
/// Guards every assertion in this file against passing vacuously. If the walk breaks —
/// a changed layout, a permissions failure, a skip rule that matches too much — the file
/// set comes back small or empty and every loop below iterates over nothing while
/// reporting success.
const EXPECTED_AT_LEAST: usize = 5_000;

struct Corpus
{
    root: PathBuf,
    files: Vec<PathBuf>,
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
        assert!(
            configured.is_none(),
            "NOMOS_RUST_CORPUS is set to {}, which is not a directory. A configured \
             corpus that cannot be read is a failure, not a skip",
            root.display()
        );

        eprintln!(
            "skipped: no corpus at {} and NOMOS_RUST_CORPUS is unset",
            root.display()
        );
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

/// Every recognized file under a root, in a deterministic order.
///
/// Sorted rather than left in directory order, so that a failure names the same file on
/// two machines and a report of "the first ten failures" is the same ten.
fn Rust_Files(root: &Path) -> Vec<PathBuf>
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

            // Recognition decides, rather than a second extension check written here.
            // Two answers to "does this provider read this file" is one answer too many.
            if Recognition::Of_Path(&name) == Recognition::Recognized
            {
                found.push(path);
            }
        }
    }

    found.sort();
    return found;
}

/// Every identifier-shaped token in a source file.
///
/// Built once per file so soundness can be checked by membership rather than by a
/// substring scan per item, which over this corpus is the difference between a test that
/// runs and one nobody waits for.
///
/// `#` is part of a token because a raw identifier is spelled `r#match`, and the provider
/// reports it that way — the name as written, which is the whole stance of a syntactic
/// reader. The corpus is what surfaced this: `xvpe-collections` declares `mod r#match;`,
/// and a tokenizer that split on `#` reported the provider unsound for saying exactly
/// what the file says.
fn Identifiers(source: &str) -> BTreeSet<&str>
{
    return source
        .split(|character: char| {
            return !character.is_alphanumeric() && character != '_' && character != '#';
        })
        .filter(|token| return !token.is_empty())
        .collect();
}

/// What a walk of the corpus found.
struct Walked
{
    read: usize,
    refused: Vec<(PathBuf, String)>,
    unreadable: Vec<PathBuf>,
    items: usize,
    unexpanded: u64,
    declaring_nothing: usize,
}

fn Walk(corpus: &Corpus) -> Walked
{
    let mut walked = Walked {
        read: 0,
        refused: Vec::new(),
        unreadable: Vec::new(),
        items: 0,
        unexpanded: 0,
        declaring_nothing: 0,
    };

    for path in &corpus.files
    {
        // A file that cannot be read off disk is not a file that failed to parse. One is
        // this provider's answer about the source; the other is the machine, and folding
        // them together would attribute an antivirus lock to a syntax error.
        let Ok(source) = std::fs::read_to_string(path)
        else
        {
            walked.unreadable.push(path.clone());
            continue;
        };

        match Read_Source(&source)
        {
            Reading::Parsed(facts) =>
            {
                walked.read = walked.read.saturating_add(1);
                walked.items = walked.items.saturating_add(facts.items.len());
                walked.unexpanded = walked.unexpanded.saturating_add(u64::from(facts.unexpanded));
                if facts.Declares_Nothing()
                {
                    walked.declaring_nothing = walked.declaring_nothing.saturating_add(1);
                }
            }
            Reading::Unparseable(failure) =>
            {
                walked.refused.push((path.clone(), failure.to_string()));
            }
        }
    }

    return walked;
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

    eprintln!(
        "{}: {total} Rust files, {} read, {} refused, {} unreadable, {} items, {} \
         unexpanded regions, {} files declaring nothing",
        corpus.root.display(),
        walked.read,
        walked.refused.len(),
        walked.unreadable.len(),
        walked.items,
        walked.unexpanded,
        walked.declaring_nothing
    );

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

    assert_eq!(
        walked.read.saturating_add(walked.refused.len()),
        total,
        "every recognized file must reach exactly one outcome"
    );

    let mut unexplained = Vec::new();

    for (path, failure) in &walked.refused
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

        if !stray_mark
        {
            unexplained.push(path.clone());
        }
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

/// Soundness, over files nobody chose to make it pass.
///
/// Every name the provider reports occurs as an identifier in the file it was read from.
/// The sample version of this lives in `tests/guarantee.rs`; this is the version where
/// the provider has not seen the input.
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
        let Ok(source) = std::fs::read_to_string(path)
        else
        {
            continue;
        };
        let Reading::Parsed(facts) = Read_Source(&source)
        else
        {
            continue;
        };

        let identifiers = Identifiers(&source);
        files = files.saturating_add(1);

        for item in &facts.items
        {
            for segment in item.name.split("::")
            {
                // `*` for a glob import and `_` for a type with no single head are this
                // provider saying it has no name, not names it claims to have found.
                if segment == "*" || segment == "_" || segment.is_empty()
                {
                    continue;
                }

                assert!(
                    identifiers.contains(segment),
                    "{} reports `{}` ({}) and `{segment}` is not an identifier in that \
                     file",
                    path.display(),
                    item.Qualified_Name(),
                    item.kind
                );
                checked = checked.saturating_add(1);
            }
        }
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

    let mut skipped: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut pending = vec![corpus.root.clone()];

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

            if let Recognition::Unrecognized { extension } = Recognition::Of_Path(&name)
            {
                let label = extension.unwrap_or_else(|| return "<none>".to_owned());
                let seen = skipped.entry(label).or_insert(0);
                *seen = seen.saturating_add(1);
            }
        }
    }

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
