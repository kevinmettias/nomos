//! What `nomos gate` was asked for.

use super::{FindingQuery, GateCommand, Invocation, Named_Value_From_String_Arguments, PathBuf};
use crate::arguments::{Name, Named_Values_From_String_Arguments, Required_Value, Usage};
use nomos_contracts::RuleId;
use nomos_gate_orchestration::{RuleSelector, ScopeSelector};

/// The flags this group accepts besides `--root`, `--rule` and `--location`.
const KNOWN_ARGUMENTS: [&str; 9] =
    ["--root", "--include", "--exclude", "--rule", "--location", "--against", "--from", "--to", "--host"];

pub(super) const USAGE: &str = "usage: nomos gate plan    [--root <path>] [--include <path>]… \
[--exclude <path>]… [--rule <id>]…\n       \
     nomos gate run     [--root <path>] [--include <path>]… [--exclude <path>]… [--rule <id>]…\n       \
     nomos gate policy  [--root <path>] [--include <path>]… [--exclude <path>]… [--rule <id>]…\n       \
     nomos gate explain [--root <path>] --rule <id> --location <path>\n       \
     nomos gate compare [--root <path>] --against <path> [--include <path>]… \
[--exclude <path>]… [--rule <id>]…\n       \
     nomos gate admits  --from <crate> --to <crate>\n       \
     nomos gate steps   [--root <path>] [--host <label>]\n\n\
     plan composes this gate's rule registry and reports what it holds.\n\
     run walks the tree, judges it, and reports a real disposition.\n\
     policy walks the tree and judges it exactly as run does, and reports which layer and \
artifact decided each field of the policy it was judged under, what that statement \
outranked, and any statement a lock refused with the reason. It changes no judgment and no \
disposition; it reports what the resolution already decided.\n\
     explain walks the tree, judges it, and reports what one named finding looks like and \
whether it would block.\n\
     compare walks --root and --against, judges each, and reports which findings were \
added, removed, or moved between buckets.\n\
     admits answers whether --from may name --to under the declared architecture, before any \
manifest carries the edge. It walks no tree, so --root does not apply to it: the architecture it answers from is read from the nearest enclosing directory that declares one, and outside a repository that declares none the answer is `not judged`.\n\
     steps executes this gate's own step set, read from .github/workflows/gate.yml, and \
reports what every step did here -- including the ones this host did not run, and why \
each was not run. It judges no tree.\n\n\
     --include/--exclude narrow which files `run` judges, by path prefix; repeat for \
several. For plan/run, --rule (repeatable) narrows which rules' findings can fail the \
build. For explain, --rule names the one rule whose finding to explain -- required, not \
repeatable -- alongside --location, one of that finding's own locations, also \
required. For compare, --against names the second tree to judge -- required -- and every \
other flag narrows both sides alike. For admits, --from and --to name the two crates -- both \
required, neither repeatable. For steps, --host names the matrix label to execute as, \
defaulting to the label this operating system corresponds to.\n\n\
     exit codes: 0 clean (the plan was composed, nothing judged can fail a build, \
explain's finding was not found or would not block, or admits permitted the edge or could \
not judge it),\n\
     \x20           1 at least one finding can fail a build, explain's finding would, or \
admits refused the edge, 2 usage,\n\
     \x20           5 this build's own composition is self-contradictory, or the tree could \
not be read,\n\
     \x20           6 nothing was judged: the walk found no source, or no fact was \
materialized for any of it.\n\
     compare reports a difference rather than judging one: it is clean whenever both sides \
were judged, whatever moved between them, and otherwise carries whichever code the side \
that could not be judged would have exited with on its own.";

/// Parses the group's arguments.
///
/// All four verbs `ARC-ROADMAP-001` names are recognized. `compare` was refused as usage
/// until `P73-GATE-COMPARE-HAS-NO-CALLER` gave it a body here, on the "no invented shape
/// ahead of a real body" discipline this group still holds to — what changed is that the
/// body exists, not the discipline.
///
/// # Errors
///
/// Returns the usage message when the verb is missing or unrecognized, `explain` is
/// missing `--rule` or `--location`, `compare` is missing `--against`, or an argument is
/// not understood.
pub fn Gate_Invocation_From_String_Arguments(arguments: &[String]) -> Result<Invocation, String>
{
    let Some((verb, rest)) = arguments.split_first()
    else
    {
        return Err(USAGE.to_owned());
    };

    Known_Verb(verb)?;
    No_Unknown_Argument(rest)?;

    let root = Named_Value_From_String_Arguments(rest, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from);

    return Verb_Invocation(verb, rest, root);
}

