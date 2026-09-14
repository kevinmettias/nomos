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
//!
//! # Why this file declares modules through `#[path]`
//!
//! `.claude/skills/nomos-task/SKILL.md` names `tests/contract/tests/corpus_gates.rs` in a
//! code span, and `agent_harness.rs` checks that every path the agent surface names is real.
//! So this file cannot become `corpus_gates/main.rs`: the split would turn that check red
//! against a document outside this item's territory. A test binary root resolves a bare
//! `mod x;` against its own directory, which cargo would then build as a second test target,
//! so `#[path]` is what lets the root keep the name the skill routes to.
//!
//! What stays here is the declaration — the table, the two headline figures, and the three
//! variable names this file assembles with `concat!` rather than spelling. What moved out is
//! the checking: [`declared`] compares the table against the tree and against its own rows,
//! [`derived`] compares it against what the source actually gates, [`report`] says how much
//! of the suite did not run, and [`scan`] is the walk all of them read the tree through.

#[path = "corpus_gates/declared.rs"]
mod declared;
#[path = "corpus_gates/derived.rs"]
mod derived;
#[path = "corpus_gates/report.rs"]
mod report;
#[path = "corpus_gates/scan.rs"]
mod scan;

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
        // All six tests of the suite are gated, so all six are in this one module with the
        // root that gates them; what moved out was the walk and the machinery, not a claim.
        path: "crates/languages/nomos-lang-rust/tests/corpus/claims.rs",
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
        // The suite's other four tests keep the register honest about itself and read no
        // corpus, so they are in a sibling module and out of this count.
        path: "crates/spec/nomos-spec-ingest/tests/family_counts/measurement.rs",
        variables: &[ARCHIVES, V14],
        gated: 2,
        tests: 2,
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
        gated: 5,
        tests: 5,
    },
    Gate {
        // The suite's other five tests check the checked-in register against itself and
        // read no archive, so they are in a sibling module and out of this count.
        path: "crates/spec/nomos-spec-ingest/tests/regression_report/archives.rs",
        variables: &[ARCHIVES],
        gated: 9,
        tests: 9,
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
        // Unchanged by P9-FALLBACK, deliberately. The five assertions it added are about
        // spending the selection per subject and are written against the precision corpus,
        // which is in this repository — so the hole this table measures did not grow.
        //
        // The suite is `tests/analysis_slice/`, and this is the only module of it that reaches
        // a corpus. Two of its four tests belong to the providers and lessons modules by
        // subject and are here because the derivation below follows calls within one file.
        path: "tests/integration/tests/analysis_slice/scale.rs",
        variables: &[RUST],
        gated: 4,
        tests: 4,
    },
    Gate {
        // Both tests strip NOMOS_V14_CORPUS deliberately and assert the no-corpus behaviour
        // directly, so neither does nothing in its absence -- the scanner finds the variable
        // name because Run_Without_Corpus names it to remove it, not because a test skips.
        path: "crates/host/nomos-cli/tests/spec_model_seam.rs",
        variables: &[V14],
        gated: 0,
        tests: 2,
    },
    Gate {
        // Same shape as spec_model_seam.rs, one file over: Run_Without_Corpus strips the
        // variable to test the absence path, and does nothing silently in neither test.
        path: "crates/host/nomos-cli/tests/spec_orchestration_seam.rs",
        variables: &[V14],
        gated: 0,
        tests: 1,
    },
];

/// How many assertions this suite does not make when no corpus is configured.
///
/// The headline. Stated once so it can be cited, and checked against the table so it cannot
/// drift from it.
const GATED_TOTAL: usize = 69;

/// The number this is worth reading against: how many tests the gated files hold in total.
///
/// Rose to 130 with `P9-FALLBACK`'s five assertions in `analysis_slice.rs`, none of which
/// reads a corpus. [`GATED_TOTAL`] is unchanged, and that is the point of keeping the two
/// numbers apart: a gated file growing is not the hole growing.
///
/// Fell to 81 as the test crates were decomposed for `check-file-size`: forty-nine of the
/// tests inside gated files never read a corpus — fourteen of `table_rows.rs`'s sixteen, one
/// of `portable.rs`'s six, five of `regression_report.rs`'s fourteen, four of
/// `family_counts.rs`'s six, twenty-five of `analysis_slice.rs`'s twenty-nine — and each now
/// lives in a sibling module rather than inflating this denominator. That is the same move in
/// the other direction, and it is why the ratio to read is no longer 68 of 130: the
/// denominator was measuring file boundaries as much as it was measuring gated code.
/// [`GATED_TOTAL`] is unchanged through every one of those splits, which is the check that
/// they moved tests rather than silence.
///
/// Rose to 84 as two files the scanner had always reached went undeclared until now:
/// `spec_model_seam.rs` and `spec_orchestration_seam.rs` both name the variable only to strip
/// it (`Run_Without_Corpus`'s `env_remove`), so neither gates a test. [`GATED_TOTAL`] is
/// unchanged — three tests added to the denominator, none of them silent.
const TESTS_IN_GATED_FILES: usize = 85;
