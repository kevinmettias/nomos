//! `nomos workflow` — a thin renderer over `nomos-workflow-orchestration`'s
//! [`nomos_workflow_orchestration::Run`], the same "walk, choose a platform, render" shape
//! `gate.rs` and `correct.rs` already are over their own orchestration seams.
//!
//! # What this increment runs, and what it does not
//!
//! `nomos_workflow_orchestration::Run` already takes a whole `&[WorkflowStepPlan]` -- real
//! multi-step sequencing is that crate's own job, not this one's. What is missing is a way
//! to *reach* it from a shell at all (`P40-WORKFLOW-CLI-VERB`'s own why: "the ordinary
//! evidence that would tell us what workflows people actually compose is not being
//! collected"), and this command is exactly that reach, kept to the single-step shape a
//! `nomos_contracts::WorkflowStep` declared over argv can honestly build today: one body,
//! chosen by `--check`, `--executor` or `--model-backend`, wrapped in a fixed, always-
//! coherent declaration. A caller composing more than one step links
//! `nomos-workflow-orchestration` directly -- the same "no invented shape ahead of a real
//! body" restraint `gate.rs`'s own doc names for `compare`, applied here to a multi-step
//! plan format nothing has asked this binary to parse yet.
//!
//! `--check`'s own `CheckOutcome::Judged` renders `Ok` whatever its findings are: this verb
//! reports whether the step *ran*, not whether it passed the gate `nomos gate run` already
//! answers that question for. A caller wanting a pass/fail reading of a check step's
//! findings runs `nomos gate run` instead; folding that judgment in here would make this
//! verb a second place a check's disposition is decided; `nomos-check-orchestration::
//! CheckOutcome` and `nomos-gate-orchestration`'s own disposition already are that once.

use crate::arguments::{Named_Value_From_String_Arguments, Named_Values_From_String_Arguments, Name, Required_Value, Usage};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{
    Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, RuleId, SchemaId,
    Timeout, TraceEquivalence, WorkflowStep,
};
use nomos_gate_orchestration::{Fresh_Run_Id, GateCommand, RuleSelector};
use nomos_ledger::Territory;
use nomos_model_package::EffortLevel;
use nomos_platform::Clock;
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
use nomos_workflow_orchestration::{Body, CheckBody, CorrectionBody, GateBody, Platform, StepOutcome, WorkflowOutcome, WorkflowStepPlan};
use nomos_workspace::BuildVariant;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What `nomos workflow run` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowCommand
{
    body: Body,
}

/// Parses `nomos workflow` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Command_From_String_Arguments(arguments: &[String]) -> Result<WorkflowCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "run" => Ok(WorkflowCommand { body: Body_From_String_Arguments(rest)? }),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// `--check`'s, `--correct`'s, `--gate`'s, `--executor`'s or `--model-backend`'s own body
/// -- exactly one of the five, the same "a call reaches exactly one" discipline
/// `agent.rs`'s own `Backend_From_String_Arguments` already holds between the latter two,
/// extended to a third, fourth and fifth family that share no trait with any of the rest.
///
/// # Errors
///
/// Returns a message when none or more than one of the five is given, or when a value
/// naming a required flag is missing.
fn Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let check = arguments.iter().any(|argument| return argument == "--check");
    let correct = arguments.iter().any(|argument| return argument == "--correct");
    let gate = arguments.iter().any(|argument| return argument == "--gate");
    let executor = Named_Value_From_String_Arguments(arguments, "--executor");
    let model_backend = Named_Value_From_String_Arguments(arguments, "--model-backend");

    let named = usize::from(check)
        .saturating_add(usize::from(correct))
        .saturating_add(usize::from(gate))
        .saturating_add(usize::from(executor.is_some()))
        .saturating_add(usize::from(model_backend.is_some()));
    if named > 1
    {
        return Err(format!(
            "--check, --correct, --gate, --executor and --model-backend each name a different body; a step dispatches through exactly one, so pass at most one of them.\n\n{}",
            Usage_Text()
        ));
    }

    if check
    {
        return Check_Body_From_String_Arguments(arguments);
    }
    if correct
    {
        return Correction_Body_From_String_Arguments(arguments);
    }
    if gate
    {
        return Gate_Body_From_String_Arguments(arguments);
    }
    if let Some(text) = executor
    {
        return match text.as_str()
        {
            "claude-code" => Ok(Body::ClaudeCode(Task_From_String_Arguments(arguments)?)),
            other => Err(format!("--executor {other:?} is not one of claude-code.\n\n{}", Usage_Text())),
        };
    }
    if let Some(text) = model_backend
    {
        return match text.as_str()
        {
            "ollama" => Ok(Body::Ollama(Task_From_String_Arguments(arguments)?)),
            other => Err(format!("--model-backend {other:?} is not one of ollama.\n\n{}", Usage_Text())),
        };
    }

    return Err(format!("one of --check, --correct, --gate, --executor or --model-backend is required.\n\n{}", Usage_Text()));
}

