//! What does not run, and how much of it.
//!
//! Several of this workspace's strongest assertions are made against corpora that live
//! outside the repository: the v14 authoring tree, the versioned archives, and a large Rust
//! tree the language providers read. Each is reached through an environment variable, and a
//! test that cannot find its corpus returns rather than failing — deliberately, because a
//! machine that is not the one holding the corpus should still be able to run the suite.
//!
//! A test that returns early prints `ok`. Nothing distinguishes it from a test that read
//! seven thousand files and agreed with all of them, and CI sets none of the three
//! variables — so on every pull request a substantial block of this suite reports success
//! having touched nothing.
//!
//! That is the defect [`Test_The_Workspace_Should_Not_Appear_Empty`] exists to prevent, one
//! level up from where it was being applied. The remedy here is not to fail: a gate that can
//! never be green is a gate everybody learns to ignore, and the corpora genuinely are not
//! present on most machines. The remedy is that the size of the hole is declared, checked,
//! and reported — so it is a number somebody chose rather than a silence nobody measured.
//!
//! # The three variables
//!
//! `NOMOS_V14_CORPUS` — the v14.36 authoring tree. Carries the normalizer's reproduction of
//! the real canonical hash, the byte order mark sweep over every authored document, the
//! 282/258/234 and 30/28 domain counts, the whole-corpus ingestion, and restoration.
//!
//! `NOMOS_SPEC_ARCHIVES` — the versioned archives, read without unpacking. Carries revision
//! fingerprinting across every archive, the cross-revision regression report, the sibling
//! suites, and the family counts that are measured against the register.
//!
//! `NOMOS_RUST_CORPUS` — a large real Rust tree, defaulting to the sibling workspace.
//! Carries the analysis slice's scale claims: materialization over a corpus nobody wrote for
//! it, the second-run reuse figure, and the coverage the weaker provider buys.
//!
//! # Absence is silent; misconfiguration is not
//!
//! Every helper behind these variables asserts the path is a directory and fails when it is
//! not, with the reasoning written at the assertion. So a corpus pointed somewhere wrong is
//! loud. Only an *unset* variable is quiet, and that is the case this file is about.
//!
//! # Where the numbers come from
//!
//! Measured, not estimated. With all three variables set to nonexistent paths,
//! `cargo test --workspace --no-fail-fast` fails exactly [`GATED_TOTAL`] tests; with none
//! set, the same tests pass. Re-run that to reproduce the table.
//!
//! The measurement is no longer the only thing standing behind the table. Every column is
//! now derived from the source and compared against what is declared here — the file set,
//! the variables each file reads, the tests each file holds, and, by way of
//! [`nomos_contract_tests::Corpus_Gates`], how many of those tests are gated. The two
//! derivations were built independently and agree on all fourteen rows.

use nomos_contract_tests::{Corpus_Gates, Workspace, CORPUS_VARIABLES};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One test file that reaches a corpus.
struct Gate
{
    /// Repo-relative, forward slashes.
    path: &'static str,
    /// The variables this file reads, sorted.
    variables: &'static [&'static str],
    /// How many of its tests do nothing when their corpus is absent.
    ///
    /// Declared rather than derived, and then checked against the derivation by
    /// [`Test_Every_Declared_Gate_Count_Should_Be_The_One_In_The_Source`]. Both halves
    /// matter: deriving it outright would let a fifteenth gated assertion appear in the
    /// headline with nobody deciding it should, and declaring it outright is what left
    /// the most-cited number in this file the least checked one.
    gated: usize,
    /// How many tests the file has in total. Declared so that a test added to a gated file
    /// cannot inherit the file's silence without somebody deciding it should.
    tests: usize,
}

// Assembled from their parts so that this file does not contain the spellings it searches
// for. It reads all three variables — to report on them — and would otherwise find itself,
// and the alternative is an exemption list.
//
// An exemption is the easiest place in a check to hide something, which is the finding
// P8-COMPOSE recorded when `Test_Nothing_In_The_Slice_Should_Invent_An_Identity` had the
// same problem: a scanner that exempts itself leaves the one unchecked file exactly where
// somebody would put the thing it looks for.
const V14: &str = concat!("NOMOS_", "V14_CORPUS");
const ARCHIVES: &str = concat!("NOMOS_", "SPEC_ARCHIVES");
const RUST: &str = concat!("NOMOS_", "RUST_CORPUS");

