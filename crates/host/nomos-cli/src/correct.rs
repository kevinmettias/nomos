//! `nomos correct` — the corrections product vertical's first real Finding-to-Commit path.
//!
//! `OD-CORRECTIONS-001` found `nomos-corrections` real (a tested `Preview -> Stage ->
//! Validate -> Commit -> Rollback` lifecycle over `nomos_workspace::Workspace`'s one door)
//! and zero real callers anywhere in this workspace. This module is the first one, and
//! `candidate.rs`'s own doc says exactly which finding shape it is real for and why: one
//! blocking Phantom finding from `nomos_rules::Check_Completeness_Mirrors`, where the safe,
//! judgment-free correction is striking a doc comment's already-false claim of a mirror
//! that does not exist, never inventing one.
//!
//! # What a composition root still has to assemble
//!
//! The same three things `check.rs`'s own doc names, for the identical reasons: walking
//! the tree ([`sources::Walked`], a near-duplicate of `check::sources` rather than a
//! shared dependency on it — see that module's own doc), reading what this binary was
//! compiled as ([`Correction_Variant`]), and choosing a [`nomos_platform::ProcessLauncher`]
//! ([`nomos_platform_std::StdProcessLauncher`]) for `nomos_check_orchestration::Run`'s one
//! subprocess-running provider.
//!
//! A fourth thing is new here and has no analogue in `check.rs`: a second, local
//! [`nomos_workspace::Workspace`], seeded from the one file a correction touches and used
//! only to prove that file has not moved between staging and validating (and, with
//! `--commit`, committing) it — [`nomos_workspace::Workspace`]'s own module doc calls this
//! "one door", and this is this command's own use of it, independent of whatever workspace
//! state `nomos-check-orchestration`'s internal fact store may or may not carry.
//!
//! # Why only one candidate, ever, per run
//!
//! `candidate::Phantom_Claim` is asked for the *first* finding it recognizes, in the same
//! sorted order `Check_Completeness_Mirrors` already returns findings in. Two phantom
//! claims in the same file would conflict in `nomos_corrections::CorrectionPlan::New`
//! (two candidates touching one path); two phantom claims in two different files could in
//! principle share a plan, but batching real filesystem writes together in a first
//! increment is exactly the "any rule, any correction" generality this workspace's own
//! culture — and this item's own scope — declines to build ahead of a second real case.
//! Run the command again after a commit to correct the next one.

mod candidate;
mod exit_code;
mod parsing;
mod sources;
#[cfg(test)]
mod tests;

pub(crate) use exit_code::ExitCode;
pub use parsing::Parse;

use crate::arguments::Named_Value_From_String_Arguments;
use candidate::{Candidate_For, ClaimError, Finding_Reference, Phantom_Claim, PhantomClaim};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{ConfigurationId, EvidenceClass, Finding, ProviderId, RuleId};
use nomos_corrections::{CommittedPlan, CorrectionPlan, ValidatedPlan};
use nomos_model::{Content_Digest, Evidence, Subject_Of_Path};
use nomos_platform_std::{StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use sources::Walked;
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
    return Attempted_Correction(command, stdout, stderr).unwrap_or_else(|code| return code);
}