/// `--root`'s (default `.`) and every `--rule`'s value, as a [`CheckBody`] -- the walk
/// itself deferred to [`Run`], the same division `gate.rs` and `correct.rs` already keep
/// between parsing a command and walking the tree it names.
fn Check_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let selected: Vec<RuleId> = Named_Values_From_String_Arguments(arguments, "--rule").iter().map(RuleId::New).collect();

    return Ok(Body::Check(CheckBody::New(root, Vec::new(), selected)));
}

/// `--root`'s (default `.`) and `--commit`'s presence, as a [`CorrectionBody`] -- the walk
/// itself deferred to [`Run`], the identical division [`Check_Body_From_String_Arguments`]
/// already keeps.
fn Correction_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let commit = arguments.iter().any(|argument| return argument == "--commit");

    return Ok(Body::Correction(CorrectionBody::New(root, Vec::new(), commit)));
}

/// `--root`'s (default `.`) and every `--rule`'s value, as a [`GateBody`] -- the identical
/// two flags [`Check_Body_From_String_Arguments`] already reads, since `Run_Gate` narrows
/// by the same `root` and rule selection `nomos_check_orchestration::Run` does.
fn Gate_Body_From_String_Arguments(arguments: &[String]) -> Result<Body, String>
{
    let root = Named_Value_From_String_Arguments(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let include: Vec<RuleId> = Named_Values_From_String_Arguments(arguments, "--rule").iter().map(RuleId::New).collect();
    let command = GateCommand { root, rules: RuleSelector { include }, ..GateCommand::default() };

    return Ok(Body::Gate(GateBody::New(Vec::new(), command)));
}

/// `--goal`'s value, as a bare [`TaskEnvelope`] -- every other field empty or its own
/// default, the identical shape `agent.rs`'s own `execute` builds for a person's direct
/// question with no scope, no prohibited changes and no tool allow-list of its own.
fn Task_From_String_Arguments(arguments: &[String]) -> Result<TaskEnvelope, String>
{
    let value = Named_Value_From_String_Arguments(arguments, "--goal");
    let goal = Required_Value(value.as_ref(), Name("--goal"), Usage(&Usage_Text()))?;

    return Ok(TaskEnvelope {
        goal,
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.workflow.step.v1"),
        effort: EffortLevel::BackendDefault,
    });
}

/// Runs `command`'s one step and renders what came back.
pub fn Run(command: &WorkflowCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let body = match &command.body
    {
        Body::Check(check) => match Walked(check)
        {
            Some(walked) if walked.sources.is_empty() =>
            {
                // The composition root's own decision, made before `nomos_check_
                // orchestration::Run` is ever called: that crate's own doc names an empty
                // `sources` as never its own case to classify, and `check.rs`'s identical
                // guard is what this mirrors.
                return Rendered_Check(&nomos_check_orchestration::CheckOutcome::NoSource, stdout, stderr);
            }
            Some(walked) => Body::Check(walked),
            None =>
            {
                let CheckBody { root, .. } = check;
                let _ = writeln!(stderr, "`{}` is not a directory", root.display());
                return ExitCode::Unavailable;
            }
        },
        Body::Correction(correction) => match Walked_Correction(correction)
        {
            Some(walked) => Body::Correction(walked),
            None =>
            {
                let CorrectionBody { root, .. } = correction;
                let _ = writeln!(stderr, "`{}` is not a directory", root.display());
                return ExitCode::Unavailable;
            }
        },
        Body::Gate(gate) => match Walked_Gate(gate)
        {
            Some(walked) => Body::Gate(walked),
            None =>
            {
                let _ = writeln!(stderr, "`{}` is not a directory", gate.command.root.display());
                return ExitCode::Unavailable;
            }
        },
        other => other.clone(),
    };

    let plan = [WorkflowStepPlan { declaration: Coherent_Declaration(), body }];
    let platform = Platform { launcher: &LAUNCHER, filesystem: &FILE_SYSTEM, environment: &ENVIRONMENT };
    let run = Fresh_Run_Id(CLOCK.Now());
    let outcome = nomos_workflow_orchestration::Run(&plan, &platform, &Workflow_Variant(), run);

    return Rendered(&outcome, stdout, stderr);
}

/// `check`, with its own `sources` replaced by a real walk of its `root` -- `None` if
/// `root` is not a directory, the identical guard `gate.rs`'s and `correct.rs`'s own walks
/// already give a tree that cannot be read at all.
fn Walked(check: &CheckBody) -> Option<CheckBody>
{
    if !check.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&check.root);

    return Some(CheckBody::New(check.root.clone(), sources, check.selected.clone()));
}

