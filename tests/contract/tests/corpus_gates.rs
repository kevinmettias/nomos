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

use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One test file that reaches a corpus.
struct Gate
{
    /// Repo-relative, forward slashes.
    path: &'static str,
    /// The variables this file reads, sorted.
    variables: &'static [&'static str],
    /// How many of its tests do nothing when their corpus is absent.
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
        path: "crates/spec/nomos-spec-store/tests/table_rows.rs",
        variables: &[V14],
        gated: 2,
        tests: 16,
    },
    Gate {
        path: "crates/substrate/nomos-workspace/tests/portable.rs",
        variables: &[RUST],
        gated: 5,
        tests: 6,
    },
    Gate {
        path: "tests/integration/tests/analysis_slice.rs",
        variables: &[RUST],
        gated: 4,
        tests: 24,
    },
];

/// How many assertions this suite does not make when no corpus is configured.
///
/// The headline. Stated once so it can be cited, and checked against the table so it cannot
/// drift from it.
const GATED_TOTAL: usize = 68;

/// The number this is worth reading against: how many tests the gated files hold in total.
const TESTS_IN_GATED_FILES: usize = 125;

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

    let declared: BTreeSet<&str> = GATES.iter().map(|gate| return gate.path).collect();
    let actual: BTreeSet<&str> = found.iter().map(String::as_str).collect();

    let undeclared: Vec<&&str> = actual.difference(&declared).collect();
    assert!(
        undeclared.is_empty(),
        "these test files reach a corpus and are not in GATES: {undeclared:#?}.\n\
         A gated test that nothing counts is a test that reports ok on every machine \
         without a corpus, which is every CI machine. Add it to the table with what it \
         gates."
    );

    let vanished: Vec<&&str> = declared.difference(&actual).collect();
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
        let path = root.join(gate.path);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        let tests = text
            .lines()
            .filter(|line| return line.trim() == "#[test]")
            .count();
        if tests != gate.tests
        {
            wrong.push(format!(
                "{}: declares {} tests, has {tests}",
                gate.path, gate.tests
            ));
        }

        let named = Variables_Read_By(&text);
        if named != gate.variables
        {
            wrong.push(format!(
                "{}: declares {:?}, reads {named:?}",
                gate.path, gate.variables
            ));
        }

        assert!(
            gate.gated <= gate.tests,
            "{} declares {} gated of {} tests",
            gate.path,
            gate.gated,
            gate.tests
        );
    }

    assert!(
        wrong.is_empty(),
        "the table disagrees with the source: {wrong:#?}.\n\
         Update GATES deliberately. A test added to a gated file inherits that file's \
         silence, and whether it should is a decision rather than a consequence."
    );
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
    let mut skipped = 0_usize;
    let mut lines = Vec::new();

    for variable in VARIABLES
    {
        let carried = GATES
            .iter()
            .filter(|gate| return gate.variables.contains(variable))
            .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));

        if let Some(value) = std::env::var_os(variable)
        {
            lines.push(format!(
                "  {variable}: {} — {carried} assertion(s) reachable",
                PathBuf::from(value).display()
            ));
        }
        else
        {
            skipped = skipped.saturating_add(carried);
            lines.push(format!("  {variable}: unset — up to {carried} assertion(s) skipped"));
        }
    }

    // Not the sum of the per-variable figures: a file gated on two variables is counted
    // against each, and is skipped once.
    let unreachable = GATES
        .iter()
        .filter(|gate| {
            return gate
                .variables
                .iter()
                .any(|variable| return std::env::var_os(variable).is_none());
        })
        .fold(0_usize, |running, gate| return running.saturating_add(gate.gated));

    eprintln!(
        "corpus gates: {unreachable} of {GATED_TOTAL} corpus-backed assertions did not run\n{}",
        lines.join("\n")
    );

    assert!(
        skipped >= unreachable,
        "a file cannot be skipped for more variables than it reads"
    );
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
        if !tests.is_dir()
        {
            continue;
        }

        for file in Rust_Files(&tests)
        {
            let Ok(text) = std::fs::read_to_string(&file)
            else
            {
                continue;
            };

            if Variables_Read_By(&text).is_empty()
            {
                continue;
            }

            let Ok(relative) = file.strip_prefix(root)
            else
            {
                continue;
            };

            found.insert(relative.display().to_string().replace('\\', "/"));
        }
    }

    return found;
}

/// Every `.rs` file under a directory, recursively.
///
/// A local copy of `boundaries.rs`'s walk rather than a shared one. Two test binaries that
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
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| return extension == "rs")
            {
                found.push(path);
            }
        }
    }

    return found;
}
