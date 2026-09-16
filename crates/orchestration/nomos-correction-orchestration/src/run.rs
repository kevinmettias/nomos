//! Composing an already-walked tree into a real correction run, apart from choosing a
//! platform, walking a tree, or rendering the answer.
//!
//! Moved here from `nomos-cli`'s own `correct.rs`, the same migration `P13-GATE-RUN-SEAM-
//! CRATE` made for `gate.rs`: that module's own composition — walk, judge, find the one
//! claim, plan, seed a workspace, stage, validate, and optionally commit — is this file's
//! [`Run_Correction`] now, and `nomos-cli`'s `correct.rs` is a thin renderer over it.
//!
//! # A second correction family, and one shared pipeline for both
//!
//! `P40-CORRECTIONS-SECOND-FAMILY-3` adds [`crate::trailing_whitespace`] beside
//! [`crate::phantom_mirror`] -- a second rule (`no-trailing-whitespace`), a second real
//! fix, over a second real finding shape. [`Judged`] now selects both rules at once, and
//! [`Claimed_Fix`] tries each family's own claim-recognizer against the one resulting
//! `findings` list, in a fixed priority order, and hands whichever matches to the one
//! `plan`/`preview`/seed/stage/validate/commit pipeline [`Staged`] runs exactly once.
//! Neither family's own module knows the other exists; both build the same [`ClaimedFix`]
//! shape, and everything downstream of that point — the whole of [`Staged`] and
//! [`Committed`] — is unaware which family produced it.
//!
//! # Why the pipeline is four functions rather than one
//!
//! This crate's own `function-size` budget is a 24-row body, and the pipeline above is
//! longer than that read end to end. So it is split where it already has seams:
//! [`Run_Correction`] answers the walk's own two refusals, [`Corrected`] judges and
//! recognizes a claim, [`Staged`] plans and drives the lifecycle, and [`Committed`] writes
//! the corrected file back. Each step keeps the ordering the single body had, and the
//! `Result<_, CorrectionOutcome>` [`Corrected`] and [`Staged`] return is what carries an
//! early refusal from whichever step produced it to the one caller that reports it.
//!
//! ## What this proves about batching
//!
//! [`crate::trailing_whitespace`]'s own module doc makes the real claim: one candidate
//! covering every flagged line in one file, the first time this crate has batched more
//! than one finding into one edit. Phantom-mirror could not have shown this on its own --
//! `nomos_corrections::CorrectionPlan::New` refuses two candidates touching the same
//! path, so batching only becomes visible once a family's own fix naturally spans more
//! than one location in one file, which striking a single doc-comment line never does.
//!
//! ## What this leaves undecided about ranking
//!
//! [`Claimed_Fix`]'s priority order -- phantom-mirror before trailing-whitespace -- is
//! not a ranking policy. It is this pipeline's own arbitrary tie-break for the one case
//! two families create that one family never could: a tree presenting blocking findings
//! from both at once. Nothing here scores a candidate, compares competing fixes for the
//! same file, or lets a caller ask for a specific family; a real `CorrectionChoice`/
//! `ChoiceRecord` (`nomos-corrections` already declares both, unconstructed) is what
//! `COR-011`..`013`'s own ranking machinery would need a real second candidate to choose
//! between, and this pipeline still only ever proposes one candidate per run. A third
//! family choosing between two real, competing fixes for the *same* location is what
//! would make that decidable; a second family whose own fix shape never overlaps the
//! first's is not that population yet.

use crate::{CorrectionCommand, CorrectionFamily, CorrectionOutcome};
use crate::phantom_mirror::{self, ClaimError, Finding_Reference, Phantom_Claim, PhantomClaim};
use crate::trailing_whitespace::{self, TrailingWhitespaceClaim};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{ConfigurationId, EvidenceClass, Finding, ProviderId, RuleId};
use nomos_corrections::{CommittedPlan, CorrectionCandidate, CorrectionPlan, ValidatedPlan};
use nomos_model::{Content_Digest, Evidence, EvidenceRef};
use nomos_platform::{Environment, FileSystem, ProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use std::path::Path;

/// The build variant, process launcher and filesystem [`Run_Correction`] needs but does not
/// compute -- grouped into one value the same way `nomos_gate_orchestration::
/// GateEnvironment` groups its own three, for the identical reason: neither computes a
/// build variant, chooses a platform, or is the caller that gets to decide either.
pub struct CorrectionEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    pub variant: BuildVariant,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
}

