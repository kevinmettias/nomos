//! `nomos check` — run the rules over a tree and report what they find.
//!
//! The composition root for `nomos-rules`. That crate takes its subject as an argument
//! and deliberately cannot read a disk; this module is where the tree is decided, walked
//! and handed over, which is the seam that lets the same rule be run against the real
//! workspace here and against code that no longer exists in a test.
//!
//! # What a composition root has to assemble now
//!
//! Four things, and none of them is decoration. `OD-RULES-001` moved check-name
//! resolution onto the fact layer, so this command:
//!
//! 1. ingests the walk through [`Workspace::Apply`], the one door a workspace state comes
//!    through, which is where the snapshot and generation every fact is filed under come
//!    from;
//! 2. declares [`nomos_cap_syntax::Capability_Contract`] and registers
//!    [`nomos_lang_rust::Provider_Offer`] — a *floor* is stated by the rule and the
//!    registry decides who serves it, so this root registers providers and does not pick
//!    one;
//! 3. materializes one syntax fact per file into a [`MemoryFactStore`];
//! 4. runs the rule over a [`Reader`] on that store.
//!
//! `nomos-lang-rust-scan` is not registered. It offers the same capability at
//! `FactVariant::Approximate` and `Assurance::Unsound`, which is below
//! `nomos_rules::Syntax_Requirement`; registering it would make the refusal a runtime
//! event nobody sees rather than a decision written down here.
//!
//! # The vacuity guard lives here, and it now has two shapes
//!
//! A rule that finds no subjects returns no findings, which renders identically to a
//! clean run. That is the defect the sibling workspace recorded four times over —
//! `check-standards-tree /nonexistent` walked nothing, found nothing and reported CLEAN —
//! and `boundaries.rs` already guards its own assertions against it. The guard belongs
//! here rather than in the rule, because "did I see a plausible amount of the world" is a
//! question only the caller that chose the tree can answer.
//!
//! The second shape arrived with the fact layer: a walk that found source and materialized
//! no fact for any of it. That is the same lie one layer in — the rule would resolve every
//! mirror claim against an empty index and, because the index is empty, would refuse to
//! block on any of them. Both exit [`ExitCode::Vacuous`], which already means "the answer
//! is empty because something expected was not there". No seventh code: an exit code means
//! one thing per binary, and this is that thing.
//!
//! Where the vacuity guard *belongs* was a live question; `OD-GATE-003` answers it. The
//! guard stays here, decided by this caller, for the reason given above — but `vacuity.rs`
//! is now the one place a reader finds that this module and `spec.rs` both made that
//! decision, rather than discovering it twice by reading two doc comments.
//!
//! # This command is a gate step now, and that changes what an exit code costs
//!
//! `OD-GATE-004` wired the `Rules` step of `.github/workflows/gate.yml` to
//! `cargo run --quiet -p nomos-cli --bin nomos -- check --root .`. Until then nothing in
//! CI ran this command, so the rule layer enforced nothing: a broken walk, a provider that
//! stopped answering, a rotted argv or a panic out of [`Registered`] all shipped green.
//!
//! **Zero is the only success, and the workflow says so by containing no branch.** Actions
//! fails a step on any non-zero exit, and that default *is* the policy — so the numbers
//! below stay spelled here, once, rather than being restated in YAML where they would go
//! stale against this enum. What each code now does to a pull request:
//!
//! - [`ExitCode::Ok`] — the rules ran over the workspace, materialized facts, and nothing
//!   they found can fail a build. Not "found nothing": twelve admitted gaps and the two
//!   `tests/corpus/analysis/gamma/broken.rs` advisories print on every run, and the counts
//!   line carries both denominators. **Green.**
//! - [`ExitCode::Violations`] — the arm the step exists for. Reachable only since
//!   `OD-RULES-002` made incompleteness a property of the claim rather than of the run;
//!   before that a phantom anywhere in this workspace was downgraded by `broken.rs` and the
//!   step could not have failed for its own reason. **Red.**
//! - [`ExitCode::Usage`] — from a step, this means *the workflow's own argv is wrong*. A
//!   gate that mistypes its invocation and passes is a gate checking nothing. **Red.**
//! - [`ExitCode::Unreadable`] — no tree, so nothing was checked. **Red.**
//! - [`ExitCode::Vacuous`] — the one this wiring is really about. A gate treating "I judged
//!   nothing" as success would be `OD-GATE-001`'s defect installed one level up from where
//!   that record found it, this time with a green tick beside it. **Red.**
//! - anything else — `101` from a build failure or from [`Registered`]'s own `expect`, a
//!   signal, a truncation. Absence, unknown and error must not become success, and the
//!   default gives that for free. **Red.**
//!
//! Two consequences for anybody editing this module. A sixth code must survive
//! `main`'s `u8::try_from(code).unwrap_or(1)`, or a distinct outcome arrives at CI as an
//! ordinary violation. And [`ExitCode::Vacuous`] must never be renumbered to `0` "because
//! there is nothing to report" — `Test_Only_Ok_Should_Carry_The_Passing_Exit_Code` below is
//! the whole exit-code policy as one assertion, and it is where that would go red.
//!
//! No corpus is involved. This command reads no environment variable at run time —
//! `main.rs` passes it none — so a CI runner with no `NOMOS_*` set produces the full
//! answer. Measured. What it does need is what `build.rs` baked in for
//! [`Host_Variant`], and a runner that cannot supply those cannot link the binary and
//! fails at `Lint` long before this step.

mod parsing;
mod sources;
mod composition;
mod facts;
mod report;
#[cfg(test)]
mod tests;

pub use parsing::Parse;
use sources::Walked;
use composition::{Host_Variant, Registered, Resolved_Configuration};
use facts::{Materialize_Syntax, Nothing_Materialized, Prepare};
use report::{Examined, Report};

mod exit_code;
mod command;

pub(crate) use exit_code::ExitCode;
pub(crate) use command::CheckCommand;

use crate::arguments::Named_Value;
// check-dependency-placement reports three of this crate's edges as this file's alone, and
// says the same about corpus.rs for a fourth. Both readings are right and neither is a
// misplacement: this is a composition root, so one verb per module and one module per set
// of crates it composes is the shape, and an edge that belonged to more than one verb would
// mean two verbs were doing the same thing.
use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{CapabilityId, ConfigurationId, Finding, Guarantee};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_model::{Content_Digest, Subject_Of_Path};
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Runs the rules and renders what they say.
pub fn Run(command: &CheckCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let prepared = match Prepare(&command.root, stderr)
    {
        Ok(prepared) => prepared,
        Err(code) => return code,
    };

    let mut store = MemoryFactStore::New();
    let facts = Materialize_Syntax(&prepared.sources, &prepared.context, &mut store);
    if facts == 0
    {
        return Nothing_Materialized(prepared.sources.len(), &command.root, stderr);
    }

    let mut reader = Reader::On(&store, &prepared.registry, prepared.context);
    let findings = Check_Completeness_Mirrors(&prepared.sources, &mut reader);
    let examined = Examined {
        files: prepared.sources.len(),
        facts,
    };

    return Report(&findings, examined, stdout);
}