/// `correction`, with its own `sources` replaced by a real walk of its `root` -- `None` if
/// `root` is not a directory, the identical guard [`Walked`] gives `Body::Check`.
/// `Run_Correction` itself already reports `NoSourceFound` for an empty, but real, walk,
/// so there is no second empty-source guard to duplicate here the way [`Run`] keeps for
/// `Body::Check`.
fn Walked_Correction(correction: &CorrectionBody) -> Option<CorrectionBody>
{
    if !correction.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&correction.root);

    return Some(CorrectionBody::New(correction.root.clone(), sources, correction.commit));
}

/// `gate`, with its own `sources` replaced by a real walk of `gate.command.root` -- `None`
/// if that root is not a directory, the identical guard [`Walked`] and [`Walked_Correction`]
/// both already give.
fn Walked_Gate(gate: &GateBody) -> Option<GateBody>
{
    if !gate.command.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&gate.command.root);

    return Some(GateBody::New(sources, gate.command.clone()));
}

/// Every `.rs` or `.go` file under `root`, with its text and the subject its facts are
/// filed under -- a near-duplicate of `correct.rs`'s own walk rather than a shared
/// dependency on it, the identical division that module's own doc names: a composition
/// root's own territory is per group, not shared through a third module neither group's
/// item reserved.
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

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go") && let Ok(text) = std::fs::read_to_string(&path)
    {
        sources.push(Read_Source(root, &path, text));
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

/// The one `WorkflowStep` declaration this verb ever builds -- fixed and always coherent,
/// so `Is_Coherent` never refuses the one step this command composes. A future increment
/// parsing a real multi-step plan from argv is what would ever construct one that is not.
fn Coherent_Declaration() -> WorkflowStep
{
    return WorkflowStep {
        input_schema: SchemaId::New("nomos.workflow.cli.input.v1"),
        output_schema: SchemaId::New("nomos.workflow.cli.output.v1"),
        has_side_effects: false,
        idempotent: true,
        retry: RetryPolicy::NoRetry,
        timeout: Timeout::Unbounded,
        cacheability: Cacheability::NotCacheable,
        privileges: Vec::new(),
        cancellation: CancellationBehavior::Uncancellable,
        compensation: Compensation::None,
        determinism_strength: DeterminismStrength::None,
        reproducibility_scope: ReproducibilityScope::SingleRun,
        trace_equivalence: TraceEquivalence::NotApplicable,
        evidence: EvidenceClass::AgentJudged,
    };
}

/// Renders `outcome` and reports the [`ExitCode`] it earns.
fn Rendered(outcome: &WorkflowOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        WorkflowOutcome::Refused { .. } =>
        {
            let _ = writeln!(stderr, "the one step this command composed declared itself incoherent");
            ExitCode::Refused
        }
        WorkflowOutcome::Failed { error: nomos_workflow_orchestration::DispatchError::Gate(result), .. } =>
        {
            let _ = writeln!(stderr, "the gate step failed: {} blocking finding(s)", result.findings.blocking_findings.len());
            ExitCode::Refused
        }
        WorkflowOutcome::Failed { error, .. } =>
        {
            let _ = writeln!(stderr, "the step's dispatch failed: {error:?}");
            ExitCode::Unavailable
        }
        WorkflowOutcome::Completed { completed } => match completed.first()
        {
            Some(step) => Rendered_Step(step, stdout, stderr),
            None =>
            {
                let _ = writeln!(stderr, "completed with no step run, which this command never asks for");
                ExitCode::Usage
            }
        },
    };
}