const VARIABLES: &[&str] = &[ARCHIVES, RUST, V14];

/// Every test file that reaches a corpus, with what it gates.
const GATES: &[Gate] = &[
    Gate {
        path: "crates/languages/nomos-lang-rust/tests/corpus.rs",
        variables: &[RUST],
        gated: 6,
        tests: 6,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/archive_reads.rs",
        variables: &[ARCHIVES],
        gated: 3,
        tests: 8,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/family_counts.rs",
        variables: &[ARCHIVES, V14],
        gated: 2,
        tests: 6,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/overlay.rs",
        variables: &[ARCHIVES, V14],
        gated: 4,
        tests: 5,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/real_corpus.rs",
        variables: &[V14],
        gated: 4,
        tests: 4,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/regression_report.rs",
        variables: &[ARCHIVES],
        gated: 9,
        tests: 14,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/restoration.rs",
        variables: &[V14],
        gated: 6,
        tests: 6,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/revisions.rs",
        variables: &[ARCHIVES],
        gated: 12,
        tests: 12,
    },
    Gate {
        path: "crates/spec/nomos-spec-ingest/tests/siblings.rs",
        variables: &[ARCHIVES],
        gated: 9,
        tests: 9,
    },
    Gate {
        path: "crates/spec/nomos-spec-model/tests/byte_order_mark.rs",
        variables: &[V14],
        gated: 1,
        tests: 3,
    },
    Gate {
        path: "crates/spec/nomos-spec-model/tests/normalizer_gate.rs",
        variables: &[V14],
        gated: 1,
        tests: 6,
    },
    Gate {
        // The suite is `tests/table_rows/`, and only this module of it reaches a corpus.
        // Both of its gated tests sit beside `Corpus_Root` deliberately: the derivation
        // below follows helpers within one file, so a gated test in a sibling module would
        // be counted by nobody.
        path: "crates/spec/nomos-spec-store/tests/table_rows/corpus.rs",
        variables: &[V14],
        gated: 2,
        tests: 2,
    },
    Gate {
        // Five of the suite's six tests are gated, and all five are in this one module with
        // the root that gates them. The sixth needs no corpus and lives with the permutation
        // it is about.
        path: "crates/substrate/nomos-workspace/tests/portable/over_the_corpus.rs",
        variables: &[RUST],
        gated: 5,
        tests: 5,
    },
    Gate {
        path: "tests/integration/tests/analysis_slice.rs",
        variables: &[RUST],
        // Unchanged by P9-FALLBACK, deliberately. The five assertions it added are about
        // spending the selection per subject and are written against the precision corpus,
        // which is in this repository — so the hole this table measures did not grow.
        gated: 4,
        tests: 29,
    },
];

/// How many assertions this suite does not make when no corpus is configured.
///
/// The headline. Stated once so it can be cited, and checked against the table so it cannot
/// drift from it.
const GATED_TOTAL: usize = 68;

/// The number this is worth reading against: how many tests the gated files hold in total.
///
/// Rose to 130 with `P9-FALLBACK`'s five assertions in `analysis_slice.rs`, none of which
/// reads a corpus. [`GATED_TOTAL`] is unchanged, and that is the point of keeping the two
/// numbers apart: a gated file growing is not the hole growing.
///
/// Fell to 115 as the test crates were decomposed for `check-file-size`: fourteen of
/// `table_rows.rs`'s sixteen tests never read a corpus, and one of `portable.rs`'s six did
/// not either, and all fifteen now live in sibling modules rather than inflating this
/// denominator — the same move in the other direction. [`GATED_TOTAL`] is unchanged through
/// every one of those splits, which is the check that they moved tests rather than silence.
const TESTS_IN_GATED_FILES: usize = 115;