/// Judges `walked` for a blocking claim from either correction family, and if one
/// matches, plans, previews, seeds a workspace, stages and validates the fix --
/// committing it and writing the corrected file through `environment.filesystem` when
/// `command.commit` is set.
///
/// `walked` is the walk, already done and already decided by the composition root, the
/// same reason `nomos_gate_orchestration::Run_Gate` takes it rather than a root to read.
#[must_use]
pub fn Run_Correction<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(walked: Option<Vec<SourceFile>>, environment: CorrectionEnvironment<'_, Launcher, Fs, Env>, command: &CorrectionCommand) -> CorrectionOutcome
{
    let CorrectionEnvironment { variant, launcher, filesystem, environment } = environment;

    let Some(sources) = walked
    else
    {
        return CorrectionOutcome::UnreadableRoot;
    };

    return match Corrected(&sources, RunRequest { variant, launcher, filesystem, environment, command })
    {
        Ok(outcome) => outcome,
        Err(outcome) => outcome,
    };
}

/// Everything the rest of the pipeline needs once the walk is known to have carried no
/// refusal: the platform the composition root chose, and the command naming the root.
struct RunRequest<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    variant: BuildVariant,
    launcher: &'a Launcher,
    filesystem: &'a Fs,
    environment: &'a Env,
    command: &'a CorrectionCommand,
}

/// The pipeline past the walk: an empty walk is refused, the tree is judged, one family's
/// claim is recognized, and [`Staged`] carries it through the lifecycle.
///
/// `Ok` is the run's own outcome; `Err` is the [`CorrectionOutcome`] that ended it before
/// [`Staged`] was reached. Both halves are the same type because a refusal here is not a
/// distinct kind of failure, it is the answer a refusal always was.
fn Corrected<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(sources: &[SourceFile], request: RunRequest<'_, Launcher, Fs, Env>) -> Result<CorrectionOutcome, CorrectionOutcome>
{
    if sources.is_empty()
    {
        return Err(CorrectionOutcome::NoSourceFound);
    }

    let findings = Judged(sources, JudgeContext { launcher: request.launcher, filesystem: request.filesystem, environment: request.environment, variant: request.variant.clone(), root: &request.command.root })?;
    let fix = Claimed_Fix(&request.command.root, &findings, request.filesystem)?.ok_or(CorrectionOutcome::Clean)?;

    return Staged(fix, request);
}

/// What [`Judged`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- the same grouping this crate's own callers use to stay
/// within its parameter-count limit.
struct JudgeContext<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    /// The process launcher a provider's subprocess runs through.
    launcher: &'a Launcher,
    /// The filesystem a provider reads through.
    filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from.
    environment: &'a Env,
    variant: BuildVariant,
    root: &'a Path,
}

/// Runs both correction families' own rules over `sources`, or the [`CorrectionOutcome`] a
/// non-`Judged` check outcome already decides.
fn Judged<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(sources: &[SourceFile], context: JudgeContext<'_, Launcher, Fs, Env>) -> Result<Vec<Finding>, CorrectionOutcome>
{
    let selected: Vec<RuleId> = CorrectionFamily::ALL.iter().map(|family| return family.Rule()).collect();
    let outcome = nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: context.variant,
            root: context.root,
            launcher: context.launcher,
            filesystem: context.filesystem,
            environment: context.environment,
            workspace: &mut None,
            store: &mut nomos_analysis::MemoryFactStore::New(),
        },
        &selected,
    );

    return Judged_Outcome(outcome);
}

/// The findings [`CheckOutcome::Judged`] carries, or the [`CorrectionOutcome`] every other
/// check outcome already decides.
fn Judged_Outcome(outcome: CheckOutcome) -> Result<Vec<Finding>, CorrectionOutcome>
{
    return match outcome
    {
        CheckOutcome::Judged { findings, .. } => Ok(findings),
        CheckOutcome::Unreadable => Err(CorrectionOutcome::UnreadableWorkspaceState),
        CheckOutcome::Contradictory(error) => Err(CorrectionOutcome::ContradictoryRegistry(error.to_string())),
        CheckOutcome::NoSource => Err(CorrectionOutcome::NoSourceFound),
        CheckOutcome::NoFacts { files } => Err(CorrectionOutcome::NoFactsMaterialized(files)),
    };
}

