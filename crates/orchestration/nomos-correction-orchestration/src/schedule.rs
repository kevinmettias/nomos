//! Running every correction a set of findings claims, in the waves the substrate computes
//! -- the consumer `nomos-corrections`' compatibility judgment and wave partition did not
//! have.
//!
//! # What this is, beside [`crate::run`]
//!
//! [`crate::Run_Correction`] walks a tree, judges it, and takes the *first* family that
//! recognizes a claim: one candidate, one plan, one fix per run, and a repository with
//! twenty correctable findings across twenty files fixes them one run at a time without
//! ever learning that the twenty were independent. That path is untouched and still the
//! one both hosts call.
//!
//! [`Run_Correction_Waves`] starts from findings somebody else already judged and takes
//! *every* claim every family recognizes. It hands the resulting plans to
//! `nomos_corrections::WavePartition`, which groups them so that no wave holds a
//! conflicting pair and every conflicting pair is separated by wave order, and then stages
//! and validates them wave by wave. Independence is `nomos_corrections::Compatibility`'s
//! judgment over the read and write sets the plans' candidates declare -- never an
//! assumption, and never an inference from the edits themselves.
//!
//! # One plan per path
//!
//! Two claims on one path become one plan, the first family's. `CorrectionPlan::New`
//! already refuses two candidates touching one path inside a plan; across plans the
//! substrate cannot refuse it, because two plans on one path are a legitimate thing to
//! *schedule* -- the partition puts them in separate waves and the earlier one runs first.
//! What makes it wrong here is narrower than that: both candidates were built by reading
//! the same pre-correction file, so the second one's declared prior content is stale the
//! instant the first commits, and staging it would manufacture a refusal rather than
//! discover one. A second correction for a path is a second run's work, over a file that
//! has been corrected once.
//!
//! What that leaves, honestly stated: with the two families this crate ships, every
//! scheduled plan writes a different file, so the partition yields one wave and the
//! interesting half of it -- a conflicting pair separated into two waves -- is exercised
//! by this module's own tests over constructed plans rather than by a real tree. It stops
//! being hypothetical the moment a family declares a read set (`With_Read`) naming a file
//! another family writes, which is the shape `COR-EXEC-003` has in mind and the shape
//! neither of today's two mechanical fixes has.
//!
//! # Convergence
//!
//! `COR-005`'s second half -- "rerun affected rules, compare state signatures" -- is built
//! here rather than declared out of scope, but it reruns through the caller's own judgment
//! rather than running a check pipeline itself. The affected scope is fixed for the whole
//! run (every path any scheduled plan writes), the caller's `rejudge` is asked for that
//! same scope before the first wave and again after each committed wave, and the two
//! observations are compared both ways: what the wave cleared, and what it introduced. A
//! correction that reintroduces a finding is named in [`crate::WaveReport::Introduced`].
//!
//! Why the caller's judgment and not this crate's own: `Run_Correction`'s `Judged_Findings`
//! needs a launcher, a filesystem and an environment to run a provider, and this entry
//! point is handed findings rather than a tree precisely so that a caller that has already
//! judged something -- a host, an agent, a test -- can schedule over it. Taking the whole
//! check environment as well would make the two entry points the same function with a
//! flag, and a rerun run through a *different* judgment from the one that produced the
//! input findings would compare two things that were never comparable.
//!
//! Oscillation is `nomos_corrections::ConvergenceTrail`'s answer, and it is a property of
//! what was observed: the run stops when the scope is in a state it has already been in,
//! however few rounds that took, and never stops a run that keeps reaching new states,
//! however many rounds that takes. A dry run observes nothing at all, because it writes
//! nothing and so cannot move the scope -- an observation that could only ever repeat
//! would report an oscillation that is an artifact of not having committed anything.

use crate::phantom_mirror::{self, ClaimError, Finding_Reference, Phantom_Claim};
use crate::trailing_whitespace;
use crate::{CorrectionFamily, CorrectionSchedule, PlanOutcome, ScheduleHalt, WaveReport, WaveScheduling};
use nomos_contracts::{ConfigurationId, EvidenceClass, Finding, ProviderId};
use nomos_corrections::{ConvergenceTrail, CorrectionCandidate, CorrectionPlan, StateSignature, ValidatedPlan, WavePartition};
use nomos_model::{Content_Digest, Evidence, EvidenceRef};
use nomos_platform::FileSystem;
use nomos_workspace::{ChangeSource, Workspace, WorkspaceChangeSet};
use std::collections::BTreeSet;
use std::path::Path;

