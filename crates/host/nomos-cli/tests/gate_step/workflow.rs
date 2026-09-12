//! The real workflow, the part of it that runs, and the doctored copies the controls need.

use std::path::{Path, PathBuf};

/// The step `OD-GATE-004` added. Deliberately not `Lint`, and
/// `Test_The_Derived_Lint_Step_Should_Still_Be_Clippy` is why.
pub(crate) const RULES_STEP: &str = "Rules";

/// The three ways a step is told to run and not be believed.
///
/// `continue-on-error:` is the platform's own; `|| true` and a trailing `exit 0` are the
/// shell's. Any of them turns this into "a step that runs the command and ignores its exit
/// code", which `P10-CHECK-GATE`'s `done_when` names as not closing the item.
const EXCUSES: [&str; 3] = ["continue-on-error", "|| true", "exit 0"];

/// How long a git commit identifier is, in hexadecimal characters.
///
/// The full identifier and not a prefix. An abbreviated one is ambiguous by construction and
/// GitHub Actions refuses to resolve it, so a short "pin" is a broken reference rather than a
/// weak one.
const COMMIT_IDENTIFIER_LENGTH: usize = 40;

/// This repository's root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// The real gate workflow, read from disk through the ledger's own path function.
///
/// `Workflow_Path` rather than a string joined here, so that if `work finish` ever starts
/// looking somewhere else these assertions follow it instead of quietly checking a file
/// nothing reads.
pub(crate) fn Workflow() -> String
{
    let path = nomos_ledger::Workflow_Path(&Repository_Root());

    return std::fs::read_to_string(&path)
        // Every assertion in this suite is a claim about this one file, so an empty string
        // handed back on failure would make all of them true of nothing — the suite would go
        // green in exactly the situation where the gate had stopped existing. The path is
        // printed because the interesting failure is `Workflow_Path` having moved rather than
        // the file being unreadable, and only the resolved path tells those apart.
        .unwrap_or_else(|error| panic!("this repository has a gate workflow at {}: {error}", path.display()));
}

