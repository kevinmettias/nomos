//! What the gate checks, derived from the gate rather than retyped beside it.
//!
//! Every item's `verification` argv is a `cargo test` invocation. The gate is more than
//! that: it lints first, with `-D warnings`, over a workspace that denies `unwrap_used`,
//! `indexing_slicing` and `arithmetic_side_effects` — denials, because a panic on the
//! analysis path is a determinism defect rather than a bug. So an item could be recorded
//! verified while the gate that follows it was already red, and twice it was.
//!
//! The fix is not to write the lint command into this crate. A copy of it here is a
//! second source of truth that goes stale the day the workflow changes, and two guards
//! for one rule is how they come to disagree. This module reads the workflow.

// What running the gate produced.
#[path = "gate/gate_outcome.rs"]
mod outcome;

pub use outcome::GateOutcome;

// The workflow's whole text, and a step's name within it — kept apart purely by type, and
// each large enough on its own to want a file of its own.
#[path = "gate/workflow_text.rs"]
mod workflow_text;
#[path = "gate/step_name.rs"]
mod step_name;

pub use workflow_text::WorkflowText;
pub use step_name::StepName;

// The whole step set, and what a local execution of it produced. Each is a public type of
// its own, so each has the file the naming rule wants.
#[path = "gate/step_guard.rs"]
mod step_guard;
#[path = "gate/derived_step.rs"]
mod derived_step;
#[path = "gate/step_execution.rs"]
mod step_execution;
#[path = "gate/local_gate_run.rs"]
mod local_gate_run;

pub use step_guard::StepGuard;
pub use derived_step::DerivedStep;
pub use step_execution::StepExecution;
pub use local_gate_run::{LocalGateRun, Run_Gate_Locally};

// The whole step set, read in one pass. Its own file because this one had grown past the
// size the standards ask a submodule to be extracted at.
#[path = "gate/step_derivation.rs"]
mod step_derivation;

pub use step_derivation::Derive_Steps;

use std::path::Path;

/// Where the gate is defined, relative to the tree the predicate runs in.
pub const GATE_WORKFLOW: &str = ".github/workflows/gate.yml";

/// The step every item's predicate was missing.
///
/// Only the lint step is derived. The gate's test step stays authored per item, because
/// a scoped test is what a per-item predicate is *for* — running the whole workspace on
/// every finish costs minutes, and a finish that costs minutes is one that gets skipped.
/// That remaining gap is deliberate and is recorded, not hidden.
pub const LINT_STEP: &str = "Lint";

/// Characters that mean the `run:` line is a shell script rather than one command.
///
/// A predicate is an argv, never a command string: the prototype split strings with
/// bespoke quote handling and put a parser between what an author wrote and what ran.
/// This module does not reintroduce that parser. It splits on whitespace, which is exact
/// for a bare command, and refuses anything where whitespace is not the whole story.
///
/// The last two are spelled as escapes rather than as themselves. A char literal holding a
/// quote defeats the scanner that blanks strings before the brace checks read columns, and
/// every block after this line then reports its braces as misaligned. The escape is the same
/// character and costs nothing but the false report.
const SHELL_METACHARACTERS: [char; 9] = ['|', '&', ';', '<', '>', '$', '`', '\u{22}', '\u{27}'];

/// Why the gate could not be derived.
///
/// Never "so run the item's predicate on its own". An underived gate is an unanswered
/// question about what finishing means, and this crate already has the rule for that one
/// level up: unknown independence is not safe parallelism, so the claim is refused rather
/// than granted. Unknown gate coverage is not gate coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateUnknown
{
    /// The workflow could not be read.
    Unreadable
    {
        /// Where it was looked for.
        path: String,
        /// Why not.
        cause: String,
    },
    /// The workflow has no step by that name.
    NoSuchStep
    {
        /// The step that was wanted.
        step: String,
    },
    /// The step's `run:` is a script, so no argv can be derived from it without guessing.
    NotASingleCommand
    {
        /// The step.
        step: String,
        /// What it runs.
        run: String,
    },
    /// The step's `if:` uses a form this reader does not implement, so which legs run it
    /// cannot be decided without guessing.
    ///
    /// `OD-GATE-033` names this half in as many words: `GateUnknown` already carried the
    /// three causes for the `run:` half, and the guard half needs the same treatment.
    GuardOutsideSubset
    {
        /// The step.
        step: String,
        /// The `if:` text, verbatim, so a refusal can quote what it did not understand.
        guard: String,
    },
    /// The step was admitted and its argv derived, but it never exited on its own, so this
    /// execution is not evidence about the step's subject.
    ///
    /// Distinct from an exit code for the reason `ProgramLauncher` already separates the
    /// two: "we could not ask" is not "we asked and the answer was no".
    DidNotRun
    {
        /// The step.
        step: String,
        /// What became of it instead of an exit.
        cause: String,
    },
}