/// The whole correction attempt: walking source, judging it, finding the one claim to
/// correct, planning the fix, previewing it, seeding a workspace to stage it against,
/// staging and validating it, and either reporting the dry run or committing it -- nine
/// distinct decisions, each with the name of the helper that makes it, chained with `?` so
/// this function's own body reads as one line per decision rather than one `match` per
/// decision.
fn Attempted_Correction(command: &CorrectCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<ExitCode, ExitCode>
{
    let sources = Walked_Or_Reported(&command.root, stderr)?;
    let findings = Judged_Findings(&sources, &command.root, stderr)?;

    let Some(claim) = Claimed_Or_Reported(&findings, &command.root, stdout)
    else
    {
        return Ok(ExitCode::Ok);
    };

    let (plan, before, after) = Planned(&command.root, &claim, stderr)?;
    Rendered_Preview(&plan, stdout);

    let mut workspace = Seeded_Workspace(&command.root, SeedContent { path: claim.path, content: &before }, stderr);
    let validated = Staged_And_Validated(&plan, &workspace, stderr)?;

    if !command.commit
    {
        Reported_Dry_Run(&claim, stdout);
        return Ok(ExitCode::Ok);
    }

    return Ok(Committed((&command.root, &claim), (&after, validated), &mut workspace, (stdout, stderr)));
}

/// Walks `root` for source, or reports why there is none to walk and the [`ExitCode`] that
/// already decides -- [`Run`]'s own first section, factored out so a reader sees the walk
/// and its vacuity guard as one call rather than an inline match.
fn Walked_Or_Reported(root: &Path, stderr: &mut impl Write) -> Result<Vec<SourceFile>, ExitCode>
{
    return match Walked(root)
    {
        None =>
        {
            let _ = writeln!(stderr, "`{}` is not a directory", root.display());
            Err(ExitCode::Unreadable)
        }
        Some(sources) if sources.is_empty() =>
        {
            let _ = writeln!(stderr, "no `.rs` or `.go` source found under `{}`", root.display());
            Err(ExitCode::Vacuous)
        }
        Some(sources) => Ok(sources),
    };
}

/// Runs `Check_Completeness_Mirrors` over `sources` and returns its findings, or the
/// [`ExitCode`] a non-`Judged` outcome already decides.
fn Judged_Findings(sources: &[SourceFile], root: &Path, stderr: &mut impl Write) -> Result<Vec<Finding>, ExitCode>
{
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];
    let outcome = nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: Correction_Variant(),
            root,
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut None,
            store: &mut nomos_analysis::MemoryFactStore::New(),
        },
        &selected,
    );

    return match outcome
    {
        CheckOutcome::Judged { findings, .. } => Ok(findings),
        CheckOutcome::Unreadable =>
        {
            let _ = writeln!(stderr, "the tree could not be read as a workspace state");
            Err(ExitCode::Vacuous)
        }
        CheckOutcome::Contradictory(error) =>
        {
            let _ = writeln!(stderr, "this build's own capability registry is self-contradictory: {error}");
            Err(ExitCode::Vacuous)
        }
        CheckOutcome::NoSource =>
        {
            let _ = writeln!(stderr, "the walk found no source under `{}`", root.display());
            Err(ExitCode::Vacuous)
        }
        CheckOutcome::NoFacts { files } =>
        {
            let _ = writeln!(stderr, "{files} file(s) were read but no syntax fact was materialized for any of them");
            Err(ExitCode::Vacuous)
        }
    };
}

/// The first blocking phantom claim `findings` names, or `None` once a clean run has
/// already been reported.
fn Claimed_Or_Reported<'a>(findings: &'a [Finding], root: &Path, stdout: &mut impl Write) -> Option<PhantomClaim<'a>>
{
    let Some(claim) = findings.iter().find_map(Phantom_Claim)
    else
    {
        let _ = writeln!(stdout, "clean: no blocking phantom mirror claim under `{}`", root.display());
        return None;
    };

    return Some(claim);
}