/// The matrix label an execution runs as: `--host` when given, and otherwise the label this
/// operating system corresponds to in the workflow's own matrix.
///
/// An operating system the matrix does not name is carried through verbatim rather than
/// mapped onto one of the two that it is not. Every guarded step is then unavailable and the
/// run says so, which is the honest answer on a machine no leg of this gate runs on -- and a
/// better one than silently reporting what a different host would have done.
fn Host_Label(rest: &[String]) -> String
{
    if let Some(named) = Named_Value_From_String_Arguments(rest, "--host")
    {
        return named;
    }

    return match std::env::consts::OS
    {
        "linux" => "ubuntu-latest".to_owned(),
        "windows" => "windows-latest".to_owned(),
        other => other.to_owned(),
    };
}

/// Refuses anything but the verbs this group implements today.
fn Known_Verb(verb: &str) -> Result<(), String>
{
    let is_unknown_verb = verb != "plan"
        && verb != "run"
        && verb != "policy"
        && verb != "explain"
        && verb != "compare"
        && verb != "admits"
        && verb != "steps";
    if is_unknown_verb
    {
        return Err(format!("unknown verb `{verb}`.\n\n{USAGE}"));
    }

    return Ok(());
}

/// Refuses any flag but `--root`, `--include`, `--exclude`, `--rule` and `--location`.
fn No_Unknown_Argument(rest: &[String]) -> Result<(), String>
{
    if let Some(unknown) = rest
        .iter()
        .find(|argument| return argument.starts_with('-') && !KNOWN_ARGUMENTS.contains(&argument.as_str()))
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(());
}

/// Turns a recognized verb's own flags into the invocation that verb names.
///
/// The three verbs carrying a shape of their own -- `explain`'s query, `compare`'s second
/// tree, `admits`' two crates -- are the ones whose arguments a shared `GateCommand` cannot
/// express, so they are dispatched rather than folded in. `plan` and `run` share one command
/// and differ only in which variant wraps it, which is the last line here.
fn Verb_Invocation(verb: &str, rest: &[String], root: PathBuf) -> Result<Invocation, String>
{
    if verb == "explain"
    {
        return Explain_Invocation(rest, root);
    }

    if verb == "compare"
    {
        return Compare_Invocation(rest, root);
    }

    if verb == "admits"
    {
        return Admits_Invocation(rest);
    }

    if verb == "steps"
    {
        return Ok(Invocation::Steps { root, host: Host_Label(rest) });
    }

    let command = Plan_Or_Run_Command(root, rest);

    if verb == "policy"
    {
        return Ok(Invocation::Policy(command));
    }

    return Ok(if verb == "run" { Invocation::Run(command) } else { Invocation::Plan(command) });
}

/// `explain`'s own required `--rule`/`--location`, read as single values rather than
/// `plan`/`run`'s repeatable `--rule`: naming one finding needs exactly one rule, not a
/// selector, so this does not populate `GateCommand::rules` at all -- `Explain_Gate`
/// ignores it regardless, and populating it from the same flag that also names the query
/// would read as a selector nobody asked for.
fn Explain_Invocation(rest: &[String], root: PathBuf) -> Result<Invocation, String>
{
    let rule = Required_Value(Named_Value_From_String_Arguments(rest, "--rule").as_ref(), Name("--rule"), Usage(USAGE))?;
    let location = Required_Value(Named_Value_From_String_Arguments(rest, "--location").as_ref(), Name("--location"), Usage(USAGE))?;

    let command = GateCommand { root, ..GateCommand::default() };
    let query = FindingQuery { rule: RuleId::New(rule), location };

    return Ok(Invocation::Explain { command, query });
}

/// `compare`'s own required `--against`, naming the tree judged against `--root`.
///
/// Both sides are built from the same `rest`, so every selector flag narrows both alike.
/// That is what makes a difference attributable to the roots: a compare whose two sides
/// were narrowed differently would report findings that moved because a selector moved,
/// and read as though the tree had.
fn Compare_Invocation(rest: &[String], root: PathBuf) -> Result<Invocation, String>
{
    let against = Required_Value(
        Named_Value_From_String_Arguments(rest, "--against").as_ref(),
        Name("--against"),
        Usage(USAGE),
    )?;

    return Ok(Invocation::Compare {
        baseline: Plan_Or_Run_Command(root, rest),
        candidate: Plan_Or_Run_Command(PathBuf::from(against), rest),
    });
}