/// Who the evidence a scheduled commit carries is attributed to. Distinct from
/// [`crate::run`]'s own producer, because a wave-scheduled commit and a single-family
/// commit are two different things a reader of a `CommittedPlan` may want to tell apart.
const SCHEDULER: &str = "nomos-correction-orchestration::waves";

/// The `kind` an `EvidenceRef` pointing at a finding carries.
const FINDING_KIND: &str = "finding";

/// Schedules every correction `findings` claims into the waves the substrate computes,
/// then stages, validates and -- when `scheduling.commit` is set -- commits them wave by
/// wave, rerunning the affected scope through `rejudge` after each committed wave.
///
/// `rejudge` is asked for the findings that now stand over the paths it is handed. It is
/// called once before the first wave and once after each committed wave, and not at all
/// when the run was not asked to commit.
#[must_use]
pub fn Run_Correction_Waves<Fs: FileSystem, Rejudge: FnMut(&[String]) -> Vec<Finding>>(findings: &[Finding], scheduling: WaveScheduling<'_, Fs>, rejudge: Rejudge) -> CorrectionSchedule
{
    let built = Scheduled_Fixes(&Claiming {
        root: scheduling.root,
        findings,
        filesystem: scheduling.filesystem,
    });

    return Scheduled_Run(built, scheduling, rejudge);
}

/// Partitions `built`'s plans and runs the waves, or reports the partition's own refusal
/// and runs nothing.
///
/// Kept apart from [`Run_Correction_Waves`] because the refusal path is reachable only
/// from here: both shipped families declare their footprint at the artifact tier, where a
/// declared set is resolved by definition, so no real claim can produce a plan the
/// partition declines to place. A refusal nothing can reach is a refusal nobody knows
/// still works.
fn Scheduled_Run<Fs: FileSystem, Rejudge: FnMut(&[String]) -> Vec<Finding>>(built: ScheduledFixes, scheduling: WaveScheduling<'_, Fs>, rejudge: Rejudge) -> CorrectionSchedule
{
    let plans: Vec<CorrectionPlan> = built.fixes.iter().map(|fix| return fix.plan.clone()).collect();
    let partition = match WavePartition::Of(&plans)
    {
        Ok(partition) => partition,
        Err(refusal) => return CorrectionSchedule::Refusing(built.unbuilt, refusal),
    };

    let run = WaveRun {
        scope: Affected_Scope(&built.fixes),
        workspace: Seeded_Workspace(&scheduling, &built.fixes),
        schedule: CorrectionSchedule::Of(built.unbuilt, partition.Waves().len()),
        fixes: built.fixes,
        scheduling,
        rejudge,
        trail: ConvergenceTrail::New(),
        observed: Vec::new(),
    };

    return run.Run(&partition);
}

/// One plan the scheduler will run, and everything committing it needs that the plan
/// itself does not carry: the prior content its workspace is seeded from, the corrected
/// text written on commit, and what the fix is evidence of.
struct ScheduledFix
{
    plan: CorrectionPlan,
    path: String,
    summary: String,
    before: String,
    after: String,
    evidence_reference: EvidenceRef,
}

impl ScheduledFix
{
    fn Refused(&self, reason: String) -> PlanOutcome
    {
        return PlanOutcome::Refused {
            path: self.path.clone(),
            reason,
        };
    }

    fn Staged(&self) -> PlanOutcome
    {
        return PlanOutcome::Staged {
            path: self.path.clone(),
            summary: self.summary.clone(),
        };
    }
}

/// Every plan a set of findings claimed, and every claim that was recognized but could
/// not be built into one.
struct ScheduledFixes
{
    fixes: Vec<ScheduledFix>,
    unbuilt: Vec<String>,
}

/// The pieces a family's own candidate builder hands back, before they are a plan.
struct Built
{
    path: String,
    candidate: CorrectionCandidate,
    before: String,
    after: String,
    evidence_reference: EvidenceRef,
}

impl ScheduledFixes
{
    /// Whether an earlier claim already took `path`; this module's own doc says why a
    /// second plan on one path is not scheduled.
    fn Holds(&self, path: &str) -> bool
    {
        return self.fixes.iter().any(|fix| return fix.path == path);
    }

    /// Turns `built` into a one-candidate plan, or records why it could not be one. A
    /// single candidate is neither empty nor in conflict with a sibling, so
    /// `CorrectionPlan::New` is not expected to refuse -- recording its refusal rather
    /// than presuming it away is what keeps that expectation checkable.
    fn Absorb(&mut self, built: Built)
    {
        let summary = built.candidate.Description().to_owned();

        match CorrectionPlan::New(vec![built.candidate])
        {
            Ok(plan) => self.fixes.push(ScheduledFix {
                plan,
                path: built.path,
                summary,
                before: built.before,
                after: built.after,
                evidence_reference: built.evidence_reference,
            }),
            Err(error) => self.unbuilt.push(format!("`{}`: {error}", built.path)),
        }
    }
}

/// What a family is asked to recognize claims against: the tree they were judged over and
/// the filesystem each candidate reads through.
struct Claiming<'a, Fs: FileSystem>
{
    root: &'a Path,
    findings: &'a [Finding],
    filesystem: &'a Fs,
}

/// Every plan every family claims, in [`CorrectionFamily::ALL`]'s own order.
///
/// Matching `family` exhaustively for the same reason [`crate::run`]'s own `Claimed_Fix`
/// does: a variant added to [`CorrectionFamily`] does not compile until it is wired to a
/// real recognizer here, so a family cannot be declared and then be scheduled by nobody.
fn Scheduled_Fixes<Fs: FileSystem>(claiming: &Claiming<'_, Fs>) -> ScheduledFixes
{
    let mut built = ScheduledFixes {
        fixes: Vec::new(),
        unbuilt: Vec::new(),
    };

    for family in CorrectionFamily::ALL
    {
        match family
        {
            CorrectionFamily::PhantomMirror => Absorb_Phantom_Claims(&mut built, claiming),
            CorrectionFamily::TrailingWhitespace => Absorb_Whitespace_Claims(&mut built, claiming),
        }
    }

    return built;
}

/// Every blocking phantom claim in the findings, in the order they were reported.
fn Absorb_Phantom_Claims<Fs: FileSystem>(built: &mut ScheduledFixes, claiming: &Claiming<'_, Fs>)
{
    for claim in claiming.findings.iter().filter_map(Phantom_Claim)
    {
        if built.Holds(claim.path)
        {
            continue;
        }

        match phantom_mirror::Candidate_For(claiming.root, &claim, claiming.filesystem)
        {
            Ok((candidate, before, after)) => built.Absorb(Built {
                path: claim.path.to_owned(),
                candidate,
                before,
                after,
                evidence_reference: Finding_Reference(&claim),
            }),
            Err(error) => built.unbuilt.push(format!("`{}`: {}", claim.path, Described(&error))),
        }
    }
}

/// Every file any blocking trailing-whitespace finding names, ascending by path.
fn Absorb_Whitespace_Claims<Fs: FileSystem>(built: &mut ScheduledFixes, claiming: &Claiming<'_, Fs>)
{
    for claim in trailing_whitespace::Every_Trailing_Whitespace_Claim(claiming.findings)
    {
        if built.Holds(claim.path)
        {
            continue;
        }

        match trailing_whitespace::Candidate_For(claiming.root, &claim, claiming.filesystem)
        {
            Ok((candidate, before, after)) => built.Absorb(Built {
                path: claim.path.to_owned(),
                candidate,
                before,
                after,
                evidence_reference: Whitespace_Reference(claim.path),
            }),
            Err(error) => built.unbuilt.push(format!("`{}`: could not be read: {error}", claim.path)),
        }
    }
}

/// What a phantom claim's own refusal says, in the words [`crate::run`] already gives it.
fn Described(error: &ClaimError) -> String
{
    return match error
    {
        ClaimError::Unreadable(error) => format!("could not be read: {error}"),
        ClaimError::Ambiguous { occurrences } => {
            format!("names its claimed mirror {occurrences} time(s), not exactly once; refusing to guess which line is the real declaration")
        }
    };
}

fn Whitespace_Reference(path: &str) -> EvidenceRef
{
    return EvidenceRef {
        kind: FINDING_KIND.to_owned(),
        locator: format!("{}::{path}", nomos_rules::NO_TRAILING_WHITESPACE),
    };
}

/// One run of every wave, over one workspace, with one convergence trail.
struct WaveRun<'a, Fs: FileSystem, Rejudge>
{
    scheduling: WaveScheduling<'a, Fs>,
    rejudge: Rejudge,
    fixes: Vec<ScheduledFix>,
    workspace: Workspace,
    trail: ConvergenceTrail,
    /// Every path any scheduled plan writes -- what a rerun is asked to judge, fixed for
    /// the whole run so every observation is comparable with every other.
    scope: Vec<String>,
    /// What the last rerun reported, which the next one is compared against.
    observed: Vec<String>,
    schedule: CorrectionSchedule,
}

impl<Fs: FileSystem, Rejudge: FnMut(&[String]) -> Vec<Finding>> WaveRun<'_, Fs, Rejudge>
{
    /// Runs every wave in the partition's own order, stopping at the first that refuses
    /// or that leaves the affected scope in a state this run has already been in.
    fn Run(mut self, partition: &WavePartition) -> CorrectionSchedule
    {
        self.Observe_Baseline();

        for (index, wave) in partition.Waves().iter().enumerate()
        {
            if !self.Ran_Wave(wave, index)
            {
                break;
            }
        }

        return self.schedule;
    }

    /// Runs one wave and reports whether a later wave may be staged at all.
    fn Ran_Wave(&mut self, wave: &[usize], index: usize) -> bool
    {
        let mut report = WaveReport::Of(wave.to_vec());

        if let Some((position, reason)) = self.Staged_Wave(wave, &mut report)
        {
            self.schedule.halt = Some(ScheduleHalt::Refused {
                wave: index,
                position,
                reason,
            });
            self.schedule.waves.push(report);

            return false;
        }

        let repeated = self.Compared(&mut report);
        self.schedule.waves.push(report);

        return self.Continues(repeated, index);
    }

    /// Records a repeated state as this run's halt, and reports whether it may continue.
    fn Continues(&mut self, repeated: Option<usize>, index: usize) -> bool
    {
        let Some(round) = repeated
        else
        {
            return true;
        };

        self.schedule.halt = Some(ScheduleHalt::Repeated {
            wave: index,
            round,
        });

        return false;
    }

    /// Runs each plan in `wave` in turn against this run's one workspace, recording what
    /// each did, and reports the input position and refusal of the first that refused.
    ///
    /// The plans after a refusal are not attempted either. A refusal is evidence about
    /// the workspace the whole wave was measured against, not about the one plan that hit
    /// it first, and carrying on inside a wave whose base is in doubt is the blind
    /// continuation `COR-006` forbids one level up.
    fn Staged_Wave(&mut self, wave: &[usize], report: &mut WaveReport) -> Option<(usize, String)>
    {
        for position in wave
        {
            let Some(fix) = self.fixes.get(*position)
            else
            {
                continue;
            };

            let outcome = Ran_Plan(fix, &mut self.workspace, &self.scheduling);
            let refusal = Refusal_Of(&outcome);
            report.outcomes.push(outcome);

            if let Some(reason) = refusal
            {
                return Some((*position, reason));
            }
        }

        return None;
    }

    /// The state the affected scope is in before any wave runs, taken through the same
    /// rerun every later observation is taken through so the two are comparable.
    fn Observe_Baseline(&mut self)
    {
        if !self.scheduling.commit
        {
            return;
        }

        self.observed = self.Rejudged();
        let _first = self.trail.Observe(StateSignature::Of(&self.observed));
    }

    /// Reruns the affected scope, records what this wave cleared and introduced, and
    /// reports the earlier round that already held the state it is now in.
    fn Compared(&mut self, report: &mut WaveReport) -> Option<usize>
    {
        if !self.scheduling.commit
        {
            return None;
        }

        let observed = self.Rejudged();
        report.cleared = Absent_From(&self.observed, &observed);
        report.introduced = Absent_From(&observed, &self.observed);
        self.observed = observed;

        return self.trail.Observe(StateSignature::Of(&self.observed));
    }

    /// What the caller's own judgment reports over the affected scope right now.
    fn Rejudged(&mut self) -> Vec<String>
    {
        let scope = self.scope.clone();
        self.schedule.rounds = self.schedule.rounds.saturating_add(1);

        return Observations_Of(&(self.rejudge)(&scope));
    }
}

/// Stages and validates one fix against the run's workspace, committing it and writing
/// the corrected file when this run commits.
fn Ran_Plan<Fs: FileSystem>(fix: &ScheduledFix, workspace: &mut Workspace, scheduling: &WaveScheduling<'_, Fs>) -> PlanOutcome
{
    let staged = match fix.plan.Stage(workspace)
    {
        Ok(staged) => staged,
        Err(error) => return fix.Refused(error.to_string()),
    };
    let validated = match staged.Validate(workspace)
    {
        Ok(validated) => validated,
        Err(error) => return fix.Refused(error.to_string()),
    };

    if !scheduling.commit
    {
        return fix.Staged();
    }

    return Committed_Fix(Committing {
        fix,
        workspace,
        scheduling,
        validated,
    });
}

/// Everything committing one validated plan and writing its corrected file needs.
struct Committing<'a, Fs: FileSystem>
{
    fix: &'a ScheduledFix,
    workspace: &'a mut Workspace,
    scheduling: &'a WaveScheduling<'a, Fs>,
    validated: ValidatedPlan,
}

/// Commits one validated plan through the workspace's one door and writes the corrected
/// file, or reports whichever step refused.
fn Committed_Fix<Fs: FileSystem>(committing: Committing<'_, Fs>) -> PlanOutcome
{
    let Committing { fix, workspace, scheduling, validated } = committing;
    let evidence = Evidence {
        class: EvidenceClass::Derived,
        producer: ProviderId::New(SCHEDULER),
        supporting: vec![fix.evidence_reference.clone()],
    };

    let committed = match validated.Commit(workspace, evidence)
    {
        Ok(committed) => committed,
        Err(error) => return fix.Refused(error.to_string()),
    };

    if let Err(error) = scheduling.filesystem.Replace_Atomically(&scheduling.root.join(&fix.path), &fix.after)
    {
        return fix.Refused(format!("committed to the workspace model but could not write `{}`: {error}", fix.path));
    }

    return PlanOutcome::Committed {
        path: fix.path.clone(),
        summary: fix.summary.clone(),
        base: committed.Base().to_string(),
        after_snapshot: committed.After().to_string(),
    };
}

/// The refusal `outcome` carries, or `None` when it is not one.
fn Refusal_Of(outcome: &PlanOutcome) -> Option<String>
{
    return match outcome
    {
        PlanOutcome::Refused { reason, .. } => Some(reason.clone()),
        PlanOutcome::Staged { .. } | PlanOutcome::Committed { .. } => None,
    };
}

/// One observation per finding: the rule that raised it, what it named, and where. Two
/// runs report the same string for the same finding and a different one for a different
/// finding, which is all a state comparison needs of it.
fn Observations_Of(findings: &[Finding]) -> Vec<String>
{
    return findings
        .iter()
        .map(|finding| return format!("{}|{}|{}", finding.rule.As_Str(), finding.subject_name, finding.locations.join(",")))
        .collect();
}

/// The entries of `items` that `other` does not hold.
fn Absent_From(items: &[String], other: &[String]) -> Vec<String>
{
    return items.iter().filter(|item| return !other.contains(item)).cloned().collect();
}

/// Every path any scheduled plan writes, sorted and named once.
fn Affected_Scope(fixes: &[ScheduledFix]) -> Vec<String>
{
    let mut scope: Vec<String> = fixes.iter().map(|fix| return fix.path.clone()).collect();
    scope.sort_unstable();
    scope.dedup();

    return scope;
}

/// A fresh workspace holding each path's own prior content -- the one state every wave is
/// staged, validated and committed against.
///
/// Each path is seeded once, from the first fix that names it. `Workspace::Apply` refuses
/// a whole change set that names one path twice, and a refusal here would leave the
/// workspace empty and every plan in it stale for a reason that is about the seeding
/// rather than about any plan. The first fix is the right one because a later plan on the
/// same path declares the content an earlier one *leaves*, which is not what the file
/// holds before any wave has run.
///
/// A change set of distinct `Present` entries applied to a fresh empty workspace cannot
/// refuse; if it somehow did, `Stage` below would find no content at a fix's path and
/// refuse that plan as stale on its own, which is the report a caller should get anyway.
fn Seeded_Workspace<Fs: FileSystem>(scheduling: &WaveScheduling<'_, Fs>, fixes: &[ScheduledFix]) -> Workspace
{
    let configuration = ConfigurationId::From_Digest(Content_Digest(scheduling.root.display().to_string().as_bytes()));
    let mut workspace = Workspace::Empty(scheduling.variant.clone(), configuration);
    let mut initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
    let mut seeded: BTreeSet<&str> = BTreeSet::new();

    for fix in fixes
    {
        if seeded.insert(fix.path.as_str())
        {
            initial = initial.Present(fix.path.clone(), fix.before.clone());
        }
    }

    let _ignored = workspace.Apply(&initial);

    return workspace;
}

#[cfg(test)]
mod tests;
