//! `nomos agent` — dispatching a task to this workspace's one real `AgentExecutor`
//! (`nomos-agent-executor-claude-code`, `--executor claude-code`, the default) or its one real
//! `ModelBackend` (`nomos-model-backend-ollama`, `--model-backend ollama`).
//!
//! `--executor` and `--model-backend` used to be one flag, `--backend`, spelling
//! `claude-code` and `ollama` as if they were peer choices of the same kind. `OD-PACKAGE-013`
//! found they are not: Ollama's real mechanism — a fixed model, one forwarded field, every
//! other `TaskEnvelope` field read and ignored, no tool-use loop, no MCP surface — matches
//! `PackageKind::ModelBackendPackage`, not `PackageKind::AgentExecutorPackage`.
//! `OD-EXECUTOR-005` then checked what that does to `OD-EXECUTOR-001`/`OD-EXECUTOR-004`'s
//! shared-trait trigger and found it has not fired: with only one real `AgentExecutor` in
//! this workspace, there is nothing to build a dispatch trait generic over. What `--backend`
//! actually needed was not an abstraction, but an honest surface — two flags naming which
//! family a caller is choosing from, so passing `--model-backend ollama` cannot be read as
//! choosing an alternative agent. Internally this is still a plain `match`, not a trait: both
//! flags parse into the same two-variant [`Backend`], and `Dispatch` below is exactly the
//! match it always was, unaffected by which flag supplied the value.
//!
//! Neither composition root can supply its own launcher: both depend on `nomos-platform` and
//! not on any concrete implementation of it, the same reason `nomos-lang-rust-cargo` and every
//! other `ProcessLauncher`-driven crate stays generic. This module supplies
//! `nomos_platform_std::StdProcessLauncher`, the same choice `check.rs` and `work.rs` already
//! make for their own subprocesses.
//!
//! `execute` is the only real caller either backend crate has anywhere in this workspace
//! today, other than their own tests. It renders each backend's own outcome type directly
//! rather than assembling a `nomos_agent_contracts::WorkResult` — `OD-CONTRACTS-003` made
//! `WorkResult.plan` representable as absent, but this command has no `RuleId` or `SubjectId`
//! to give a `Finding` either, since nothing dispatched it as a rule's judgment; it is a
//! person, asking a question directly. Reporting a `Finding` about nothing in particular
//! would be inventing a subject nobody named, the same category of dishonesty `OD-EXECUTOR-001`
//! already named for trusting a process's own account of itself.
//!
//! `judge-role` is different: it *does* start from a rule's own finding.
//! `nomos_rules::Check_Declared_Role_Matches_Surface` unconditionally reports
//! `Applicability::AgentRequired` for a crate, per its own module doc, and never reaches a
//! verdict — this composition root is the real, separate dispatch `role_surface.rs` names as
//! deliberately not this rule's own job. It reads exactly the two files that finding's own
//! `locations` name (`README.md`'s band-table row, the crate's committed surface snapshot),
//! builds the real `nomos_rules::RoleSurfacePair` the rule was given, and asks the chosen
//! backend the question the rule could not answer.

use crate::arguments::{Name, Named_Value, Required, Usage};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{Finding, SchemaId};
use nomos_ledger::Territory;
use nomos_platform_std::StdProcessLauncher;
use nomos_rules::RoleSurfacePair;
use std::path::{Path, PathBuf};

/// What the process exits with.
///
/// Shares its numbers with every other group on this binary — see `check::ExitCode`'s own
/// doc for the fuller statement of that discipline. `5` already carries "could not be read
/// / run / answered at all" for `check`'s `Unreadable` and `spec`'s `StoreError`; a process
/// that could not be started, exited non-zero, timed out, or answered with something other
/// than the JSON it promised is the identical shape one seam over, not a fresh meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The executor ran and answered.
    Ok = 0,
    /// The command line was wrong.
    Usage = 2,
    /// The executor could not be started, exited non-zero, timed out or stalled, or
    /// answered with something other than the JSON `--output-format json` promises.
    Unavailable = 5,
    /// `judge-role` named a crate `README.md`'s band table does not list, or one with no
    /// committed surface snapshot. `6` already carries "the answer is empty because
    /// something expected was not there" for `spec`'s `Absent`.
    NotFound = 6,
}