impl GateUnknown
{
    /// A one-line explanation a person or an agent can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Unreadable { path, cause } => {
                format!("{path} could not be read ({cause}), so what the gate checks is unknown")
            }
            Self::NoSuchStep { step } => format!(
                "the gate workflow has no step named `{step}`, so the step an item's \
                 predicate is missing cannot be derived"
            ),
            Self::NotASingleCommand { step, run } => format!(
                "the gate's `{step}` step runs `{run}`, which is a script rather than one \
                 command. Deriving an argv from it would mean guessing, and a guessed \
                 predicate is the defect this module exists to close"
            ),
            Self::GuardOutsideSubset { step, guard } => format!(
                "the gate's `{step}` step is guarded by `{guard}`, which this reader does not implement. Which legs run it is therefore unknown, and an executor that guessed would run a step on the wrong host or drop it silently"
            ),
            Self::DidNotRun { step, cause } => format!(
                "the gate's `{step}` step was admitted here but did not exit on its own ({cause}), so this execution says nothing about what it would have found"
            ),
        };
    }
}

/// The path of the gate workflow within a tree.
#[must_use]
pub fn Workflow_Path(tree: &Path) -> std::path::PathBuf
{
    return tree.join(GATE_WORKFLOW);
}

/// The argv of a named step in a GitHub Actions workflow.
///
/// Deliberately not a YAML parser. It reads the two line shapes a step is written in and
/// refuses everything else, which is the behaviour that keeps a wrong answer from looking
/// like a right one.
///
/// # Errors
///
/// Returns a [`GateUnknown`] naming why no argv could be derived.
pub fn Derive_Step<'a>(workflow: impl Into<WorkflowText<'a>>, step: impl Into<StepName<'a>>) -> Result<Vec<String>, GateUnknown>
{
    let workflow = workflow.into();
    let step = step.into();

    let Some(run) = Find_Run(workflow.As_Text(), step.As_Text())
    else
    {
        return Err(GateUnknown::NoSuchStep {
            step: step.As_Text().to_owned(),
        });
    };

    return Argv_Of(run, step);
}


/// The trimmed argument of the named step's `run:` line, or `None` if the workflow never
/// declares that step, or declares it with no `run:` line.
fn Find_Run<'a>(workflow: &'a str, step: &str) -> Option<&'a str>
{
    let mut inside = false;

    for line in workflow.lines()
    {
        let trimmed = line.trim();

        if let Some(name) = Step_Named(trimmed)
        {
            inside = name == step;
            continue;
        }

        if inside
            && let Some(run) = trimmed.strip_prefix("run:")
        {
            return Some(run.trim());
        }
    }

    return None;
}

/// The step name a line declares, in either of the two spellings a step is written in.
///
/// `- name:` opens a step and `name:` continues the same mapping, and both mean the reader
/// has moved to a different step. Reading only one of them would leave the reader inside
/// the previous step and take that step's `run:` as this one's.
fn Step_Named(trimmed: &str) -> Option<&str>
{
    let named = trimmed
        .strip_prefix("- name:")
        .or_else(|| return trimmed.strip_prefix("name:"))?;

    return Some(named.trim());
}

