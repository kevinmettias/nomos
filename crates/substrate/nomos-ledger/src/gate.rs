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
mod outcome;

pub use outcome::GateOutcome;

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
const SHELL_METACHARACTERS: [char; 9] = ['|', '&', ';', '<', '>', '$', '`', '"', '\''];

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
        };
    }
}

/// The path of the gate workflow within a tree.
#[must_use]
pub fn Workflow_Path(tree: &Path) -> std::path::PathBuf
{
    return tree.join(GATE_WORKFLOW);
}

/// The text of a whole GitHub Actions workflow file, distinguished from [`StepName`] purely
/// by type.
///
/// [`Derive_Step`] takes one of each as adjacent parameters, and two parameters that both
/// read as `&str` there let a caller swap the workflow for the step name and have the
/// compiler accept it. `From<&str>` and `From<&String>` both convert into this, so no
/// existing call site needs to change shape to adopt it — every one already passes a
/// borrowed string.
#[derive(Clone, Copy, Debug)]
pub struct WorkflowText<'a>(&'a str);

impl<'a> From<&'a str> for WorkflowText<'a>
{
    fn from(value: &'a str) -> Self
    {
        return WorkflowText(value);
    }
}

impl<'a> From<&'a String> for WorkflowText<'a>
{
    fn from(value: &'a String) -> Self
    {
        return WorkflowText(value.as_str());
    }
}

impl<'a> WorkflowText<'a>
{
    /// The workflow's text as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}

/// The name of one step within a workflow, distinguished from [`WorkflowText`] for the
/// reason given on that type.
#[derive(Clone, Copy, Debug)]
pub struct StepName<'a>(&'a str);

impl<'a> From<&'a str> for StepName<'a>
{
    fn from(value: &'a str) -> Self
    {
        return StepName(value);
    }
}

impl<'a> From<&'a String> for StepName<'a>
{
    fn from(value: &'a String) -> Self
    {
        return StepName(value.as_str());
    }
}

impl<'a> StepName<'a>
{
    /// The step name as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
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

    let Some(run) = Find_Run(workflow.As_Str(), step.As_Str())
    else
    {
        return Err(GateUnknown::NoSuchStep {
            step: step.As_Str().to_owned(),
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
            step: step.As_Str().to_owned(),
            run: run.to_owned(),
        });
    }

    return Ok(run.split_whitespace().map(str::to_owned).collect());
}

#[cfg(test)]
mod tests
{
    use super::*;

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
    fn Test_The_Lint_Step_Should_Be_Derived_From_The_Workflow()
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

    /// The top-level `name: gate` must not be mistaken for a step, or the first `run:`
    /// in the file gets attributed to a step that has none.
    #[test]
    fn Test_A_Workflow_Name_Should_Not_Be_Read_As_A_Step()
    {
        let refusal = Derive_Step(WORKFLOW, "gate").expect_err("gate is the workflow, not a step");

        assert!(matches!(refusal, GateUnknown::NoSuchStep { .. }));
    }

    #[test]
    fn Test_A_Missing_Step_Should_Be_Refused()
    {
        let refusal =
            Derive_Step(WORKFLOW, "Boundaries").expect_err("there is no Boundaries step here");

        assert!(matches!(refusal, GateUnknown::NoSuchStep { .. }));
    }

    /// A shell script cannot be turned into an argv without guessing, and a guessed
    /// predicate is worse than a refused one because it looks like it worked.
    #[test]
    fn Test_A_Scripted_Step_Should_Be_Refused_Rather_Than_Guessed()
    {
        for run in [
            "cargo clippy && cargo test",
            "cargo test | tee log",
            "cargo test --features \"a b\"",
            "|",
        ]
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
                refusal.Describe().len() > 15,
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
