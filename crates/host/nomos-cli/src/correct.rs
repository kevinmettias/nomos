//! `nomos correct` — a thin renderer over `nomos-correction-orchestration`'s
//! [`Run_Correction`](nomos_correction_orchestration::Run_Correction), the same shape
//! `gate.rs` already is over `nomos-gate-orchestration`'s `Run_Gate`.
//!
//! `P40-CORRECTIONS-CANONICAL-SEAM` moved this module's own former composition — walk,
//! judge, find the one claim, plan, seed a workspace, stage, validate, and optionally
//! commit — into `nomos-correction-orchestration`, so `nomos-api` could reach the same
//! lifecycle without depending on this crate. What is left here is exactly what `gate.rs`'s
//! own doc names for `Gate`: walking the tree ([`Correction_Sources`] — `nomos-workspace-discovery`'s
//! own [`Walked_Sources`], the one walk `OD-HOST-008` put beneath every composition root,
//! which `check`, `gate`, `workflow` and this module now all call rather than each carrying
//! its own), reading what this binary was compiled as
//! ([`Correction_Variant`]), choosing a
//! [`nomos_platform::ProcessLauncher`]/[`nomos_platform::FileSystem`]
//! ([`nomos_composer_std::LAUNCHER`]/[`nomos_composer_std::FILE_SYSTEM`]) for
//! the seam's own two platform ports, and rendering a
//! [`nomos_correction_orchestration::CorrectionOutcome`] into the exact text and
//! [`ExitCode`] this command always reported -- verified unchanged against this module's
//! own test suite, which predates the move and was not rewritten for it.

mod exit_code;
mod parsing;
#[cfg(test)]
mod tests;

pub(crate) use exit_code::ExitCode;
pub use parsing::Correct_Command_From_String_Arguments;

use crate::arguments::Named_Value_From_String_Arguments;
use nomos_correction_orchestration::{CorrectionCommand as SeamCommand, CorrectionEnvironment, CorrectionOutcome, Run_Correction};
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use nomos_workspace_discovery::{Registered_Extensions, Walked_Sources};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What `nomos correct phantom-mirrors` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CorrectCommand
{
    pub root: PathBuf,
    /// Whether to actually commit and write the corrected file, or stop after staging and
    /// validating it.
    pub commit: bool,
}

/// Runs the correction and renders what it did.
pub fn Run(command: &CorrectCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let walked = Correction_Sources(&command.root);
    let seam_command = SeamCommand { root: command.root.clone(), commit: command.commit };
    let environment = CorrectionEnvironment { variant: Correction_Variant(), launcher: &LAUNCHER, filesystem: &FILE_SYSTEM, environment: &ENVIRONMENT };

    let outcome = Run_Correction(walked, environment, &seam_command);

    return Rendered_Correction_Outcome(&outcome, &command.root, stdout, stderr);
}

/// The Rust and Go sources under `root`, or `None` if `root` is not a directory -- the
/// walk `Run_Correction` itself does not do, the same division `gate.rs` keeps with
/// `Run_Gate`.
///
/// [`Registered_Extensions`] is what this module used to spell out as a two-extension test:
/// Rust and Go, because `Check_Completeness_Mirrors` needs every `.rs` and `.go` file under
/// the root, since a check name claimed in one language's file can be declared in the
/// other's. Naming the registry rather than the extensions is what lets a language this
/// workspace registers reach this walk without an edit here -- and it is the same argument
/// the shared walk's own doc gives for holding the population rather than each composition
/// root holding a copy of it.
fn Correction_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    return Walked_Sources(root, &Registered_Extensions());
}

/// The build variant this binary was compiled as. A near-duplicate of `check::composition::
/// Host_Variant`, not a shared dependency on it: that function is `pub(super)` to `check`, so
/// sharing it costs a visibility boundary or a hoist to a crate beneath both, and neither has
/// been taken. The walk this module once argued the same way about is not this case -- it is
/// `nomos-workspace-discovery`'s one, which `OD-HOST-008` put beneath every composition root
/// and [`Correction_Sources`] now calls.
fn Correction_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
}

/// Renders `outcome` into the exact text and [`ExitCode`] this command has always
/// reported, one variant of [`CorrectionOutcome`] at a time.
///
/// Each arm is one case: the outcome on the left, the sentence it renders and the code it
/// answers with on the right. The bytes are unchanged from the written-out form this
/// replaced -- [`Announced`] ends its text with the same newline `writeln!` did.
fn Rendered_Correction_Outcome(outcome: &CorrectionOutcome, root: &Path, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        CorrectionOutcome::UnreadableRoot =>
            Announced_Line(stderr, ExitCode::Unreadable, format!("`{}` is not a directory", root.display())),
        CorrectionOutcome::NoSourceFound =>
            Announced_Line(stderr, ExitCode::Vacuous, format!("no `.rs` or `.go` source found under `{}`", root.display())),
        CorrectionOutcome::UnreadableWorkspaceState =>
            Announced_Line(stderr, ExitCode::Vacuous, "the tree could not be read as a workspace state".to_owned()),
        CorrectionOutcome::ContradictoryRegistry(error) =>
            Announced_Line(stderr, ExitCode::Vacuous, format!("this build's own capability registry is self-contradictory: {error}")),
        CorrectionOutcome::NoFactsMaterialized(files) =>
            Announced_Line(stderr, ExitCode::Vacuous, format!("{files} file(s) were read but no syntax fact was materialized for any of them")),
        CorrectionOutcome::Refused(reason) =>
            Announced_Line(stderr, ExitCode::Refused, reason.clone()),
        CorrectionOutcome::Clean =>
            Announced_Line(stdout, ExitCode::Ok, format!("clean: no blocking correction claim under `{}`", root.display())),
        CorrectionOutcome::Staged { path, summary, preview } =>
            Announced_Line(stdout, ExitCode::Ok, format!("{}\ndry run: `{path}`: {summary}. Pass --commit to apply it.", String::from_utf8_lossy(preview))),
        CorrectionOutcome::Committed { path, summary, preview, base, after_snapshot } =>
            Announced_Line(stdout, ExitCode::Ok, format!("{}\ncommitted: `{path}`: {summary} ({base} -> {after_snapshot})", String::from_utf8_lossy(preview))),
    };
}

/// Writes `text` to `stream` as one line and answers with `code` -- the two steps every arm
/// of [`Rendered_Correction_Outcome`] ends in, so that each of those arms states one case rather than three.
fn Announced_Line(stream: &mut impl Write, code: ExitCode, text: String) -> ExitCode
{
    let _ = writeln!(stream, "{text}");
    return code;
}