/// Splits a bare command into an argv, or refuses.
///
/// `step` is [`StepName`] rather than `&str`: it travels here as the second of two adjacent
/// string-shaped parameters, and giving it the same type [`Derive_Step`] already gave it
/// keeps the two positions from becoming transposable again one call down.
fn Argv_Of(run: &str, step: StepName<'_>) -> Result<Vec<String>, GateUnknown>
{
    if run.is_empty() || run.contains(SHELL_METACHARACTERS)
    {
        return Err(GateUnknown::NotASingleCommand {
            step: step.As_Text().to_owned(),
            run: run.to_owned(),
        });
    }

    return Ok(run.split_whitespace().map(str::to_owned).collect());
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The shortest a refusal's `Describe()` may be and still tell an operator anything.
    const MINIMUM_USEFUL_DESCRIPTION_LENGTH: usize = 15;

    const WORKFLOW: &str = "name: gate\n\
                            \n\
                            jobs:\n\
                            \x20 gate:\n\
                            \x20   steps:\n\
                            \x20     - uses: actions/checkout@v4\n\
                            \x20     - name: Lint\n\
                            \x20       run: cargo clippy --workspace --all-targets -- -D warnings\n\
                            \x20     - name: Test\n\
                            \x20       run: cargo test --workspace\n";

    #[test]
    fn Test_Derive_Step_Should_Return_The_Lint_Steps_Argv()
    {
        let argv = Derive_Step(WORKFLOW, LINT_STEP).expect("the workflow has a Lint step");

        assert_eq!(
            argv,
            vec![
                "cargo".to_owned(),
                "clippy".to_owned(),
                "--workspace".to_owned(),
                "--all-targets".to_owned(),
                "--".to_owned(),
                "-D".to_owned(),
                "warnings".to_owned(),
            ]
        );
    }

    /// The test that fails if somebody later writes the clippy line into this crate as a
    /// constant. A derived step tracks the workflow; a copied one disagrees with it.
    #[test]
    fn Test_Changing_The_Workflow_Should_Change_The_Derived_Step()
    {
        let altered = WORKFLOW.replace("--all-targets", "--lib");

        let argv = Derive_Step(&altered, LINT_STEP).expect("the altered workflow still lints");

        assert!(argv.contains(&"--lib".to_owned()));
        assert!(!argv.contains(&"--all-targets".to_owned()));
    }

    #[test]
    fn Test_Each_Step_Should_Get_Its_Own_Command()
    {
        assert_eq!(
            Derive_Step(WORKFLOW, "Test").expect("the workflow has a Test step"),
            vec![
                "cargo".to_owned(),
                "test".to_owned(),
                "--workspace".to_owned()
            ]
        );
    }

    #[test]
    fn Test_A_Workflow_Name_Should_Not_Be_Read_As_A_Step()
    {
        for name in Workflow_Level_Names()
        {
            let refusal = Derive_Step(WORKFLOW, name).expect_err("gate is the workflow, not a step");

            assert!(matches!(refusal, GateUnknown::NoSuchStep { .. }));
        }
    }

    /// The top-level workflow name, for [`Test_A_Workflow_Name_Should_Not_Be_Read_As_A_Step`]:
    /// it must not be mistaken for a step, or the first `run:` in the file gets attributed to
    /// a step that has none.
    fn Workflow_Level_Names() -> [&'static str; 1]
    {
        return ["gate"];
    }

    #[test]
    fn Test_A_Missing_Step_Should_Be_Refused()
    {
        for step in Names_Of_Steps_The_Workflow_Never_Declares()
        {
            let refusal = Derive_Step(WORKFLOW, step).expect_err("there is no such step here");

            assert!(matches!(refusal, GateUnknown::NoSuchStep { .. }));
        }
    }

    /// Step names the fixture workflow never declares, for
    /// [`Test_A_Missing_Step_Should_Be_Refused`].
    fn Names_Of_Steps_The_Workflow_Never_Declares() -> [&'static str; 1]
    {
        return ["Boundaries"];
    }

    /// A shell script cannot be turned into an argv without guessing, and a guessed
    /// predicate is worse than a refused one because it looks like it worked.
    #[test]
    fn Test_A_Scripted_Step_Should_Be_Refused_Rather_Than_Guessed()
    {
        for &run in Scripted_Step_Run_Lines()
        {
            let workflow = format!("      - name: Lint\n        run: {run}\n");

            let refusal = Derive_Step(&workflow, LINT_STEP)
                .expect_err("a scripted step must not yield an argv");

            assert!(
                matches!(refusal, GateUnknown::NotASingleCommand { .. }),
                "`{run}` was split rather than refused"
            );
        }
    }

    /// `run:` lines that are scripts rather than a single command, for
    /// [`Test_A_Scripted_Step_Should_Be_Refused_Rather_Than_Guessed`].
    fn Scripted_Step_Run_Lines() -> &'static [&'static str]
    {
        return &[
            "cargo clippy && cargo test",
            "cargo test | tee log",
            "cargo test --features \"a b\"",
            "|",
        ];
    }

    #[test]
    fn Test_Every_Refusal_Should_Describe_Itself_Usefully()
    {
        let refusals = [
            GateUnknown::Unreadable {
                path: ".github/workflows/gate.yml".to_owned(),
                cause: "not found".to_owned(),
            },
            GateUnknown::NoSuchStep {
                step: "Lint".to_owned(),
            },
            GateUnknown::NotASingleCommand {
                step: "Lint".to_owned(),
                run: "a && b".to_owned(),
            },
        ];

        for refusal in &refusals
        {
            assert!(
                refusal.Describe().len() > MINIMUM_USEFUL_DESCRIPTION_LENGTH,
                "{} is too terse to act on",
                refusal.Describe()
            );
        }
    }

    #[test]
    fn Test_The_Workflow_Path_Should_Sit_Under_The_Tree()
    {
        let path = Workflow_Path(Path::new("some/tree"));

        assert!(path.ends_with("gate.yml"));
        assert!(path.starts_with("some/tree"));
    }
}