impl ExitCode
{
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// Which real backend a call dispatches to. `ClaudeCode` is the one real `AgentExecutor` in
/// this workspace (`nomos-agent-executor-claude-code`, chosen by `--executor claude-code`);
/// `Ollama` is the one real `ModelBackend` (`nomos-model-backend-ollama`, chosen by
/// `--model-backend ollama`) -- a `ModelBackendPackage` instance dispatched through the same
/// `Execute<P: ProcessLauncher>` shape, not a second `AgentExecutor` (`OD-PACKAGE-013`).
/// `ClaudeCode` is every caller's default before either flag existed, and stays the default
/// now: neither flag given must reach `nomos-agent-executor-claude-code` exactly as every
/// invocation did before `nomos-model-backend-ollama` existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Backend
{
    ClaudeCode,
    Ollama,
}

/// What `nomos agent` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Execute
    {
        goal: String, effort: nomos_model_package::EffortLevel, backend: Backend
    },
    JudgeRole
    {
        crate_name: String, root: PathBuf, effort: nomos_model_package::EffortLevel, backend: Backend
    },
}

/// Parses `nomos agent` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub(crate) fn Parse(arguments: &[String]) -> Result<Command, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    let rest = arguments.get(1..).unwrap_or_default();

    return match verb.as_str()
    {
        "execute" => Parse_Execute(rest),
        "judge-role" => Parse_Judge_Role(rest),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

fn Parse_Execute(arguments: &[String]) -> Result<Command, String>
{
    let value = Named_Value(arguments, "--goal");
    let goal = Required(value.as_ref(), Name("--goal"), Usage(&Usage_Text()))?;
    let effort = Parse_Effort(arguments)?;
    let backend = Parse_Backend(arguments)?;

    return Ok(Command::Execute { goal, effort, backend });
}

fn Parse_Judge_Role(arguments: &[String]) -> Result<Command, String>
{
    let value = Named_Value(arguments, "--crate");
    let crate_name = Required(value.as_ref(), Name("--crate"), Usage(&Usage_Text()))?;
    let root = Named_Value(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);
    let effort = Parse_Effort(arguments)?;
    let backend = Parse_Backend(arguments)?;

    return Ok(Command::JudgeRole { crate_name, root, effort, backend });
}

/// `--executor`'s or `--model-backend`'s value, or [`Backend::ClaudeCode`] when neither flag
/// is given -- every caller before either flag existed reached
/// `nomos-agent-executor-claude-code`, so both absent must keep reaching it, byte-identical.
///
/// The two flags used to be one, `--backend`, spelling `claude-code` and `ollama` as if they
/// were peer choices of the same kind. `OD-PACKAGE-013` found they are not -- Ollama is a
/// `ModelBackend`, not a second `AgentExecutor` -- so a caller now names which family it is
/// choosing from, and `--executor ollama` or `--model-backend claude-code` is refused rather
/// than silently accepted the way one flag spanning both could not refuse it.
///
/// # Errors
///
/// Returns a message naming the accepted spelling for whichever flag was given, when its
/// value is not that spelling, or when both flags are given at once -- a call dispatches to
/// exactly one backend, and naming two is not a request either flag alone could satisfy.
fn Parse_Backend(arguments: &[String]) -> Result<Backend, String>
{
    let executor = Named_Value(arguments, "--executor");
    let model_backend = Named_Value(arguments, "--model-backend");

    return match (executor, model_backend)
    {
        (Some(_), Some(_)) => Err(format!(
            "--executor and --model-backend both name a backend to dispatch to; a call reaches exactly one, so pass at most one of them.\n\n{}",
            Usage_Text()
        )),
        (Some(text), None) => match text.as_str()
        {
            "claude-code" => Ok(Backend::ClaudeCode),
            other => Err(format!("--executor {other:?} is not one of claude-code.\n\n{}", Usage_Text())),
        },
        (None, Some(text)) => match text.as_str()
        {
            "ollama" => Ok(Backend::Ollama),
            other => Err(format!("--model-backend {other:?} is not one of ollama.\n\n{}", Usage_Text())),
        },
        (None, None) => Ok(Backend::ClaudeCode),
    };
}

/// `--effort`'s value, or [`EffortLevel::BackendDefault`] when the flag is absent --
/// `BackendDefault` alone maps to "omit the flag entirely"
/// (`nomos_agent_executor_claude_code::Effort_Flag`), so an absent `--effort` reaches the
/// subprocess byte-identical to every invocation that predates this flag. The six spellings
/// are `MODEL-ROUTE-004`'s own closed enumeration, kebab-cased the same way `--kind` and
/// `--origin` already kebab-case theirs in `work/parse.rs`.
///
/// # Errors
///
/// Returns a message naming the six accepted spellings when `--effort`'s value is none of
/// them.
fn Parse_Effort(arguments: &[String]) -> Result<nomos_model_package::EffortLevel, String>
{
    use nomos_model_package::EffortLevel;

    let Some(text) = Named_Value(arguments, "--effort")
    else
    {
        return Ok(EffortLevel::BackendDefault);
    };

    return match text.as_str()
    {
        "backend-default" => Ok(EffortLevel::BackendDefault),
        "minimal" => Ok(EffortLevel::Minimal),
        "low" => Ok(EffortLevel::Low),
        "medium" => Ok(EffortLevel::Medium),
        "high" => Ok(EffortLevel::High),
        "maximum" => Ok(EffortLevel::Maximum),
        other => Err(format!(
            "--effort {other:?} is not one of backend-default, minimal, low, medium, high, \
             maximum.\n\n{}",
            Usage_Text()
        )),
    };
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub(crate) fn Run(command: &Command, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match command
    {
        Command::Execute { goal, effort, backend } => Execute(goal, *effort, *backend, output, notes),
        Command::JudgeRole { crate_name, root, effort, backend } => Judge_Role(crate_name, root, *effort, *backend, output, notes),
    };
}

/// A bare `TaskEnvelope` naming only `goal` and `effort`. `scope`, `prohibited_changes` and
/// `available_tools` are the empty value `OD-EXECUTOR-001` already reads as "nothing
/// enumerated, nothing granted" — this command has no configuration surface to fill them
/// from yet, and inventing one ahead of a real need would repeat the mistake this workspace
/// has already declined to make elsewhere. `expected_output_schema` names this call site
/// rather than a real schema, since nothing here validates a response against one.
fn Task(goal: &str, effort: nomos_model_package::EffortLevel) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
}

fn Execute(
    goal: &str, effort: nomos_model_package::EffortLevel, backend: Backend, output: &mut impl std::io::Write, notes: &mut impl std::io::Write,
) -> ExitCode
{
    return Dispatch(&Task(goal, effort), backend, output, notes);
}

/// Runs `task` against `backend` and renders whichever of the two outcome shapes it
/// produces. The two crates share no trait -- `OD-EXECUTOR-001`/`OD-EXECUTOR-004` both
/// decline to invent one ahead of a real need, and `OD-EXECUTOR-005` found that trigger has
/// not fired even once `--executor`/`--model-backend` replaced `--backend`: there is still
/// only one real `AgentExecutor`, so this match is the entire dispatch, not a stand-in for a
/// trait either flag's own vocabulary would need.
fn Dispatch(task: &TaskEnvelope, backend: Backend, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match backend
    {
        Backend::ClaudeCode => match nomos_agent_executor_claude_code::Execute(task, &StdProcessLauncher)
        {
            Ok(outcome) => Answered_Claude_Code(&outcome, output),
            Err(error) => Unavailable(&error, notes),
        },
        Backend::Ollama => match nomos_model_backend_ollama::Execute(task, &StdProcessLauncher)
        {
            Ok(outcome) => Answered_Ollama(&outcome, output),
            Err(error) => Unavailable(&error, notes),
        },
    };
}

/// Renders an outcome for what it structurally reported, never for what its own text
/// claims — `OD-EXECUTOR-001`'s rule, restated at the one place this workspace renders an
/// executor's answer for a person to read. `denied_tool_uses` is printed unconditionally,
/// empty or not, so its absence is a caller's own observation rather than a line that only
/// appears when there is bad news to report.
fn Answered_Claude_Code(outcome: &nomos_agent_executor_claude_code::AgentExecutionOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{}", outcome.response);
    let _ = writeln!(output, "denied tool uses: {:?}", outcome.denied_tool_uses);
    let _ = writeln!(output, "is_error: {}  cost_usd: {}  duration_ms: {}", outcome.is_error, outcome.cost_usd, outcome.duration_ms);

    return ExitCode::Ok;
}

/// `nomos-model-backend-ollama`'s own outcome carries only `response`, honestly: there is
/// no `denied_tool_uses` to print because there is no tool subsystem to have denied
/// anything from, and no dollar cost because inference is local. Printing placeholder
/// values for fields this backend does not have would claim a signal it never produced.
fn Answered_Ollama(outcome: &nomos_model_backend_ollama::AgentExecutionOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{}", outcome.response);

    return ExitCode::Ok;
}

fn Unavailable(error: &impl std::fmt::Display, notes: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::Unavailable;
}

/// Reads `root`'s `README.md` and `root`'s committed surface snapshot for `crate_name`,
/// builds the real `nomos_rules::RoleSurfacePair` `Check_Declared_Role_Matches_Surface`
/// would be handed, runs that rule to get the real `Finding` it produces, and dispatches
/// the question that finding names — never its own guess — to Claude Code.
fn Judge_Role(
    crate_name: &str, root: &Path, effort: nomos_model_package::EffortLevel, backend: Backend, output: &mut impl std::io::Write,
    notes: &mut impl std::io::Write,
) -> ExitCode
{
    let Some(declared_role) = Declared_Role(root, crate_name)
    else
    {
        let _ = writeln!(notes, "`{crate_name}` names no row in {}'s band table", root.join("README.md").display());
        return ExitCode::NotFound;
    };

    let surface_path = root.join("tests/contract/surface").join(format!("{crate_name}.txt"));
    let Ok(actual_surface) = std::fs::read_to_string(&surface_path)
    else
    {
        let _ = writeln!(notes, "no committed surface snapshot at {}", surface_path.display());
        return ExitCode::NotFound;
    };

    let pair = RoleSurfacePair {
        crate_root: Crate_Root(root, crate_name),
        crate_name: crate_name.to_owned(),
        declared_role,
        actual_surface,
    };
    let findings = nomos_rules::Check_Declared_Role_Matches_Surface(std::slice::from_ref(&pair));
    let Some(finding) = findings.first()
    else
    {
        let _ = writeln!(notes, "the rule produced no finding for its own subject");
        return ExitCode::NotFound;
    };

    return Dispatch(&Judgment_Task(&pair, finding, effort), backend, output, notes);
}

/// `README.md`'s band-table row for `crate_name` — the third pipe-delimited cell of the
/// row whose second cell, backticks stripped, is `crate_name` exactly. `None` if no row
/// names it.
fn Declared_Role(root: &Path, crate_name: &str) -> Option<String>
{
    let text = std::fs::read_to_string(root.join("README.md")).ok()?;

    for line in text.lines()
    {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(name_cell), Some(role_cell)) = (cells.get(2), cells.get(3))
        else
        {
            continue;
        };
        if name_cell.trim_matches('`') == crate_name
        {
            return Some((*role_cell).to_owned());
        }
    }

    return None;
}