/// The one step's own outcome, rendered.
fn Rendered_Step(step: &StepOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match step
    {
        StepOutcome::ClaudeCode(answer) =>
        {
            let _ = writeln!(stdout, "assumptions: {:?}", answer.result.assumptions);
            let _ = writeln!(stdout, "unresolved questions: {:?}", answer.result.unresolved_questions);
            ExitCode::Ok
        }
        StepOutcome::Ollama(answer) =>
        {
            let _ = writeln!(stdout, "{}", answer.response);
            ExitCode::Ok
        }
        StepOutcome::Check(check) => Rendered_Check(check, stdout, stderr),
        StepOutcome::Correction(correction) => Rendered_Correction(correction, stdout, stderr),
        StepOutcome::Gate(result) => Rendered_Gate(result, stdout),
    };
}

/// A `Body::Check` step's own [`nomos_check_orchestration::CheckOutcome`], rendered --
/// `Judged` reports `Ok` regardless of its findings; see this module's own doc for why.
fn Rendered_Check(outcome: &nomos_check_orchestration::CheckOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    use nomos_check_orchestration::CheckOutcome;

    return match outcome
    {
        CheckOutcome::Unreadable =>
        {
            let _ = writeln!(stderr, "the tree could not be read as a workspace state");
            ExitCode::Unavailable
        }
        CheckOutcome::Contradictory(error) =>
        {
            let _ = writeln!(stderr, "this build's own capability registry is self-contradictory: {error:?}");
            ExitCode::Unavailable
        }
        CheckOutcome::NoSource =>
        {
            let _ = writeln!(stderr, "no `.rs` or `.go` source found under the named root");
            ExitCode::Vacuous
        }
        CheckOutcome::NoFacts { files } =>
        {
            let _ = writeln!(stderr, "{files} file(s) were read but no syntax fact was materialized for any of them");
            ExitCode::Vacuous
        }
        CheckOutcome::Judged { findings, .. } =>
        {
            let _ = writeln!(stdout, "judged: {} finding(s)", findings.len());
            ExitCode::Ok
        }
    };
}

/// A `Body::Gate` step's own [`nomos_gate_orchestration::GateRunResult`], rendered.
///
/// `GateRunOutcome::Failed` is not rendered here: `Dispatch` reports a failing gate as
/// `DispatchError::Gate` before this function ever sees a `StepOutcome::Gate` at all, so
/// `Passed` and `Indeterminate` are the only two dispositions a caller can reach through
/// this path. `Failed` is still matched, defensively, as `Vacuous` rather than assumed
/// unreachable and panicked on -- a total function over every value the type can hold,
/// the same discipline every other render in this module already keeps.
fn Rendered_Gate(result: &nomos_gate_orchestration::GateRunResult, stdout: &mut impl Write) -> ExitCode
{
    use nomos_gate_orchestration::GateRunOutcome;

    return match result.disposition
    {
        GateRunOutcome::Passed =>
        {
            let _ = writeln!(stdout, "gate passed: {} finding(s), none blocking", result.findings.blocking_findings.len());
            ExitCode::Ok
        }
        GateRunOutcome::Indeterminate | GateRunOutcome::Failed =>
        {
            let _ = writeln!(stdout, "gate did not reach a judgment it could pass or fail");
            ExitCode::Vacuous
        }
    };
}