/// One family's real candidate for a claim it recognized, and everything the shared
/// pipeline downstream of [`Claimed_Fix`] needs to stage, validate and commit it without
/// knowing which family built it.
struct ClaimedFix
{
    path: String,
    /// The candidate's own description, carried through to [`CorrectionOutcome`]
    /// verbatim -- each family writes the human-readable summary its own fix deserves,
    /// and nothing downstream re-decides what it means.
    summary: String,
    candidate: CorrectionCandidate,
    before: String,
    after: String,
    evidence_reference: EvidenceRef,
}

/// Tries each [`CorrectionFamily`] against `findings`, in [`CorrectionFamily::ALL`]'s own
/// declared order -- the first to recognize a claim wins. `Ok(None)` if none does, a clean
/// run. `Err(...)` if a family recognized a claim but could not build a candidate for it.
/// This crate's own module doc says what that priority order is and is not.
///
/// Matching `family` exhaustively is the mechanism the whole declaration rests on: a
/// variant added to [`CorrectionFamily`] does not compile until it is wired to a real
/// recognizer here, so a family cannot be selected by [`Judged`] and then be recognized
/// by nobody.
fn Claimed_Fix<Fs: FileSystem>(root: &Path, findings: &[Finding], filesystem: &Fs) -> Result<Option<ClaimedFix>, CorrectionOutcome>
{
    for family in CorrectionFamily::ALL
    {
        match family
        {
            CorrectionFamily::PhantomMirror =>
            {
                if let Some(claim) = findings.iter().find_map(Phantom_Claim)
                {
                    return Ok(Some(Phantom_Mirror_Fix(root, &claim, filesystem)?));
                }
            }
            CorrectionFamily::TrailingWhitespace =>
            {
                if let Some(claim) = trailing_whitespace::Trailing_Whitespace_Claim(findings)
                {
                    return Ok(Some(Trailing_Whitespace_Fix(root, &claim, filesystem)?));
                }
            }
        }
    }

    return Ok(None);
}

