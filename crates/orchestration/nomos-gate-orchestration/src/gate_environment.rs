//! Composing an already-walked tree into a real `nomos gate run`, apart from choosing a
//! platform, walking a tree or rendering the answer.

mod provenance;
mod reduction;

#[cfg(test)]
mod reason_recording_tests;
#[cfg(test)]
mod tests;

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{RuleId, RunId};
use nomos_platform::{Environment, FileSystem, ProcessLauncher, Timestamp};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::policy::{GatePolicyFile, Resolve_Gate_Policy};
use crate::{Evaluated_Phases, GateCommand, GateRunOutcome, GateRunProvenance, GateRunResult, Phased_Disposition};
use reduction::{DispositionPolicies, Reduction, Reduced_Findings, Scoped_Findings};

/// Judges `walked` exactly as `nomos check` would.
///
/// `walked` is the walk, already done and already decided by the composition root, the same
/// reason `nomos_check_orchestration::Run` takes `sources` rather than a root to read:
/// `None` for a root that was not a directory, `Some(sources)` otherwise -- including the
/// empty case, so the no-source-found decision stays visible to a caller rather than
/// collapsing into `Some` versus `None`. `context.variant` and `launcher` cross to
/// [`nomos_check_orchestration::Run`] unchanged; see its own documentation for why each is a
/// composition-root value this crate cannot compute for itself.
///
/// Shared by [`Run_Gate`] (over a `command.scope`-narrowed walk, and `command.rules`-selected
/// per `OD-GATE-017`) and [`crate::Explain_Gate`] (over the whole one and every rule, since
/// explain answers a question about one named finding, not a scope- or rule-narrowed
/// disposition) — factored out so the two do not duplicate this match.
///
/// `context.selected` names which rules [`nomos_check_orchestration::Run`] should compute
/// at all -- empty for every rule, the same default `RuleSelector::include` already has. A
/// caller that must see every rule's findings regardless of `command.rules` (`Explain_Gate`)
/// passes an empty slice here rather than `command.rules.include`.
pub(crate) fn Judged_Sources<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    context: JudgeContext<'_, Launcher, Fs, Env>,
) -> CheckOutcome
{
    return match walked
    {
        None => CheckOutcome::Unreadable,
        Some(sources) if sources.is_empty() => CheckOutcome::NoSource,
        Some(sources) => nomos_check_orchestration::Run(
            &sources,
            nomos_check_orchestration::RunContext {
                variant: context.variant,
                root: context.root,
                launcher: context.launcher,
                filesystem: context.filesystem,
                environment: context.environment,
                workspace: &mut None,
                store: &mut nomos_analysis::MemoryFactStore::New(),
            },
            context.selected,
        ),
    };
}

/// The build variant, process launcher and filesystem [`Run_Gate`] and [`crate::Explain_Gate`]
/// both need but neither computes -- grouped into one value so each stays within this crate's
/// own parameter-count limit. `command` and `walked`/`query`/`run` stay separate parameters:
/// this groups only the three values every gate entry point shares.
pub struct GateEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    pub variant: BuildVariant,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
    /// The moment this run is judged against.
    ///
    /// Supplied by whoever composed the run rather than read from a clock inside policy
    /// logic, so a replay of a past run answers as that run did instead of as today would.
    /// It rides here for the reason `variant` does: the composing function is already at this
    /// workspace's own parameter-count limit, and an execution fact belongs with the other
    /// execution facts rather than in the caller-authored `GateCommand`, which is policy.
    pub now: Timestamp,
}

/// What [`Judged_Sources`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- grouped into one value so [`Judged_Sources`] stays within this
/// crate's own parameter-count limit.
pub(crate) struct JudgeContext<'a, Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>
{
    /// The process launcher a provider's subprocess runs through.
    pub(crate) launcher: &'a Launcher,
    /// The filesystem a provider reads through.
    pub(crate) filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from.
    pub(crate) environment: &'a Env,
    /// Which [`BuildVariant`] to judge as.
    pub(crate) variant: BuildVariant,
    /// The tree this judgment is over.
    pub(crate) root: &'a Path,
    /// Which rules [`nomos_check_orchestration::Run`] should compute at all -- empty for
    /// every rule.
    pub(crate) selected: &'a [RuleId],
}