/// A `Body::Correction` step's own [`nomos_correction_orchestration::CorrectionOutcome`],
/// rendered -- the identical mapping `correct.rs`'s own render already uses, since this
/// verb reports the same seam's own outcome rather than a second reading of it.
fn Rendered_Correction(outcome: &nomos_correction_orchestration::CorrectionOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    use nomos_correction_orchestration::CorrectionOutcome;

    return match outcome
    {
        CorrectionOutcome::UnreadableRoot =>
        {
            let _ = writeln!(stderr, "the named root is not a directory");
            ExitCode::Unavailable
        }
        CorrectionOutcome::NoSourceFound =>
        {
            let _ = writeln!(stderr, "no `.rs` or `.go` source found under the named root");
            ExitCode::Vacuous
        }
        CorrectionOutcome::UnreadableWorkspaceState =>
        {
            let _ = writeln!(stderr, "the tree could not be read as a workspace state");
            ExitCode::Unavailable
        }
        CorrectionOutcome::ContradictoryRegistry(error) =>
        {
            let _ = writeln!(stderr, "this build's own capability registry is self-contradictory: {error:?}");
            ExitCode::Unavailable
        }
        CorrectionOutcome::NoFactsMaterialized(files) =>
        {
            let _ = writeln!(stderr, "{files} file(s) were read but no syntax fact was materialized for any of them");
            ExitCode::Vacuous
        }
        CorrectionOutcome::Clean =>
        {
            let _ = writeln!(stdout, "clean: no blocking correction claim under the named root");
            ExitCode::Ok
        }
        CorrectionOutcome::Refused(reason) =>
        {
            let _ = writeln!(stderr, "{reason}");
            ExitCode::Refused
        }
        CorrectionOutcome::Staged { path, summary, preview } =>
        {
            let _ = writeln!(stdout, "{}", String::from_utf8_lossy(preview));
            let _ = writeln!(stdout, "dry run: `{path}`: {summary}. Pass --commit to apply it.");
            ExitCode::Ok
        }
        CorrectionOutcome::Committed { path, summary, preview, base, after_snapshot } =>
        {
            let _ = writeln!(stdout, "{}", String::from_utf8_lossy(preview));
            let _ = writeln!(stdout, "committed: `{path}`: {summary} ({base} -> {after_snapshot})");
            ExitCode::Ok
        }
    };
}