/// Builds the one real candidate for `claim` and the single-candidate plan around it, or
/// the [`ExitCode`] a refusal already decides.
fn Planned(root: &Path, claim: &PhantomClaim<'_>, stderr: &mut impl Write) -> Result<(CorrectionPlan, String, String), ExitCode>
{
    let (candidate, before, after) = match Candidate_For(root, claim)
    {
        Ok(triple) => triple,
        Err(ClaimError::Unreadable(error)) =>
        {
            let _ = writeln!(stderr, "could not read `{}`: {error}", claim.path);
            return Err(ExitCode::Refused);
        }
        Err(ClaimError::Ambiguous { occurrences }) =>
        {
            let _ = writeln!(
                stderr,
                "`{}` names `{}` {occurrences} time(s) in `{}`, not exactly once; refusing to guess which line is the real declaration",
                claim.finding.subject_name, claim.claimed, claim.path
            );
            return Err(ExitCode::Refused);
        }
    };

    return match CorrectionPlan::New(vec![candidate])
    {
        Ok(plan) => Ok((plan, before, after)),
        Err(error) =>
        {
            let _ = writeln!(stderr, "{error}");
            Err(ExitCode::Refused)
        }
    };
}

/// Prints the plan's own preview, verbatim.
fn Rendered_Preview(plan: &CorrectionPlan, stdout: &mut impl Write)
{
    let _ = writeln!(stdout, "{}", String::from_utf8_lossy(plan.Preview().Rendered()));
}

/// The file a [`Workspace`] is seeded with: where it lives and what it holds, paired so a
/// caller cannot transpose which is which -- both are `&str` and the compiler cannot catch
/// a swap between them on its own.
struct SeedContent<'a>
{
    /// The path the content lives at, as a real [`Finding`] named it.
    path: &'a str,
    /// The content itself, as this run actually read it from disk.
    content: &'a str,
}

/// A fresh local [`Workspace`], holding exactly the one file this run may correct, at the
/// content it actually read from disk — the same content [`Candidate_For`]'s `Edit` declares
/// as `before`, so staging's own staleness check is checking this run's own read against
/// itself, not against a second, independent read that could disagree with it.
fn Seeded_Workspace(root: &Path, seed: SeedContent<'_>, stderr: &mut impl Write) -> Workspace
{
    let configuration = ConfigurationId::From_Digest(Content_Digest(root.display().to_string().as_bytes()));
    let mut workspace = Workspace::Empty(Correction_Variant(), configuration);

    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present(seed.path, seed.content.to_owned());
    if let Err(error) = workspace.Apply(&initial)
    {
        // A one-member, non-empty change set applied to a fresh empty workspace cannot be
        // `Vacuous` or `Conflicting`, and `claim.path` is a real location a real Finding named
        // rather than caller-typed text, so `Unnamed` does not apply either -- this branch
        // should be unreachable. Reported rather than silently discarded, but still not fatal:
        // leaving `workspace` unseeded is still honest either way, because `Stage` would then
        // see no content at `path` and refuse the plan as stale, rather than this function
        // papering over a real refusal by pretending the seed succeeded.
        let _ = writeln!(stderr, "internal: seeding the correction workspace unexpectedly refused: {error}");
    }

    return workspace;
}

/// Stages `plan` against `workspace` and validates it again immediately, or the
/// [`ExitCode`] a refusal at either step already decides.
fn Staged_And_Validated(plan: &CorrectionPlan, workspace: &Workspace, stderr: &mut impl Write) -> Result<ValidatedPlan, ExitCode>
{
    let staged = plan.Stage(workspace).map_err(|error| {
        let _ = writeln!(stderr, "{error}");
        return ExitCode::Refused;
    })?;

    return staged.Validate(workspace).map_err(|error| {
        let _ = writeln!(stderr, "{error}");
        return ExitCode::Refused;
    });
}

/// Reports what committing this claim would do, for a run that staged and validated
/// cleanly but was not asked to `--commit`.
fn Reported_Dry_Run(claim: &PhantomClaim<'_>, stdout: &mut impl Write)
{
    let _ = writeln!(
        stdout,
        "dry run: `{}` no longer claims `{}` in this preview. Pass --commit to apply it.",
        claim.path, claim.claimed
    );
}

/// Commits `validated` through the workspace's one door and writes the corrected file to
/// disk, or reports why either step refused. `target` is the tree and the claim being
/// corrected, `commit` is the corrected file content and the plan committing it, and
/// `output` is where a result is reported — three of this file's own natural pairs, kept
/// as tuples rather than named types this function is the only caller of.
fn Committed(
    target: (&Path, &PhantomClaim<'_>), commit: (&str, ValidatedPlan), workspace: &mut Workspace,
    output: (&mut impl Write, &mut impl Write),
) -> ExitCode
{
    let (root, claim) = target;
    let (after, validated) = commit;
    let (stdout, stderr) = output;

    let committed = match Committed_Through_The_Workspace(validated, workspace, claim, stderr)
    {
        Ok(committed) => committed,
        Err(code) => return code,
    };

    if let Err(code) = Written_To_Disk(root, claim, after, stderr)
    {
        return code;
    }

    Reported_Commit(claim, &committed, stdout);
    return ExitCode::Ok;
}

/// Commits `validated` through the workspace's one door, carrying `claim` as the evidence a
/// real commit records, or reports why it refused.
fn Committed_Through_The_Workspace(
    validated: ValidatedPlan, workspace: &mut Workspace, claim: &PhantomClaim<'_>, stderr: &mut impl Write,
) -> Result<CommittedPlan, ExitCode>
{
    let evidence = Evidence {
        class: EvidenceClass::Derived,
        producer: ProviderId::New("nomos-cli-correct-phantom-mirrors"),
        supporting: vec![Finding_Reference(claim)],
    };

    return validated.Commit(workspace, evidence).map_err(|error| {
        let _ = writeln!(stderr, "{error}");
        return ExitCode::Refused;
    });
}

/// Writes the corrected file to disk, once the workspace model already carries the commit,
/// or reports why the write failed.
fn Written_To_Disk(root: &Path, claim: &PhantomClaim<'_>, after: &str, stderr: &mut impl Write) -> Result<(), ExitCode>
{
    return std::fs::write(root.join(claim.path), after).map_err(|error| {
        let _ = writeln!(stderr, "committed to the workspace model but could not write `{}`: {error}", claim.path);
        return ExitCode::Refused;
    });
}

/// Reports what was committed.
fn Reported_Commit(claim: &PhantomClaim<'_>, committed: &CommittedPlan, stdout: &mut impl Write)
{
    let _ = writeln!(
        stdout,
        "committed: `{}` no longer claims `{}` ({} -> {})",
        claim.path,
        claim.claimed,
        committed.Base(),
        committed.After()
    );
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