/// Judges `walked` exactly as `nomos check` would, and reduces the result to a
/// [`GateRunResult`].
///
/// Each step is one named function, because each answers a question the others do not: which
/// policy the command is judged under, what judged it, what was judged, which findings still
/// block, and what disposition those leave. Read in that order this is the whole verb.
///
/// `run` identifies this execution and is not computed here -- `OD-WORKFLOW-001`'s amendment
/// decided a `RunId` identifies one execution, not one configuration, so this function must
/// not derive it from `command` or `variant` the way everything else it composes is derived.
/// The composition root supplies one, typically [`crate::Fresh_Run_Id`] over a real clock
/// reading.
#[must_use]
pub fn Run_Gate<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    environment: GateEnvironment<'_, Launcher, Fs, Env>,
    command: &GateCommand,
    run: RunId,
) -> GateRunResult
{
    let GateEnvironment { variant, launcher, filesystem, environment, now } = environment;
    let declared = Resolve_Gate_Policy(&command.root, filesystem);
    let effective = Effective_Policy(declared.as_ref().ok().and_then(Option::as_ref), command);
    let provenance = GateRunProvenance {
        source: provenance::Source_Digest(walked.as_deref()),
        policy: provenance::Policy_Digest(&effective),
        selection: provenance::Selection_Digest(command),
        instrument: provenance::Instrument_Digest(&variant),
        at: now,
    };
    let context = JudgeContext { launcher, filesystem, environment, variant, root: &command.root, selected: &command.rules.include };
    let outcome = Scoped_Judgment(walked, command, context);
    let policies = DispositionPolicies_Of(&effective, now);
    let reduced = Reduced_Findings(&outcome, &command.rules, policies, effective.coverage);
    let disposition = Phased_Outcome(command, &reduced);
    let unusable_policy = declared.as_ref().err().map(|error| return error.As_No_Verdict());

    return GateRunResult {
        root: command.root.clone(),
        run,
        check_outcome: outcome,
        findings: reduced.findings,
        unmatched_policy: reduced.unmatched_policy,
        // The policy failure wins when both could apply. It cannot: an unreadable file falls
        // back to a default policy whose coverage is Unset, so Reduced_With_Coverage never
        // downgrades under one. Written as a preference anyway rather than as an assumption,
        // because the fallback is in a different function than this line.
        disposition: if unusable_policy.is_some() { GateRunOutcome::Indeterminate } else { disposition },
        no_verdict: unusable_policy.or(reduced.no_verdict),
        provenance: Some(provenance),
    };
}

/// The policy `command` is judged under.
///
/// A file that resolved to a policy is resolved over the command's own; no file at all, or one
/// that could not be read, leaves the command's policies standing alone -- which for every
/// caller that states none is today's behavior exactly. The two failures are one case here and
/// only here: [`Run_Gate`] reads the cause off the `Err` arm itself, which is what reaches a
/// caller as [`crate::NoVerdict::MalformedPolicy`].
fn Effective_Policy(from_file: Option<&GatePolicyFile>, command: &GateCommand) -> GatePolicyFile
{
    return match from_file
    {
        Some(file) => file.Resolved_Over(command),
        None => GatePolicyFile::default().Resolved_Over(command),
    };
}

/// `walked` judged, then narrowed to what `command.scope` admits.
///
/// `OD-GATE-025`: the scope never reaches the judging. Every walked file is judged, and the
/// scope narrows the findings afterwards -- a rule answering a cross-file question must see
/// the whole world or it answers a different question and labels it the same.
///
/// A scope admitting no walked source at all is still `NoSource`, which is what it was before
/// the scope moved. The judging happened and is simply discarded: what a caller is told is that
/// nothing it asked about was there, and a mistyped `--include` must not read as a repository
/// with nothing to say.
fn Scoped_Judgment<Launcher: ProcessLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    command: &GateCommand,
    context: JudgeContext<'_, Launcher, Fs, Env>,
) -> CheckOutcome
{
    let admits_a_source = walked
        .as_ref()
        .is_none_or(|sources| return sources.iter().any(|source| return command.scope.Is_In_Scope(&source.path)));
    let judged = Judged_Sources(walked, context);

    if !admits_a_source
    {
        return CheckOutcome::NoSource;
    }

    return Scoped_Findings(judged, &command.scope);
}

/// The three per-finding overrides `policy` resolved to, read against `now`.
fn DispositionPolicies_Of(policy: &GatePolicyFile, now: Timestamp) -> DispositionPolicies<'_>
{
    return DispositionPolicies { adoption: &policy.adoption, suppressions: &policy.suppressions, baseline: &policy.baseline, now };
}

/// `reduced`'s disposition once `command.phases` has been evaluated over it.
///
/// A phase approval is the one thing that can turn a blocking finding into a passing run, so
/// it is read here rather than by [`Reduced_Findings`]: that function decides what blocks, and
/// this one decides what a repository has agreed to live with.
fn Phased_Outcome(command: &GateCommand, reduced: &Reduction) -> GateRunOutcome
{
    let phase_outcomes = Evaluated_Phases(&command.phases, &reduced.findings.blocking_findings, &command.approvals);

    return Phased_Disposition(reduced.disposition, &command.phases, &phase_outcomes, &reduced.findings.blocking_findings);
}