/// A workflow with its comment lines dropped, which is the part that runs.
///
/// Extracted rather than written twice. Every structural assertion in this suite is about what
/// the workflow *does*, and the steps here explain themselves at length in prose that names
/// the very strings being guarded against — `continue-on-error`, `@v4`, `contents: read` all
/// appear in comments arguing for or against them. A predicate that read the prose would make
/// explaining a decision impossible in the file the decision lives in, and a second copy of
/// this rule beside the first is how two guards come to disagree about what a comment is.
pub(crate) fn Executable_Part(workflow: &str) -> String
{
    return workflow
        .lines()
        .filter(|line| return !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");
}

/// Which excuses a workflow carries, comments not counted.
///
/// Extracted so that the control in [`super::excuses`] exercises **this** predicate rather
/// than a second implementation of it written beside the first — the arrangement
/// `governing_records_are_present.rs` uses for `Disagreements`.
///
/// Comment lines are dropped because the predicate is about what the workflow *does*. The
/// `Rules` step's own comment explains why it carries no branch and no excuse, and a guard
/// that fired on prose would make explaining a decision impossible in the file the decision
/// is in.
pub(crate) fn Excuses_In(workflow: &str) -> Vec<String>
{
    let executable = Executable_Part(workflow);

    return EXCUSES
        .iter()
        .filter(|excuse| return executable.contains(**excuse))
        .map(|excuse| return (*excuse).to_owned())
        .collect();
}

/// The action reference a `uses:` line names, with any trailing comment removed.
///
/// The comment is where the pin says which release it is, so it is deliberately not part of
/// the reference being judged — a pin is the SHA, and the note beside it is for the reader.
fn Action_Reference(line: &str) -> Option<&str>
{
    let reference = line
        .strip_prefix("- uses:")
        .or_else(|| return line.strip_prefix("uses:"))?;

    return Some(reference.split('#').next().unwrap_or(reference).trim());
}

/// Whether a reference is pinned to a full commit identifier.
fn Is_A_Commit_Pin(version: &str) -> bool
{
    return version.len() == COMMIT_IDENTIFIER_LENGTH
        && version
            .chars()
            .all(|character| return character.is_ascii_hexdigit());
}

/// Which actions this workflow reaches for by something other than a commit, comments not
/// counted.
///
/// A reference carrying no `@` at all is reported too. That is not this repository's case
/// today, and it is included because the shape it would take — a local composite action — is
/// still a decision about what executes here, and reporting it is what keeps this from being
/// a check that only understands the one line it was written for.
pub(crate) fn Unpinned_Actions_In(workflow: &str) -> Vec<String>
{
    return Executable_Part(workflow)
        .lines()
        .map(str::trim)
        .filter_map(Action_Reference)
        .filter(|reference| {
            return !reference
                .rsplit_once('@')
                .is_some_and(|(_, version)| return Is_A_Commit_Pin(version));
        })
        .map(str::to_owned)
        .collect();
}

/// The same workflow with every action reached for by a moving tag instead of a commit.
///
/// Written by walking the `uses:` lines rather than replacing the SHA that is there today, so
/// it keeps working the next time the pin is advanced and cannot silently rewrite nothing —
/// the arrangement [`With_A_Scripted_Rules_Step`] uses for its own subject.
pub(crate) fn With_A_Tagged_Action(workflow: &str) -> String
{
    let mut rewritten = String::new();

    for line in workflow.lines()
    {
        let trimmed = line.trim();

        if !trimmed.starts_with('#')
            && let Some(reference) = Action_Reference(trimmed)
            && let Some((action, _)) = reference.rsplit_once('@')
        {
            rewritten.push_str("      - uses: ");
            rewritten.push_str(action);
            rewritten.push_str("@v4\n");
            continue;
        }

        rewritten.push_str(line);
        rewritten.push('\n');
    }

    return rewritten;
}

/// The same workflow with the `Rules` step's `run:` turned into a block scalar.
///
/// Written by walking the step rather than by replacing a literal command, so it keeps
/// working when the argv changes and cannot silently rewrite nothing.
pub(crate) fn With_A_Scripted_Rules_Step(workflow: &str) -> String
{
    let mut rewritten = String::new();
    let mut inside = false;
    for line in workflow.lines()
    {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("- name:")
        {
            inside = name.trim() == RULES_STEP;
        }
        else if inside && let Some(run) = trimmed.strip_prefix("run:")
        {
            Rewrite_As_A_Block(&mut rewritten, run);
            inside = false;
            continue;
        }

        rewritten.push_str(line);
        rewritten.push('\n');
    }

    return rewritten;
}

/// The same command, written as a YAML block scalar so that the deriver must read past the
/// `|` rather than off the line itself.
fn Rewrite_As_A_Block(rewritten: &mut String, run: &str)
{
    rewritten.push_str("        run: |\n          ");
    rewritten.push_str(run.trim());
    rewritten.push('\n');
}

/// The same workflow with the `Rules` step excused from failing.
pub(crate) fn With_An_Excused_Rules_Step(workflow: &str) -> String
{
    return workflow.replace(
        "      - name: Rules\n",
        "      - name: Rules\n        continue-on-error: true\n",
    );
}

/// One step of the workflow: the name it is listed under, and everything it runs.
///
/// `commands` is every line of the step after its own `- name:` and before the next one,
/// which is what makes a block scalar (`run: |`) read the same as a one-line `run:`. The
/// alternative -- reading the `run:` line itself -- is the shape
/// `Test_A_Scripted_Rules_Step_Should_Not_Satisfy_That_Assertion` already exists to refuse.
pub(crate) struct Step
{
    pub(crate) name: String,
    pub(crate) commands: String,
}

/// Every step this workflow runs, in the order it runs them, comments not counted.
pub(crate) fn Steps_In(workflow: &str) -> Vec<Step>
{
    let mut steps: Vec<Step> = Vec::new();

    for line in Executable_Part(workflow).lines()
    {
        let trimmed = line.trim();

        if let Some(name) = trimmed.strip_prefix("- name:")
        {
            steps.push(Step { name: name.trim().to_owned(), commands: String::new() });
            continue;
        }

        if let Some(current) = steps.last_mut()
        {
            current.commands.push_str(trimmed);
            current.commands.push('\n');
        }
    }

    return steps;
}

/// The marker that a step puts a tool on the runner.
const TOOL_INSTALL: &str = "cargo install";

/// The marker that a step runs this workspace's own tests.
const TEST_RUN: &str = "cargo test";

/// The position of every step that installs a tool, and of the first that runs the tests.
///
/// Positions rather than names, because the question is an ordering and a name cannot be
/// compared. Both halves are derived from the same [`Steps_In`] walk, so a step renamed or
/// rewritten moves both together instead of one silently matching nothing.
pub(crate) fn Installs_After_The_First_Test_Run(workflow: &str) -> Vec<String>
{
    let steps = Steps_In(workflow);

    let Some(first_test_run) = steps.iter().position(|step| return step.commands.contains(TEST_RUN))
    else
    {
        // No step runs the tests at all. Reported as a violation rather than as nothing to
        // check: this assertion is about a tool being present before the tests need it, and
        // a workflow with no test step has lost the subject rather than satisfied the rule.
        return vec![format!("no step runs `{TEST_RUN}`, so this workflow has lost the subject of the ordering")];
    };

    return steps
        .iter()
        .enumerate()
        .filter(|(position, step)| return *position > first_test_run && step.commands.contains(TOOL_INSTALL))
        .map(|(_, step)| return step.name.clone())
        .collect();
}

/// The same workflow with every tool install moved after every other step.
///
/// Built by partitioning the steps this file's own [`Steps_In`] derives, rather than by
/// moving the one step that is there today, so it keeps working when a second tool is
/// installed and cannot silently rewrite nothing -- the arrangement [`With_A_Tagged_Action`]
/// uses for its own subject.
///
/// The result is a step list rather than a runnable workflow: the `jobs:` preamble is not
/// reproduced, because the only thing read back out of it is the step order, and writing a
/// second copy of the workflow's own header here would be a fixture that could drift from
/// the file it stands in for.
pub(crate) fn With_Tool_Installs_After_The_Tests(workflow: &str) -> String
{
    let steps = Steps_In(workflow);
    let (installs, others): (Vec<&Step>, Vec<&Step>) =
        steps.iter().partition(|step| return step.commands.contains(TOOL_INSTALL));

    let mut rewritten = String::new();

    for step in others.into_iter().chain(installs)
    {
        rewritten.push_str("      - name: ");
        rewritten.push_str(&step.name);
        rewritten.push('\n');

        for command in step.commands.lines()
        {
            rewritten.push_str("        ");
            rewritten.push_str(command);
            rewritten.push('\n');
        }
    }

    return rewritten;
}