/// The table accounts for every file that reaches a corpus.
///
/// The direction that matters. A test file added tomorrow that reads one of these variables
/// is invisible in a green suite — it will skip, print `ok`, and never appear in any count —
/// unless something goes looking for it. This goes looking.
#[test]
fn Test_Every_File_That_Reaches_A_Corpus_Should_Be_Declared()
{
    let root = Workspace::Workspace_Root();
    let found = Files_Naming_A_Corpus(&root);

    assert!(
        !found.is_empty(),
        "no test file in this workspace names any of {VARIABLES:?}.\n\
         Every assertion in this file is about that set, so an empty one passes having \
         checked nothing — and this workspace has corpus-gated tests."
    );
    Assert_The_Table_And_The_Tree_Agree(&found);
}

/// Both directions: a file that reaches a corpus and is in no row, and a row naming a file
/// that no longer reaches one.
fn Assert_The_Table_And_The_Tree_Agree(found: &BTreeSet<String>)
{
    let declared: BTreeSet<&str> = GATES.iter().map(|gate| return gate.path).collect();
    let actual: BTreeSet<&str> = found.iter().map(String::as_str).collect();
    let undeclared: Vec<&&str> = actual.difference(&declared).collect();
    let vanished: Vec<&&str> = declared.difference(&actual).collect();

    assert!(
        undeclared.is_empty(),
        "these test files reach a corpus and are not in GATES: {undeclared:#?}.\n\
         A gated test that nothing counts is a test that reports ok on every machine \
         without a corpus, which is every CI machine. Add it to the table with what it \
         gates."
    );
    assert!(
        vanished.is_empty(),
        "GATES names these files and they no longer reach a corpus: {vanished:#?}.\n\
         A table that over-reports is as useless as one that under-reports: it inflates \
         the size of the hole and nobody trusts the number."
    );
}

/// Each declared file holds the number of tests the table says, and reads the variables it
/// says.
///
/// This is what makes a *new test inside an already-declared file* visible. Without it the
/// table would be satisfied forever by listing the fourteen files once, and a fifteenth
/// gated assertion could be added to any of them and counted by nobody.
#[test]
fn Test_Every_Declared_File_Should_Match_What_It_Declares()
{
    let root = Workspace::Workspace_Root();
    let mut wrong = Vec::new();
    for gate in GATES
    {
        let disagreements = Disagreements_With(&root, gate);

        Assert_The_Row_Is_Internally_Consistent(gate);
        wrong.extend(disagreements);
    }

    assert!(
        wrong.is_empty(),
        "the table disagrees with the source: {wrong:#?}.\n\
         Update GATES deliberately. A test added to a gated file inherits that file's \
         silence, and whether it should is a decision rather than a consequence."
    );
}

/// A row cannot gate more tests than its own file holds.
fn Assert_The_Row_Is_Internally_Consistent(gate: &Gate)
{
    assert!(
        gate.gated <= gate.tests,
        "{} declares {} gated of {} tests",
        gate.path,
        gate.gated,
        gate.tests
    );
}

/// What one row says about its file, against what the file holds.
fn Disagreements_With(root: &Path, gate: &Gate) -> Vec<String>
{
    let text = Text_Of(root, gate.path);
    let tests = text
        .lines()
        .filter(|line| return line.trim() == "#[test]")
        .count();
    let named = Variables_Read_By(&text);
    let mut wrong = Vec::new();
    if tests != gate.tests
    {
        wrong.push(format!("{}: declares {} tests, has {tests}", gate.path, gate.tests));
    }
    if named != gate.variables
    {
        wrong.push(format!("{}: declares {:?}, reads {named:?}", gate.path, gate.variables));
    }

    return wrong;
}

