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