/// [`Usage_Text`]'s content.
const USAGE_TEXT: &str = "usage: nomos workflow <command>\n\
        \n\
        \x20 run --check --root <path> [--rule <id>]...\n\
        \x20 run --correct --root <path> [--commit]\n\
        \x20 run --gate --root <path> [--rule <id>]...\n\
        \x20 run --executor claude-code --goal <text>\n\
        \x20 run --model-backend ollama --goal <text>\n\
        \n\
        Dispatches the one step this command composes through `nomos-workflow-\
        orchestration::Run`, over a fixed, always-coherent WorkflowStep declaration -- a \
        thin renderer over that seam, not a second place workflow semantics live. Pass \
        exactly one of --check, --correct, --gate, --executor or --model-backend; a step \
        dispatches through exactly one.\n\
        \n\
        --check walks --root (default the current directory) for `.rs` and `.go` source \
        and runs nomos-check-orchestration::Run over it, narrowed to --rule if given \
        (repeatable; empty selects every registered rule). Reports Ok for a Judged \
        outcome regardless of its findings -- run `nomos gate run` for a pass/fail \
        reading.\n\
        \n\
        --correct walks --root (default the current directory) the identical way \
        --check does and runs nomos-correction-orchestration::Run_Correction over it, \
        staging and validating a real blocking claim from either correction family and, \
        only with --commit, writing the fix back. The identical seam and the identical \
        --commit flag `nomos correct` already has.\n\
        \n\
        --gate walks --root the identical way --check does and runs nomos-gate-\
        orchestration::Run_Gate over it, narrowed to --rule the identical way --check is. \
        Unlike every other body, a failing gate ends the workflow rather than merely \
        completing as this step's own answer -- exit code 1, the same code an incoherent \
        step's own refusal already carries.\n\
        \n\
        --executor and --model-backend name which family of backend --goal dispatches \
        to, the identical two flags and the identical restriction `nomos agent execute` \
        already has.\n\
        \n\
        exit codes: 0 ok, 1 the one step refused itself or a gate step failed, 2 usage, \
        5 the named root, registry, executor or model backend could not be read, run, or \
        answered at all, 6 the check, correction or gate step found nothing to judge";

fn Usage_Text() -> String
{
    return USAGE_TEXT.to_owned();
}

/// The build variant this binary was compiled as. A near-duplicate of `correct.rs`'s own
/// `Correction_Variant`, not a shared dependency on it -- that function is private to
/// `correct`, the identical reasoning `correct.rs`'s own doc gives for not sharing
/// `check.rs`'s walk.
fn Workflow_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
}