/// One of this repository's own files, which must be readable.
fn Text_Of(root: &Path, relative: &str) -> String
{
    let path = root.join(relative);

    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// The headline is the table's own sum.
#[test]
fn Test_The_Declared_Total_Should_Be_The_Sum_Of_The_Table()
{
    let gated = GATES
        .iter()
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
    let tests = GATES
        .iter()
        .fold(0_usize, |running, gate| return running.saturating_add(gate.tests));

    assert_eq!(
        gated, GATED_TOTAL,
        "GATED_TOTAL is cited as the number of assertions a corpus-less run does not make, \
         and the table sums to {gated}"
    );
    assert_eq!(tests, TESTS_IN_GATED_FILES);
    assert!(
        gated < tests,
        "every test in every gated file is gated, which would mean the files hold nothing \
         that runs without a corpus — check the table rather than believing it"
    );
}

/// The gated count is the one figure the source can settle, so it does.
///
/// Everything else in the table was already derived and compared: the file set, the
/// variables, the test counts, the headline. `gated` was not. It was asserted to be no
/// larger than its file's test count, which `gated: 1` satisfies in every row — and it is
/// the column [`GATED_TOTAL`] sums, OD-GATE-001 cites and the gate workflow prints. The
/// most-load-bearing number here was the least checked.
///
/// The derivation resolves a test to the corpora it reaches through the helpers it calls.
/// It has to: almost no gated test names a variable itself, so counting the tests in a file
/// that mention one would find nearly none of them.
///
/// This does not replace the declaration. A derived count would let a fifteenth gated
/// assertion join the headline without anybody deciding it should, which is the silence
/// this file exists to break — one level further in.
#[test]
fn Test_Every_Declared_Gate_Count_Should_Be_The_One_In_The_Source()
{
    let derived = Gated_Tests_Per_File();

    assert!(
        !derived.is_empty(),
        "the scanner found no corpus-gated test anywhere in the workspace.\n\
         Every comparison below would then pass over an empty set, reporting that the \
         table is correct because nothing contradicted it — which is the shape of defect \
         this whole file is about."
    );

    let wrong = Rows_Disagreeing_With(&derived);

    assert!(
        wrong.is_empty(),
        "the table and the source disagree about what is gated: {wrong:#?}.\n\
         Set GATES to what the source now holds, deliberately. A test that inherits its \
         file's silence should be a decision somebody made rather than a consequence of \
         where it was written."
    );
}

/// Both directions in one list: a row the source contradicts, and a file the source gates
/// that no row names.
fn Rows_Disagreeing_With(derived: &BTreeMap<String, usize>) -> Vec<String>
{
    let declared: BTreeSet<&str> = GATES.iter().map(|gate| return gate.path).collect();
    let mut wrong = Vec::new();
    for gate in GATES
    {
        let found = derived.get(gate.path).copied().unwrap_or(0);
        if found != gate.gated
        {
            wrong.push(format!("{}: declares {} gated, source has {found}", gate.path, gate.gated));
        }
    }
    for (path, found) in derived
    {
        if !declared.contains(path.as_str())
        {
            wrong.push(format!("{path}: not in GATES, source has {found} gated"));
        }
    }

    return wrong;
}

/// The two spellings of the same three variables must not drift apart.
///
/// This file assembles the names with `concat!` so that it does not contain the strings it
/// searches for; the scanner spells them out, and excludes this crate from its own walk.
/// Two defences against the same problem, and therefore two lists — so the fact that they
/// are one fact is worth asserting. If they drift, one of them quietly stops seeing a
/// corpus and reports a smaller hole.
#[test]
fn Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables()
{
    let here: BTreeSet<&str> = VARIABLES.iter().copied().collect();
    let there: BTreeSet<&str> = CORPUS_VARIABLES.iter().copied().collect();

    assert_eq!(
        here, there,
        "this table and the scanner name different sets of corpus variables. One of them \
         has stopped looking for a corpus the other still counts, and whichever it is now \
         reports a hole smaller than the one that exists"
    );
}

/// How many gated tests the source holds, per file, repo-relative with forward slashes.
fn Gated_Tests_Per_File() -> BTreeMap<String, usize>
{
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for gate in Corpus_Gates()
    {
        let entry = counts.entry(gate.file).or_default();
        *entry = entry.saturating_add(1);
    }

    return counts;
}

/// Says how much of this suite did not run, whatever the answer is.
///
/// Prints rather than fails, and the distinction is the whole design. The corpora are not
/// in this repository and most machines will never have them; a test that failed for their
/// absence would be a gate that can never be green, which is a gate everybody learns to
/// ignore. So this reports.
///
/// It reports on the way through in both directions — a configured corpus prints zero
/// skipped, so the line is present in every log and its absence is itself visible. CI runs
/// this with `--nocapture` under its own step name, which is what puts the number beside the
/// green tick rather than behind it.
#[test]
fn Test_A_Run_Should_Report_What_It_Did_Not_Check()
{
    let (skipped, lines) = Per_Variable_Report();
    let unreachable = Assertions_No_Corpus_Reaches();

    eprintln!(
        "corpus gates: {unreachable} of {GATED_TOTAL} corpus-backed assertions did not run\n{}",
        lines.join("\n")
    );

    assert!(
        skipped >= unreachable,
        "a file cannot be skipped for more variables than it reads"
    );
}

/// One line per corpus variable, and how many assertions the unset ones carry.
fn Per_Variable_Report() -> (usize, Vec<String>)
{
    let mut skipped = 0_usize;
    let mut lines = Vec::new();
    for variable in VARIABLES
    {
        let carried = Assertions_Carried_By(variable);
        let Some(value) = std::env::var_os(variable)
        else
        {
            skipped = skipped.saturating_add(carried);
            lines.push(format!("  {variable}: unset — up to {carried} assertion(s) skipped"));
            continue;
        };
        let at = PathBuf::from(value);

        lines.push(format!("  {variable}: {} — {carried} assertion(s) reachable", at.display()));
    }

    return (skipped, lines);
}

/// The assertions the table says the files reading this variable gate.
fn Assertions_Carried_By(variable: &str) -> usize
{
    return GATES
        .iter()
        .filter(|gate| return gate.variables.contains(&variable))
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
}

/// Not the sum of the per-variable figures: a file gated on two variables is counted against
/// each, and is skipped once.
fn Assertions_No_Corpus_Reaches() -> usize
{
    return GATES
        .iter()
        .filter(|gate| {
            return gate
                .variables
                .iter()
                .any(|variable| return std::env::var_os(variable).is_none());
        })
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));
}