/// Builds the [`ClaimedFix`] phantom-mirror's own candidate makes for `claim`, or the
/// [`CorrectionOutcome`] its own refusal already decides.
fn Phantom_Mirror_Fix<Fs: FileSystem>(root: &Path, claim: &PhantomClaim<'_>, filesystem: &Fs) -> Result<ClaimedFix, CorrectionOutcome>
{
    let (candidate, before, after) = match phantom_mirror::Candidate_For(root, claim, filesystem)
    {
        Ok(triple) => triple,
        Err(ClaimError::Unreadable(error)) => return Err(CorrectionOutcome::Refused(format!("could not read `{}`: {error}", claim.path))),
        Err(ClaimError::Ambiguous { occurrences }) =>
        {
            return Err(CorrectionOutcome::Refused(format!(
                "`{}` names `{}` {occurrences} time(s) in `{}`, not exactly once; refusing to guess which line is the real declaration",
                claim.finding.subject_name, claim.claimed, claim.path
            )));
        }
    };

    return Ok(ClaimedFix {
        path: claim.path.to_owned(),
        summary: candidate.Description().to_owned(),
        candidate,
        before,
        after,
        evidence_reference: Finding_Reference(claim),
    });
}

/// Builds the [`ClaimedFix`] trailing-whitespace's own candidate makes for `claim`, or the
/// [`CorrectionOutcome`] its own refusal already decides.
fn Trailing_Whitespace_Fix<Fs: FileSystem>(root: &Path, claim: &TrailingWhitespaceClaim<'_>, filesystem: &Fs) -> Result<ClaimedFix, CorrectionOutcome>
{
    let (candidate, before, after) = trailing_whitespace::Candidate_For(root, claim, filesystem)
        .map_err(|error| return CorrectionOutcome::Refused(format!("could not read `{}`: {error}", claim.path)))?;

    return Ok(ClaimedFix {
        path: claim.path.to_owned(),
        summary: candidate.Description().to_owned(),
        candidate,
        before,
        after,
        evidence_reference: EvidenceRef {
            kind: "finding".to_owned(),
            locator: format!("{}::{}", nomos_rules::NO_TRAILING_WHITESPACE, claim.path),
        },
    });
}

/// Plans `fix`, seeds a workspace from the prior content it read, stages and validates the
/// plan against that workspace, and either reports the staged plan or commits it through
/// [`Committed`].
///
/// The `Result` is [`Corrected`]'s own: `Err` is the [`CorrectionOutcome`] one of the two
/// checked steps refused with, carried back unchanged.
fn Staged<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(fix: ClaimedFix, request: RunRequest<'_, Launcher, Fs, Env>) -> Result<CorrectionOutcome, CorrectionOutcome>
{
    let mut workspace = Seeded_Workspace(request.variant, request.command, &fix);
    let plan = CorrectionPlan::New(vec![fix.candidate]).map_err(|error| return CorrectionOutcome::Refused(error.to_string()))?;
    let preview = plan.Preview().Rendered().to_vec();
    let validated = Staged_And_Validated(&plan, &workspace)?;

    if !request.command.commit
    {
        return Ok(CorrectionOutcome::Staged { path: fix.path, summary: fix.summary, preview });
    }

    return Ok(Committed(Committing {
        root: &request.command.root,
        path: fix.path,
        summary: fix.summary,
        evidence_reference: fix.evidence_reference,
        validated,
        workspace: &mut workspace,
        after: &fix.after,
        preview,
        filesystem: request.filesystem,
    }));
}

/// The workspace `plan` is staged and validated against: a fresh empty one seeded with
/// `fix.path`'s own prior content, which is what `Stage` compares the plan's declared
/// prior content to.
///
/// A one-member, non-empty change set applied to a fresh empty workspace cannot refuse --
/// see `nomos-cli`'s own former `Seeded_Workspace` for the full reasoning this carries
/// forward unchanged. A refusal here is not reported further; `Stage` below would then see
/// no content at `fix.path` and refuse the plan as stale on its own.
fn Seeded_Workspace(variant: BuildVariant, command: &CorrectionCommand, fix: &ClaimedFix) -> Workspace
{
    let configuration = ConfigurationId::From_Digest(Content_Digest(command.root.display().to_string().as_bytes()));
    let mut workspace = Workspace::Empty(variant, configuration);
    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present(fix.path.clone(), fix.before.clone());
    let _ignored = workspace.Apply(&initial);

    return workspace;
}

/// Stages and validates `plan` against `workspace`, or the [`CorrectionOutcome`] whichever
/// step refused with. `Stage` catches a plan whose declared prior content no longer
/// matches `workspace`; `Validate` catches the workspace moving between the two.
fn Staged_And_Validated(plan: &CorrectionPlan, workspace: &Workspace) -> Result<ValidatedPlan, CorrectionOutcome>
{
    let staged = plan.Stage(workspace).map_err(|error| return CorrectionOutcome::Refused(error.to_string()))?;

    return staged.Validate(workspace).map_err(|error| return CorrectionOutcome::Refused(error.to_string()));
}

/// Everything [`Committed`] needs to commit a validated plan and write the corrected file,
/// grouped into one value so that function stays within this crate's own parameter-count
/// limit.
struct Committing<'a, Fs: FileSystem>
{
    root: &'a Path,
    path: String,
    summary: String,
    evidence_reference: EvidenceRef,
    validated: ValidatedPlan,
    workspace: &'a mut Workspace,
    after: &'a str,
    preview: Vec<u8>,
    filesystem: &'a Fs,
}

/// Commits `committing.validated` through the workspace's one door and writes the
/// corrected file through `committing.filesystem`, or reports why either step refused.
fn Committed<Fs: FileSystem>(committing: Committing<'_, Fs>) -> CorrectionOutcome
{
    let Committing { root, path, summary, evidence_reference, validated, workspace, after, preview, filesystem } = committing;
    let committed = match Committed_Plan(validated, workspace, evidence_reference)
    {
        Ok(committed) => committed,
        Err(outcome) => return outcome,
    };

    if let Err(error) = filesystem.Replace_Atomically(&root.join(&path), after)
    {
        return CorrectionOutcome::Refused(format!("committed to the workspace model but could not write `{path}`: {error}"));
    }

    return CorrectionOutcome::Committed {
        path,
        summary,
        preview,
        base: committed.Base().to_string(),
        after_snapshot: committed.After().to_string(),
    };
}

/// Commits `validated` through the workspace's one door, submitting under the derived
/// evidence `evidence_reference` supports, or reports the refusal.
fn Committed_Plan(validated: ValidatedPlan, workspace: &mut Workspace, evidence_reference: EvidenceRef) -> Result<CommittedPlan, CorrectionOutcome>
{
    let evidence = Evidence { class: EvidenceClass::Derived, producer: ProviderId::New("nomos-correction-orchestration"), supporting: vec![evidence_reference] };

    return validated.Commit(workspace, evidence).map_err(|error| return CorrectionOutcome::Refused(error.to_string()));
}

#[cfg(test)]
mod tests;