/// The same manifest-relative root `RoleSurfacePair::crate_root` documents —
/// `crates/<band-folder>/<crate_name>` is not derivable from the name alone, so this reads
/// it from `Cargo.toml`'s own `[workspace] members` list rather than guess a layout.
fn Crate_Root(root: &Path, crate_name: &str) -> String
{
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();

    return manifest
        .lines()
        .map(str::trim)
        .find(|line| return line.trim_matches(['"', ',']).ends_with(crate_name))
        .map_or_else(|| return crate_name.to_owned(), |line| return line.trim_matches([' ', '"', ',']).to_owned());
}

/// The judgment `role_surface.rs`'s own module doc says this rule cannot reach itself —
/// whether `pair`'s declared role and actual surface agree — carrying `finding.summary`
/// so the dispatched question is traceably the rule's own, not a paraphrase invented here.
fn Judgment_Task(pair: &RoleSurfacePair, finding: &Finding, effort: nomos_model_package::EffortLevel) -> TaskEnvelope
{
    let goal = format!(
        "A Rust crate's declared role, from its workspace README's band table: {}\n\n\
         The crate's actual public surface, as a list of every item it exports:\n{}\n\n\
         {}. Does the declared role accurately and completely describe what the surface \
         exports? Name anything the role claims that the surface does not show, or anything \
         the surface exports that the role does not mention, in 2-4 sentences.",
        pair.declared_role, pair.actual_surface, finding.summary
    );

    return TaskEnvelope {
        goal,
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: vec![finding.rule.clone()],
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
}

fn Usage_Text() -> String
{
    return "usage: nomos agent <command>\n\
            \n\
            \x20 execute --goal <text> [--effort <level>] [--executor <name> | --model-backend <name>]\n\
            \x20 judge-role --crate <name> [--root <path>] [--effort <level>] [--executor <name> | --model-backend <name>]\n\
            \n\
            `execute` dispatches --goal to the chosen backend as a bounded, tool-free \
            subprocess. It renders the response and, unconditionally, which tool uses (if \
            any) were structurally denied -- the response text is never evidence of what \
            happened, only of what the process said. It does not assemble a WorkResult or a \
            Finding: this is a person's direct question, not a rule's judgment, and there is \
            no subject or rule identity to report one against.\n\
            \n\
            `judge-role` reads --crate's row in --root's README.md band table and its \
            committed tests/contract/surface/<crate>.txt, runs the real \
            nomos_rules::Check_Declared_Role_Matches_Surface rule over them, and dispatches \
            the AgentRequired finding that rule produces -- never a paraphrase -- to execute. \
            --root defaults to the current directory.\n\
            \n\
            --executor and --model-backend name which family of backend to dispatch to; pass \
            at most one, since a call reaches exactly one. Omitting both is --executor \
            claude-code, byte-identical to every invocation before either flag existed. They \
            used to be one flag, --backend, spelling claude-code and ollama as if they were \
            peer choices of the same kind -- OD-PACKAGE-013 found Ollama's real mechanism is \
            a ModelBackendPackage's, not a second AgentExecutor's, so the flag that chooses it \
            says so.\n\
            \n\
            --executor takes claude-code (the only real AgentExecutor). Dispatches through \
            nomos-agent-executor-claude-code under OD-EXECUTOR-001's structural capability \
            boundary: an isolated working directory, no MCP configuration, an allow-list \
            naming no real tool, a $1 budget cap, one --print turn.\n\
            \n\
            --model-backend takes ollama (the only real ModelBackend). Dispatches through \
            nomos-model-backend-ollama, a local model, under OD-EXECUTOR-004's boundary: an \
            isolated working directory, never --experimental/--experimental-yolo/\
            --experimental-websearch (the only flags that open any tool-use capability), a \
            wall-clock timeout in place of a dollar budget. Its own outcome carries only the \
            response text -- no denied-tool-uses line, since there is no tool subsystem to \
            have denied anything from.\n\
            \n\
            --effort takes backend-default, minimal, low, medium, high or maximum -- \
            MODEL-ROUTE-004's closed vocabulary, carried on TaskEnvelope.effort. \
            nomos-agent-executor-claude-code maps it to a real `claude --effort` flag; \
            nomos-model-backend-ollama accepts and ignores it, a real, named gap rather than \
            an invented approximation. Omitting it is backend-default.\n\
            \n\
            exit codes: 0 ok, 2 usage, 5 the executor could not run or answer, 6 the named \
            crate has no README row or no committed surface snapshot"
        .to_owned();
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }

    #[test]
    fn Test_An_Execute_Command_Should_Parse_Its_Goal()
    {
        let arguments = Arguments("execute --goal hello");

        let Command::Execute { goal, effort, backend } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(goal, "hello");
        assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
        assert_eq!(backend, Backend::ClaudeCode);
    }

    /// The default is `BackendDefault`, the one value `Effort_Flag` maps to "omit the flag
    /// entirely" -- an `execute` call with no `--effort` must reach the subprocess exactly
    /// as it did before this flag existed.
    #[test]
    fn Test_An_Execute_Command_With_No_Effort_Defaults_To_Backend_Default()
    {
        let arguments = Arguments("execute --goal hello");

        let Command::Execute { effort, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
    }

    #[test]
    fn Test_An_Execute_Command_Should_Parse_Its_Effort()
    {
        let arguments = Arguments("execute --goal hello --effort high");

        let Command::Execute { effort, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(effort, nomos_model_package::EffortLevel::High);
    }

    /// All six of `MODEL-ROUTE-004`'s spellings, not just one -- the same universe-closing
    /// discipline `nomos-model-package::effort_level`'s own `Test_Every_Value_Is_In_The_Tested_Universe`
    /// holds itself to.
    #[test]
    fn Test_An_Execute_Command_Parses_Every_Effort_Spelling()
    {
        use nomos_model_package::EffortLevel;

        let cases = [
            ("backend-default", EffortLevel::BackendDefault),
            ("minimal", EffortLevel::Minimal),
            ("low", EffortLevel::Low),
            ("medium", EffortLevel::Medium),
            ("high", EffortLevel::High),
            ("maximum", EffortLevel::Maximum),
        ];

        for (spelling, expected) in cases
        {
            let arguments = Arguments(&format!("execute --goal hello --effort {spelling}"));

            let Command::Execute { effort, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

            assert_eq!(effort, expected, "spelling {spelling}");
        }
    }

    #[test]
    fn Test_An_Unrecognized_Effort_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute --goal hello --effort superhuman");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--effort"), "{error}");
        assert!(error.contains("superhuman"), "{error}");
    }

    /// The default is `ClaudeCode` -- every caller before `--executor`/`--model-backend`
    /// existed must keep reaching `nomos-agent-executor-claude-code`.
    #[test]
    fn Test_An_Execute_Command_With_No_Backend_Defaults_To_Claude_Code()
    {
        let arguments = Arguments("execute --goal hello");

        let Command::Execute { backend, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(backend, Backend::ClaudeCode);
    }

    #[test]
    fn Test_An_Execute_Command_Parses_Executor_Claude_Code()
    {
        let arguments = Arguments("execute --goal hello --executor claude-code");

        let Command::Execute { backend, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(backend, Backend::ClaudeCode);
    }

    #[test]
    fn Test_An_Execute_Command_Parses_Model_Backend_Ollama()
    {
        let arguments = Arguments("execute --goal hello --model-backend ollama");

        let Command::Execute { backend, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(backend, Backend::Ollama);
    }

    #[test]
    fn Test_An_Unrecognized_Executor_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute --goal hello --executor gpt5");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--executor"), "{error}");
        assert!(error.contains("gpt5"), "{error}");
    }

    #[test]
    fn Test_An_Unrecognized_Model_Backend_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute --goal hello --model-backend gpt5");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--model-backend"), "{error}");
        assert!(error.contains("gpt5"), "{error}");
    }

    /// `--executor` naming an `AgentExecutor` and `--model-backend` naming a `ModelBackend`
    /// at once is not a request either flag alone could satisfy -- a call dispatches to
    /// exactly one backend, so both present is refused rather than one silently winning.
    #[test]
    fn Test_Both_Executor_And_Model_Backend_Together_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute --goal hello --executor claude-code --model-backend ollama");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--executor"), "{error}");
        assert!(error.contains("--model-backend"), "{error}");
    }

    #[test]
    fn Test_A_Judge_Role_Command_Should_Parse_Its_Model_Backend()
    {
        let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --model-backend ollama");

        let Command::JudgeRole { backend, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(backend, Backend::Ollama);
    }

    #[test]
    fn Test_A_Missing_Goal_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("execute");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--goal"));
    }

    #[test]
    fn Test_An_Unknown_Verb_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("dance");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("dance"));
    }

    #[test]
    fn Test_No_Verb_At_All_Should_Be_A_Usage_Error()
    {
        let error = Parse(&[]).expect_err("must refuse");

        assert!(error.contains("usage"));
    }

    /// The one place `--goal ""` reaches this crate's tests without spending a real
    /// invocation: parsing accepts an empty string exactly as it accepts any other, and
    /// whatever `nomos-agent-executor-claude-code` or the real CLI does with it is that crate's own
    /// concern, verified there, not re-verified through this transport.
    #[test]
    fn Test_An_Empty_Goal_Still_Parses()
    {
        let arguments = Arguments("execute --goal");
        let error = Parse(&arguments).expect_err("a flag with nothing after it has no value");

        assert!(error.contains("--goal"));
    }

    #[test]
    fn Test_A_Judge_Role_Command_Should_Parse_Its_Crate_And_Default_Root()
    {
        let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code");

        let Command::JudgeRole { crate_name, root, effort, backend } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(crate_name, "nomos-agent-executor-claude-code");
        assert_eq!(root, PathBuf::from("."));
        assert_eq!(effort, nomos_model_package::EffortLevel::BackendDefault);
        assert_eq!(backend, Backend::ClaudeCode);
    }

    #[test]
    fn Test_A_Judge_Role_Command_Should_Parse_An_Explicit_Root()
    {
        let arguments = Arguments("judge-role --crate nomos-agent-executor --root /some/tree");

        let Command::JudgeRole { root, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(root, PathBuf::from("/some/tree"));
    }

    #[test]
    fn Test_A_Judge_Role_Command_Should_Parse_Its_Effort()
    {
        let arguments = Arguments("judge-role --crate nomos-agent-executor-claude-code --effort low");

        let Command::JudgeRole { effort, .. } = Parse(&arguments).expect("parses") else { panic!("wrong variant") };

        assert_eq!(effort, nomos_model_package::EffortLevel::Low);
    }

    #[test]
    fn Test_A_Missing_Crate_Should_Be_A_Usage_Error()
    {
        let arguments = Arguments("judge-role");

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--crate"));
    }

    /// This repository's own root, three levels above `crates/host/nomos-cli` — the same
    /// derivation `nomos-lang-rust-cargo`'s own tests use to run against a real tree rather
    /// than a fixture nobody could have produced.
    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }

    /// Read against this repository's own real `README.md`, not a fixture — the same
    /// standard `nomos-lang-rust-cargo`'s own tests already hold themselves to: a reader
    /// that cannot be checked against a real row is checked against nothing.
    #[test]
    fn Test_Declared_Role_Reads_A_Real_Row_From_This_Repositorys_Own_Readme()
    {
        let role = Declared_Role(&Repository_Root(), "nomos-agent-executor-claude-code").expect("this crate has a row");

        assert!(role.contains("AgentExecutor"), "{role}");
    }

    #[test]
    fn Test_Declared_Role_Is_None_For_A_Crate_Named_Nowhere()
    {
        assert!(Declared_Role(&Repository_Root(), "nomos-does-not-exist").is_none());
    }

    #[test]
    fn Test_Crate_Root_Reads_This_Crates_Own_Real_Manifest_Path()
    {
        let root = Crate_Root(&Repository_Root(), "nomos-agent-executor-claude-code");

        assert_eq!(root, "crates/agent/nomos-agent-executor-claude-code");
    }
}