/// The corpus variables a file actually reads, in [`VARIABLES`] order.
///
/// Comment lines do not count. A variable named in prose is a reference and not a use —
/// this file documents all three and gates on none of them, and so may the next one. The
/// same rule as `Test_A_Capability_Id_Should_Be_Written_In_One_Crate`, for the same reason:
/// a check that cannot tell an explanation from a declaration eventually gets satisfied by
/// deleting the explanation.
fn Variables_Read_By(text: &str) -> Vec<&'static str>
{
    let code: String = text
        .lines()
        .filter(|line| return !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    return VARIABLES
        .iter()
        .filter(|variable| return code.contains(**variable))
        .copied()
        .collect();
}

/// Every `.rs` file under a workspace member's `tests/` that names a corpus variable,
/// repo-relative with forward slashes.
///
/// Walks the tree rather than asking cargo, because a test file is not a compilation target
/// cargo metadata will enumerate for us, and the question here is about files on disk.
fn Files_Naming_A_Corpus(root: &Path) -> BTreeSet<String>
{
    let workspace = Workspace::Load();
    let mut found = BTreeSet::new();
    for member in workspace.Members()
    {
        let tests = member.root.join("tests");
        for file in Rust_Files(&tests)
        {
            let named = Named_If_It_Reaches_A_Corpus(&file, root);

            found.extend(named);
        }
    }

    return found;
}

/// A file's repo-relative path with forward slashes, if it reads a corpus variable.
fn Named_If_It_Reaches_A_Corpus(file: &Path, root: &Path) -> Option<String>
{
    let text = std::fs::read_to_string(file).ok()?;
    if Variables_Read_By(&text).is_empty()
    {
        return None;
    }

    let relative = file.strip_prefix(root).ok()?;

    return Some(relative.display().to_string().replace('\\', "/"));
}

/// Every `.rs` file under a directory, recursively.
///
/// A local copy of `boundaries/common.rs`'s walk rather than a shared one. Two test binaries
/// that
/// share a helper share its failure, and these two files check different properties for
/// different reasons — the duplication is four lines and the coupling would be permanent.
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
            Keep_Or_Descend(&entry.path(), &mut pending, &mut found);
        }
    }

    return found;
}

/// A directory to walk later, a Rust file to keep, or neither.
fn Keep_Or_Descend(path: &Path, pending: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| return extension == "rs")
    {
        found.push(path.to_path_buf());
    }
}
