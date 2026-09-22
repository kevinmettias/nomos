//! The whole step set, read from the workflow in one pass.
//!
//! Beside `gate_unknown.rs` rather than inside it. That file answers what the gate checks
//! for one named step, which is what `work finish` needs; this one answers what the whole
//! set is, which is what `OD-GATE-033`'s local execution needs. Two questions about one
//! document, and the file that held both had grown past the size at which the standards
//! ask for a submodule.
//!
//! The resolution of a `run:` line is deliberately not duplicated here: [`Derived_From`]
//! reaches back to the parent's own `Argv_Of`, so the single-step and whole-set readers
//! cannot come to disagree about what a derivable command is.

use super::{Argv_Of, DerivedStep, Step_Named, StepGuard, StepName, WorkflowText};

/// One step mid-read, before its `run:` has been turned into an argv or a refusal.
struct ParsedStep
{
    name: String,
    guard: StepGuard,
    run: Option<String>,
}

/// Every step the workflow declares, in file order, each with its guard and its argv.
///
/// Beside [`Derive_Step`] rather than replacing it: that function answers for one named
/// step and `work finish` depends on exactly that, so it keeps its behaviour including its
/// indifference to every key but `run:`. This answers the different question `OD-GATE-033`
/// needs -- what the whole set is, and which legs each member belongs to.
///
/// Unreadable parts are carried rather than dropped. A step whose guard is outside the
/// subset still appears, with [`StepGuard::Outside`]; a step with no derivable command
/// still appears, with the cause in its `argv`. A reader that dropped either would hand a
/// caller a set that looks complete and is not.
#[must_use]
pub fn Derive_Steps<'a>(workflow: impl Into<WorkflowText<'a>>) -> Vec<DerivedStep>
{
    let workflow = workflow.into();
    let mut parsed: Vec<ParsedStep> = Vec::new();

    for line in workflow.As_Text().lines()
    {
        Read_Into(&mut parsed, line);
    }

    return parsed.into_iter().map(Derived_From).collect();
}

/// One workflow line folded into the steps read so far.
///
/// A `name:` line nested under something opens a step. Anything else belongs to the step
/// currently open, and a line arriving before any step has opened belongs to nobody.
fn Read_Into(parsed: &mut Vec<ParsedStep>, line: &str)
{
    let trimmed = line.trim();

    if let Some(name) = Step_Named(trimmed)
        && Is_Nested(line)
    {
        parsed.push(Opened_Step(name));
        return;
    }

    if let Some(current) = parsed.last_mut()
    {
        Fold_Into(current, trimmed);
    }
}

/// Whether a line is nested under something, rather than a key of the document itself.
///
/// The discriminator is indentation, and it is needed here and not in [`Derive_Step`]. That
/// function is asked for one step by name and is indifferent to every other `name:` in the
/// file; this one collects them all, and a workflow's own `name:` key sits at column zero
/// looking exactly like a step's. Reading it as one produced a sixteenth step called `gate`
/// with no `run:`, which then reported as a refusal in a local run -- a step the workflow
/// does not have, named in the set of steps this host did not execute.
fn Is_Nested(line: &str) -> bool
{
    return line.starts_with(char::is_whitespace);
}

/// A step just opened by its `name:` line, with nothing read into it yet.
fn Opened_Step(name: &str) -> ParsedStep
{
    return ParsedStep { name: name.to_owned(), guard: StepGuard::Unguarded, run: None };
}

/// An `if:` or `run:` line folded into the step currently being read.
///
/// The first `run:` wins, which is the indifference to later ones [`super::Derive_Step`]
/// already has: a step declares one command, and a second is a workflow this reader does not
/// claim to understand.
fn Fold_Into(current: &mut ParsedStep, trimmed: &str)
{
    if let Some(guard) = trimmed.strip_prefix("if:")
    {
        current.guard = Guard_From(guard);
        return;
    }

    if let Some(run) = trimmed.strip_prefix("run:")
        && current.run.is_none()
    {
        current.run = Some(run.trim().to_owned());
    }
}

/// The guard an `if:` line declares, or [`StepGuard::Outside`] when it is not the one
/// spelling this reader implements.
///
/// The subset is deliberately one spelling wide. GitHub's expression language admits
/// boolean operators, functions and contexts this workspace does not evaluate, and a reader
/// that accepted more of it than it understood would decide a step's legs by accident.
fn Guard_From(text: &str) -> StepGuard
{
    let trimmed = text.trim();

    let Some(operand) = trimmed.strip_prefix("matrix.os ==")
    else
    {
        return StepGuard::Outside(trimmed.to_owned());
    };

    let quoted = operand.trim();
    let Some(host) = quoted.strip_prefix("'").and_then(|rest| return rest.strip_suffix("'"))
    else
    {
        return StepGuard::Outside(trimmed.to_owned());
    };

    if host.is_empty() || host.contains("'")
    {
        return StepGuard::Outside(trimmed.to_owned());
    }

    return StepGuard::Host(host.to_owned());
}

/// A read step, with its `run:` resolved through the same [`Argv_Of`] the single-step
/// derivation uses, so the two cannot disagree about what a derivable command is.
fn Derived_From(parsed: ParsedStep) -> DerivedStep
{
    let run = parsed.run.unwrap_or_default();
    let argv = Argv_Of(&run, StepName::from(parsed.name.as_str()));

    return DerivedStep { name: parsed.name, guard: parsed.guard, argv };
}
