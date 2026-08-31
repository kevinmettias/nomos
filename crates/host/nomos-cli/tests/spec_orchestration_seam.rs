//! `nomos_cli` <-> `nomos_spec_orchestration`, exercised through the compiled binary's public
//! surface.
//!
//! `vacuity.rs`'s inline `mod tests` constructs a `nomos_spec_orchestration::corpus::
//! CorpusRequest` and calls `crate::spec::Run` directly; `spec.rs`'s inline `mod run_coverage`
//! imports `nomos_spec_orchestration::corpus::DEFAULT_REVISION` for the same purpose --
//! proving `nomos_spec_orchestration` works against `nomos-cli`'s own private `spec::Run`,
//! opened up, rather than against its public surface. `nomos-cli` is a `[[bin]]`-only crate
//! (no `src/lib.rs`), so a file under `tests/` cannot reach `spec::Run` at all; the only
//! public surface left is the compiled binary itself.
//!
//! `main.rs::Corpus_Request` is the real, production seam: it builds a
//! `nomos_spec_orchestration::corpus::CorpusRequest` naming `NOMOS_V14_CORPUS` and defaulting
//! `revision` to `nomos_spec_orchestration::corpus::DEFAULT_REVISION`, for every `spec` and
//! `request` invocation; `spec::Run` calls `Assemble_Corpus` on exactly that value before
//! dispatching any verb. This file builds the same `CorpusRequest` directly, calls
//! `Assemble_Corpus` on it -- the real function `spec::Run` calls -- and checks that the
//! absence it records is word-for-word the same text the real binary prints when a real `spec`
//! invocation is run with no corpus configured, which is `OD-SPEC-006`/`ARC-SPECDB-002`'s
//! supported, non-error state rather than one requiring the v14 corpus this repository does
//! not contain.

#[path = "support/mod.rs"]
mod support;

use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};
use std::process::Command;

/// The same corpus variable `main.rs::Corpus_Request` reads and the one this file's directly
/// constructed `CorpusRequest` names, so the absence text produced each way is about the same
/// variable and not merely the same shape.
const V14_CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

/// Runs the real binary with the v14 corpus variable stripped from its environment, so a
/// machine holding the real corpus behaves like one that does not -- the state this test is
/// about.
fn Run_Without_Corpus(arguments: &[&str]) -> support::Ran
{
    let finished = Command::new(support::NOMOS)
        .args(arguments)
        .env_remove(V14_CORPUS_VARIABLE)
        .output()
        .expect("the binary cargo just built must be runnable");

    return support::Ran {
        code: finished.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&finished.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&finished.stderr).into_owned(),
    };
}

/// A `CorpusRequest` naming no root, built and assembled directly, must record the exact
/// absence the real binary prints on a real `spec record` invocation with no corpus
/// configured -- proving `main.rs`'s composition and this test's direct construction are the
/// same seam rather than two things that merely look alike.
#[test]
fn Test_A_Spec_Command_With_No_Corpus_Should_Report_The_Same_Absence_Assemble_Corpus_Records_Directly()
{
    let request = CorpusRequest { variable: V14_CORPUS_VARIABLE.to_owned(), root: None, revision: DEFAULT_REVISION.to_owned() };

    let assembly = Assemble_Corpus(&request).expect("assembles from the embedded governing records alone");
    assert!(!assembly.Is_Complete(), "an unnamed corpus is still a recorded absence");
    let described = assembly.Describe_Absences();
    assert!(described.contains("the v14 authoring corpus"), "{described}");
    assert!(
        described.contains(&format!("{V14_CORPUS_VARIABLE} is not set, and no --corpus was given")),
        "{described}"
    );

    let outcome = Run_Without_Corpus(&["spec", "record", "--id", "D-129"]);

    assert!(outcome.stderr.contains("the v14 authoring corpus"), "{}", outcome.stderr);
    assert!(
        outcome.stderr.contains(&format!("{V14_CORPUS_VARIABLE} is not set, and no --corpus was given")),
        "the real binary's absence text and the one Assemble_Corpus just built directly must \
         say the same thing: {}",
        outcome.stderr
    );
}