/// `admits`' own required `--from` and `--to`, naming the two crates.
///
/// It takes no root and is handed none. Every other verb walks a tree; this one asks about
/// two names against the declared architecture, which is what makes it cheap enough to ask
/// before the edge exists — `OD-GATE-026` measured the two answers at 40 milliseconds against
/// thirteen seconds. Accepting a `--root` here would advertise a narrowing that changes
/// nothing about the answer.
fn Admits_Invocation(rest: &[String]) -> Result<Invocation, String>
{
    let depending = Required_Value(Named_Value_From_String_Arguments(rest, "--from").as_ref(), Name("--from"), Usage(USAGE))?;
    let depended = Required_Value(Named_Value_From_String_Arguments(rest, "--to").as_ref(), Name("--to"), Usage(USAGE))?;

    return Ok(Invocation::Admits { depending, depended });
}

/// Builds `plan`/`run`'s shared command from `rest`'s flags, now that the verb and its
/// argument shape are already known.
fn Plan_Or_Run_Command(root: PathBuf, rest: &[String]) -> GateCommand
{
    return GateCommand {
        root,
        scope: ScopeSelector {
            include: Named_Values_From_String_Arguments(rest, "--include"),
            exclude: Named_Values_From_String_Arguments(rest, "--exclude"),
        },
        rules: RuleSelector { include: Named_Values_From_String_Arguments(rest, "--rule").into_iter().map(RuleId::New).collect() },
        // No flag authors a Suppression, a BaselineDebt or a RuleCalibration yet -- see
        // `nomos_gate_orchestration::SuppressionPolicy`'s own doc for why inventing one now
        // would be premature.
        suppressions: nomos_gate_orchestration::SuppressionPolicy::default(),
        baseline: nomos_gate_orchestration::BaselinePolicy::default(),
        adoption: nomos_gate_orchestration::AdoptionPolicy::default(),
        // No flag authors a non-default CoveragePolicy yet -- see
        // `nomos_gate_orchestration::CoveragePolicy`'s own doc for why.
        coverage: nomos_gate_orchestration::CoveragePolicy::default(),
        // No flag authors a ModelExecutionProfile yet -- see
        // `nomos_gate_orchestration::GateCommand::model`'s own doc for why.
        model: None,
        // No flag authors a GatePhase or a PhaseApproval yet -- see
        // `nomos_gate_orchestration::GateCommand::phases`'s own doc for why.
        phases: Vec::new(),
        approvals: Vec::new(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Default_To_Plan_With_The_Current_Directory()
    {
        let arguments = vec!["plan".to_owned()];

        let invocation = Gate_Invocation_From_String_Arguments(&arguments)
            .expect("`plan` is one of the gate verbs, and a bare verb carries no flag of its own");

        let Invocation::Plan(command) = invocation
        else
        {
            // This test's only argument is "plan", and Gate_Invocation_From_String_Arguments
            // dispatches that verb to Invocation::Plan unconditionally -- reaching else here
            // means that dispatch itself broke, not a condition this test should assert
            // around.
            panic!("expected Plan, got {invocation:?}");
        };
        assert_eq!(command.root, PathBuf::from("."));
    }

    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Parse_A_Run_Verb()
    {
        let arguments = vec!["run".to_owned(), "--root".to_owned(), "some/tree".to_owned()];

        let invocation = Gate_Invocation_From_String_Arguments(&arguments)
            .expect("`run` is one of the gate verbs, and `--root` is the flag it reads");

        assert!(matches!(invocation, Invocation::Run(_)), "{invocation:?}");
    }

    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Refuse_An_Unknown_Verb()
    {
        let arguments = vec!["diff".to_owned()];

        let error = Gate_Invocation_From_String_Arguments(&arguments).expect_err("diff is not a verb");

        assert!(error.contains("unknown verb"), "{error}");
    }

    /// `compare` without `--against` names one tree and asks for a difference, which is not
    /// a question. Refusing is what keeps it from quietly comparing a tree with itself.
    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Refuse_Compare_Without_A_Second_Tree()
    {
        let arguments = vec!["compare".to_owned(), "--root".to_owned(), "some/tree".to_owned()];

        let error = Gate_Invocation_From_String_Arguments(&arguments).expect_err("compare needs two trees");

        assert!(error.contains("--against"), "{error}");
    }

    /// Both sides carry the roots they were given, and the selector flags reach both --
    /// which is what makes the reported difference attributable to the trees.
    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Parse_Compare_With_Both_Trees()
    {
        let arguments = vec![
            "compare".to_owned(),
            "--root".to_owned(),
            "before".to_owned(),
            "--against".to_owned(),
            "after".to_owned(),
            "--include".to_owned(),
            "src".to_owned(),
        ];

        let invocation = Gate_Invocation_From_String_Arguments(&arguments)
            .expect("`compare` needs `--against` beside `--root`, and the list carries both");

        let Invocation::Compare { baseline, candidate } = invocation
        else
        {
            panic!("expected Compare, got {invocation:?}");
        };
        assert_eq!(baseline.root, PathBuf::from("before"));
        assert_eq!(candidate.root, PathBuf::from("after"));
        assert_eq!(baseline.scope.include, ["src".to_owned()]);
        assert_eq!(candidate.scope.include, ["src".to_owned()], "a selector must narrow both sides alike");
    }

    /// `steps` is a verb of its own, carrying a root and a host and no `GateCommand`.
    #[test]
    fn Test_Steps_Should_Parse_With_A_Root_And_A_Host()
    {
        let arguments = vec![
            "steps".to_owned(),
            "--root".to_owned(),
            "somewhere".to_owned(),
            "--host".to_owned(),
            "windows-latest".to_owned(),
        ];

        let invocation = Gate_Invocation_From_String_Arguments(&arguments).expect("steps is a verb");

        let Invocation::Steps { root, host } = invocation
        else
        {
            panic!("steps must parse as Steps, got {invocation:?}");
        };
        assert_eq!(root, PathBuf::from("somewhere"));
        assert_eq!(host, "windows-latest");
    }

    /// With no `--host`, the label is this machine's own leg rather than a fixed guess, so a
    /// run on either supported host answers for that host.
    #[test]
    fn Test_Steps_Should_Default_The_Host_To_This_Machines_Own_Leg()
    {
        let invocation =
            Gate_Invocation_From_String_Arguments(&["steps".to_owned()]).expect("steps with no flags is valid");

        let Invocation::Steps { host, .. } = invocation
        else
        {
            panic!("steps must parse as Steps");
        };

        let expected = match std::env::consts::OS
        {
            "linux" => "ubuntu-latest",
            "windows" => "windows-latest",
            other => other,
        };
        assert_eq!(host, expected);
    }

    /// The refusal for an unknown verb keeps naming the whole set, so adding a verb without
    /// adding it to the usage text would show up here.
    #[test]
    fn Test_An_Unknown_Verb_Should_Still_Name_Every_Verb_Including_Steps()
    {
        let refusal =
            Gate_Invocation_From_String_Arguments(&["bogus".to_owned()]).expect_err("bogus is not a verb");

        for verb in ["plan", "run", "explain", "compare", "admits", "steps"]
        {
            assert!(refusal.contains(verb), "the refusal must name `{verb}`: {refusal}");
        }
        // Spelled with its invocation, because "policy" alone appears in this usage text for
        // three other reasons and a bare `contains` would pass with the verb missing.
        assert!(refusal.contains("nomos gate policy"), "the refusal must name the policy verb: {refusal}");
    }

    /// `policy` is a verb of its own, carrying the same command `run` does -- the command
    /// line is one of the layers the resolution ranks, so a policy report over a bare root
    /// would report a resolution no run performs.
    #[test]
    fn Test_Gate_Invocation_From_String_Arguments_Should_Parse_A_Policy_Verb()
    {
        let arguments = vec!["policy".to_owned(), "--root".to_owned(), "some/tree".to_owned()];

        let invocation = Gate_Invocation_From_String_Arguments(&arguments)
            .expect("`policy` is one of the gate verbs, and `--root` is the flag it reads");

        let Invocation::Policy(command) = invocation
        else
        {
            panic!("expected Policy, got {invocation:?}");
        };
        assert_eq!(command.root, PathBuf::from("some/tree"));
    }
}
