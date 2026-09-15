//! `nomos correct` — a thin renderer over `nomos-correction-orchestration`'s
//! [`Run_Correction`](nomos_correction_orchestration::Run_Correction), the same shape
//! `gate.rs` already is over `nomos-gate-orchestration`'s `Run_Gate`.
//!
//! `P40-CORRECTIONS-CANONICAL-SEAM` moved this module's own former composition — walk,
//! judge, find the one claim, plan, seed a workspace, stage, validate, and optionally
//! commit — into `nomos-correction-orchestration`, so `nomos-api` could reach the same
//! lifecycle without depending on this crate. What is left here is exactly what `gate.rs`'s
//! own doc names for `Gate`: walking the tree ([`Walked`], a near-duplicate of
//! `check::sources` rather than a shared dependency on it — `OD-HOST-002`'s own division
//! of a composition root's territory is per group, not shared through a third module
//! neither group's item reserved), reading what this binary was compiled as
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
pub use parsing::Parse;

use crate::arguments::Named_Value_From_String_Arguments;
use nomos_correction_orchestration::{CorrectionCommand as SeamCommand, CorrectionEnvironment, CorrectionOutcome, Run_Correction};
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
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
    let walked = Walked(&command.root);
    let seam_command = SeamCommand { root: command.root.clone(), commit: command.commit };
    let environment = CorrectionEnvironment { variant: Correction_Variant(), launcher: &LAUNCHER, filesystem: &FILE_SYSTEM, environment: &ENVIRONMENT };

    let outcome = Run_Correction(walked, environment, &seam_command);

    return Rendered(&outcome, &command.root, stdout, stderr);
}

/// The Rust and Go sources under `root`, or `None` if `root` is not a directory -- the
/// walk `Run_Correction` itself does not do, the same division `gate.rs` keeps with
/// `Run_Gate`.
fn Walked(root: &Path) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root));
}

/// Every `.rs` or `.go` file under `root`, with its text and the subject its facts are
/// filed under. `target` is skipped: it holds generated source nobody authored.
fn Read_Sources(root: &Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
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
            Read_Entry(root, path, &mut pending, &mut sources);
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

fn Read_Entry(root: &Path, path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        let skipped = path.file_name().is_some_and(|name| return name == "target" || name == ".git");

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(Is_Recognized_Extension) && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
}

fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    use nomos_model::Subject_Of_Path;

    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
fn Relative(root: &Path, path: &Path) -> String
{
    return path.strip_prefix(root).unwrap_or(path).display().to_string().replace('\\', "/");
}

/// The build variant this binary was compiled as. A near-duplicate of `check::composition::
/// Host_Variant`, not a shared dependency on it: that function is `pub(super)` to `check`,
/// the same reasoning `sources.rs`'s own doc gives for not sharing that module's walk.
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
fn Rendered(outcome: &CorrectionOutcome, root: &Path, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        CorrectionOutcome::UnreadableRoot =>
            Announced(stderr, ExitCode::Unreadable, format!("`{}` is not a directory", root.display())),
        CorrectionOutcome::NoSourceFound =>
            Announced(stderr, ExitCode::Vacuous, format!("no `.rs` or `.go` source found under `{}`", root.display())),
        CorrectionOutcome::UnreadableWorkspaceState =>
            Announced(stderr, ExitCode::Vacuous, "the tree could not be read as a workspace state".to_owned()),
        CorrectionOutcome::ContradictoryRegistry(error) =>
            Announced(stderr, ExitCode::Vacuous, format!("this build's own capability registry is self-contradictory: {error}")),
        CorrectionOutcome::NoFactsMaterialized(files) =>
            Announced(stderr, ExitCode::Vacuous, format!("{files} file(s) were read but no syntax fact was materialized for any of them")),
        CorrectionOutcome::Refused(reason) =>
            Announced(stderr, ExitCode::Refused, reason.clone()),
        CorrectionOutcome::Clean =>
            Announced(stdout, ExitCode::Ok, format!("clean: no blocking correction claim under `{}`", root.display())),
        CorrectionOutcome::Staged { path, summary, preview } =>
            Announced(stdout, ExitCode::Ok, format!("{}\ndry run: `{path}`: {summary}. Pass --commit to apply it.", String::from_utf8_lossy(preview))),
        CorrectionOutcome::Committed { path, summary, preview, base, after_snapshot } =>
            Announced(stdout, ExitCode::Ok, format!("{}\ncommitted: `{path}`: {summary} ({base} -> {after_snapshot})", String::from_utf8_lossy(preview))),
    };
}

/// Writes `text` to `stream` as one line and answers with `code` -- the two steps every arm
/// of [`Rendered`] ends in, so that each of those arms states one case rather than three.
fn Announced(stream: &mut impl Write, code: ExitCode, text: String) -> ExitCode
{
    let _ = writeln!(stream, "{text}");
    return code;
}

/// Rust or Go -- `Check_Completeness_Mirrors` needs every `.rs` and `.go` file under the
/// root, since a check name claimed in one language's file can be declared in the other's.
fn Is_Recognized_Extension(extension: &std::ffi::OsStr) -> bool
{
    return extension == "rs" || extension == "go";
}