/// What `nomos workflow` tells the shell.
///
/// The numbers are shared with every other group on this binary -- see `check::ExitCode`'s
/// own doc for the fuller statement of that discipline. [`ExitCode::Unavailable`] carries
/// the meaning `agent`'s own `Unavailable` and `check`'s own `Unreadable` already give `5`:
/// the thing this step needed could not be read, started, or answered at all, whichever of
/// the three bodies it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The step ran and answered -- a `Judged` check regardless of its findings, or a real
    /// response from `ClaudeCode`/`Ollama`.
    Ok = 0,
    /// The one step this command composed declared itself incoherent -- never reachable
    /// through this command today, since [`Coherent_Declaration`] is fixed, but a real
    /// answer this match must still give.
    Refused = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The named root could not be read as a directory or a workspace state, this build's
    /// capability registry is self-contradictory, or the chosen executor/model backend
    /// could not be started, exited non-zero, or timed out.
    Unavailable = 5,
    /// A check step found no source under its root, or no syntax fact was materialized for
    /// any of it.
    Vacuous = 6,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_A_Check_Run()
    {
        let arguments: Vec<String> = ["run", "--check", "--root", "."].iter().map(|value| return (*value).to_owned()).collect();

        let command = Command_From_String_Arguments(&arguments).expect("parses");

        assert!(matches!(command.body, Body::Check(_)), "{command:?}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_A_Correct_Run()
    {
        let arguments: Vec<String> = ["run", "--correct", "--root", ".", "--commit"].iter().map(|value| return (*value).to_owned()).collect();

        let command = Command_From_String_Arguments(&arguments).expect("parses");

        assert!(matches!(command.body, Body::Correction(ref correction) if correction.commit), "{command:?}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_A_Gate_Run()
    {
        let arguments: Vec<String> =
            ["run", "--gate", "--root", ".", "--rule", "naming-convention"].iter().map(|value| return (*value).to_owned()).collect();

        let command = Command_From_String_Arguments(&arguments).expect("parses");

        assert!(matches!(command.body, Body::Gate(ref gate) if gate.command.rules.include == vec![RuleId::New("naming-convention")]), "{command:?}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Parse_A_Claude_Code_Run()
    {
        let arguments: Vec<String> =
            ["run", "--executor", "claude-code", "--goal", "say hello"].iter().map(|value| return (*value).to_owned()).collect();

        let command = Command_From_String_Arguments(&arguments).expect("parses");

        assert!(matches!(command.body, Body::ClaudeCode(ref task) if task.goal == "say hello"), "{command:?}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_More_Than_One_Body()
    {
        let arguments: Vec<String> =
            ["run", "--check", "--executor", "claude-code"].iter().map(|value| return (*value).to_owned()).collect();

        let error = Command_From_String_Arguments(&arguments).expect_err("two bodies named");

        assert!(error.contains("exactly one"), "{error}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_No_Body_Named()
    {
        let arguments: Vec<String> = ["run"].iter().map(|value| return (*value).to_owned()).collect();

        let error = Command_From_String_Arguments(&arguments).expect_err("no body named");

        assert!(error.contains("is required"), "{error}");
    }

    #[test]
    fn Test_Command_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
    {
        let arguments: Vec<String> = ["not-a-real-verb"].iter().map(|value| return (*value).to_owned()).collect();

        let error = Command_From_String_Arguments(&arguments).expect_err("no such verb");

        assert!(error.contains("unknown command"), "{error}");
    }

    #[test]
    fn Test_Run_Should_Report_Unavailable_Over_A_Root_That_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-workflow-missing-root");
        let _ignored = std::fs::remove_dir_all(&root);
        let command = WorkflowCommand { body: Body::Check(CheckBody::New(root, Vec::new(), Vec::new())) };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Unavailable);
    }

    #[test]
    fn Test_Run_Should_Report_Vacuous_Over_An_Empty_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-workflow-empty-root");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates an empty directory");
        let command = WorkflowCommand { body: Body::Check(CheckBody::New(root.clone(), Vec::new(), Vec::new())) };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(code, ExitCode::Vacuous);
    }

    #[test]
    fn Test_Run_Should_Report_Ok_Over_A_Clean_Real_Source_File()
    {
        let root = std::env::temp_dir().join("nomos-cli-workflow-clean-root");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a directory");
        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");
        let command = WorkflowCommand {
            body: Body::Check(CheckBody::New(root.clone(), Vec::new(), vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)])),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
    }

    #[test]
    fn Test_Run_Should_Commit_A_Real_Phantom_Claim_Through_Correct()
    {
        const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
            /// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n\
            pub const THINGS: &[&str] = &[\"a\"];\n";
        let root = std::env::temp_dir().join("nomos-cli-workflow-correct-commit-root");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a directory");
        std::fs::write(root.join("a.rs"), PHANTOM_FIXTURE).expect("writable");
        let command = WorkflowCommand { body: Body::Correction(CorrectionBody::New(root.clone(), Vec::new(), true)) };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);
        let corrected = std::fs::read_to_string(root.join("a.rs")).expect("still readable");

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
        assert_eq!(corrected, "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n");
    }

    #[test]
    fn Test_Run_Should_Report_Ok_For_A_Passing_Gate()
    {
        let root = std::env::temp_dir().join("nomos-cli-workflow-gate-passing-root");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a directory");
        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");
        let command = WorkflowCommand {
            body: Body::Gate(GateBody::New(
                Vec::new(),
                GateCommand { root: root.clone(), rules: RuleSelector { include: vec![RuleId::New(nomos_rules::PARAMETER_COUNT)] }, ..GateCommand::default() },
            )),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
    }

    #[test]
    fn Test_Run_Should_Report_Refused_For_A_Failing_Gate()
    {
        let root = std::env::temp_dir().join("nomos-cli-workflow-gate-failing-root");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a directory");
        std::fs::write(root.join("a.rs"), "pub fn Something(a: i32, b: i32, c: i32, d: i32, e: i32) {}\n").expect("writable");
        let command = WorkflowCommand {
            body: Body::Gate(GateBody::New(
                Vec::new(),
                GateCommand { root: root.clone(), rules: RuleSelector { include: vec![RuleId::New(nomos_rules::PARAMETER_COUNT)] }, ..GateCommand::default() },
            )),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Run(&command, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(code, ExitCode::Refused, "{}", String::from_utf8_lossy(&stderr));
    }
}
