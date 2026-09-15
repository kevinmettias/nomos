//! `nomos check` pointed at this repository, which is the tree the gate step judges.
//!
//! Everything else in this suite runs the command over a scratch tree the suite builds and
//! knows the answer for. What lives here is the one subject no test can arrange: the tree CI
//! walks, which is this commit's workspace. These assertions are therefore true only of this
//! repository's current contents, and they are what turns a phantom introduced anywhere in it
//! into a red pull request.
//!
//! A module of its own because that is the whole difference. The command, the report and
//! the exit code are the same ones the tests beside this file assert; what changes is the
//! subject, and a subject nobody can construct at test time is a separate claim to keep.

use crate::{Ran, Repository_Root, Run};

/// The fewest files a run over this repository must have examined to have looked at it.
///
/// A floor in the shape `FEWEST_GOVERNING_RECORDS` uses, not a count: any item may raise it
/// and only a deliberate deletion lowers it. It exists because
/// [`Test_This_Workspace_Should_Have_Nothing_That_Can_Fail_A_Build`] would otherwise be
/// satisfied by a run over an empty tree — this repository's most-repeated defect appearing
/// inside the test that guards against it. Measured at 195 files on the commit that wired
/// the gate step.
const FEWEST_FILES_IN_THIS_WORKSPACE: usize = 150;

/// The `[Blocking]` findings this workspace's own tree carries today, and why each is
/// accepted rather than fixed -- the identical allowlist `crates/host/nomos-cli/src/gate/
/// tests.rs`'s own `ACCEPTED_BLOCKING_FINDINGS` carries, duplicated here rather than shared
/// because the two live in different crates (a library's own unit tests against a separate
/// integration-test binary) with no existing shared test-support dependency between them.
/// Being a copy, it goes stale the same way and on the same commits; `P96` is where both
/// were last emptied together.
///
/// Empty today, and that is the assertion rather than the absence of one: with no entry,
/// [`Only_Accepted_Findings_Are_Blocking`] means this workspace's own tree carries no
/// `[Blocking]` finding at all, which is strictly stronger than the two-named version it
/// replaces.
///
/// Both former entries were against the same file,
/// `tests/integration/fixtures/third-party/hex-0.4.3/lib.rs`, and they left for different
/// reasons. `single-letter-names: T` was a real gap and was fixed: `OD-CAPABILITY-014` put an
/// `impl` block's own generic parameters in the syntax payload, and the rule now exempts an
/// `Implementation` item whose own name is one of them (`P96`, `c278d896`). `abbreviations:
/// val` was *not* fixed and never will be -- this list's own previous text called it a
/// permanent, deliberate true positive, because `hex`'s author really did choose that name --
/// but that reasoning was always about the finding and never about whether this repository
/// walks the file. `P96` (`c3ff169e`) stopped the shared walk descending into a directory
/// carrying its own `standards.json`, so the vendored fixture is no longer judged from this
/// root at all, and `tests/integration/tests/calibration.rs` still judges it from its own.
///
/// A named allowlist rather than a bare count: a finding accepted the same deliberate way
/// must be added here explicitly, and anything not named here fails these tests -- neither
/// `nomos_gate_orchestration::Suppression` nor `RuleCalibration` is wired to a real config
/// file yet, so this allowlist is what stands in for that mechanism.
const ACCEPTED_BLOCKING_FINDINGS: &[&str] = &[];

/// Whether `output` carries no `[Blocking]` line other than the ones
/// [`ACCEPTED_BLOCKING_FINDINGS`] names.
fn Only_Accepted_Findings_Are_Blocking(output: &str) -> bool
{
    return output
        .lines()
        .filter(|line| return line.starts_with("[Blocking]"))
        .all(|line| return ACCEPTED_BLOCKING_FINDINGS.iter().any(|accepted| return line.starts_with(accepted)));
}

/// The gate step, one layer earlier: this workspace has nothing blocking beyond
/// [`ACCEPTED_BLOCKING_FINDINGS`]'s own two, named and accepted deliberately.
///
/// `OD-GATE-004` wired `cargo run … check --root .` into the gate, so from now on a phantom
/// introduced anywhere in this repository turns a pull request red. This asserts the same
/// thing at `cargo test` time, so the author finds out before pushing rather than after — and
/// it is the acceptance test for the step being green on the day it landed. `P71-GATE-TESTS-
/// OWN-CLEAN-TREE-ASSERTION-IS-STALE-2` measured this workspace's own tree carries exactly
/// two `[Blocking]` findings today, both named above; a bare "0 of which can fail a build"
/// assertion (this test's own shape before that item) stopped being true the moment the
/// first of the two was committed, and CI's own `Test` step failed on every push since,
/// unnoticed because nothing local ran this exact test's own predicate.
///
/// The file-count floor is not decoration. Without it this assertion is satisfied by a run
/// over an empty tree, which is this repository's most-repeated defect appearing inside the
/// test that guards against it — the same reason `boundaries.rs` carries
/// `Test_The_Workspace_Should_Not_Appear_Empty`.
///
/// It walks the real tree, so `cargo test -p nomos-cli` now depends on this repository's
/// contents. That is already true of `governing_records_are_present.rs` and of
/// `gate_covers_finish.rs`, and the run costs about a second.
#[test]
fn Test_This_Workspace_Should_Have_Nothing_That_Can_Fail_A_Build()
{
    let root = Repository_Root();
    let Ran { code, said: output } = Run(&["check", "--root", &root.display().to_string()]);
    assert!(
        output.contains("file(s) examined"),
        "the run did not reach its own report, so nothing below is about this workspace: \
         {output}"
    );
    let examined = Files_Examined(&output);

    assert!(
        examined >= FEWEST_FILES_IN_THIS_WORKSPACE,
        "{examined} file(s) examined under {}, and this workspace holds no fewer than \
         {FEWEST_FILES_IN_THIS_WORKSPACE}. A run over a tree this small has not looked at \
         this repository, so the clean verdict below would mean nothing",
        root.display()
    );
    assert!(
        Only_Accepted_Findings_Are_Blocking(&output),
        "a Blocking finding exists that ACCEPTED_BLOCKING_FINDINGS does not name -- a real, \
         new regression, or an accepted finding whose exact rendered text drifted: {output}"
    );
    assert_eq!(
        code, 0,
        "this workspace's own tree carries nothing that can fail a build today, and \
         ACCEPTED_BLOCKING_FINDINGS is empty to say so; a non-zero code here is either a real \
         regression or a finding somebody meant to accept without naming it. The file count \
         asserted above is what keeps this 0 from being a run over the wrong tree: {output}"
    );
}

/// The count the report opens with, which is how a run over the wrong tree is told from a run
/// over this one.
fn Files_Examined(output: &str) -> usize
{
    return output
        .split_once(" file(s) examined")
        .and_then(|(before, _)| {
            return before
                .split_whitespace()
                .next_back()
                .and_then(|count| return count.parse().ok());
        })
        // A missing count cannot be reported as zero: zero is a meaningful answer here — it is
        // what a run over the wrong tree prints — and the callers of this function compare the
        // number to decide exactly that. Refusing to invent one keeps "the report changed
        // shape" from being read as "the run examined nothing". The whole report is printed
        // because the count's absence is only diagnosable next to what was printed instead.
        .unwrap_or_else(|| panic!("the report must carry a file count: {output}"));
}
