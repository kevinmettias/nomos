//! `nomos check` — run the rules over a tree and report what they find.
//!
//! The composition root for `nomos-rules`. That crate is a pure function from source
//! text to findings and deliberately cannot read a disk; this module is where the tree
//! is decided, walked and handed over, which is the seam that lets the same rule be run
//! against the real workspace here and against code that no longer exists in a test.
//!
//! # The vacuity guard lives here
//!
//! A rule that finds no subjects returns no findings, which renders identically to a
//! clean run. That is the defect the sibling workspace recorded four times over —
//! `check-standards-tree /nonexistent` walked nothing, found nothing and reported CLEAN
//! — and `boundaries.rs` already guards its own assertions against it. The guard belongs
//! here rather than in the rule, because "did I see a plausible amount of the world" is a
//! question only the caller that chose the tree can answer. An empty walk exits
//! [`ExitCode::Vacuous`] and reports nothing clean.

use crate::arguments::Named_Value;
use nomos_contracts::Finding;
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `3` and `4` are `work`'s claim
/// codes and are not reused here, and `5` and `6` carry the meanings `spec` gave them —
/// "could not be read at all" and "the answer is empty because something expected was
/// not there".
///
/// [`ExitCode::Vacuous`] is the one that earns its own code rather than folding into
/// `Ok`. A run that judged nothing and a run that judged everything and approved are the
/// two states this binary must never render the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The rules ran and nothing they found can fail a build.
    Ok = 0,
    /// At least one finding can fail a build.
    Violations = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The tree could not be read at all.
    Unreadable = 5,
    /// The walk found no source to judge, so a clean result would mean nothing.
    Vacuous = 6,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What to check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckCommand
{
    /// The tree to judge.
    pub root: PathBuf,
}

const USAGE: &str = "usage: nomos check [--root <path>]\n\n\
     Runs every rule over the tree and reports what they find.\n\n\
     rules:\n  \
     completeness-mirror   a declared universe must name a check that compares it\n\
     \x20                     against the reality it enumerates, and that check must\n\
     \x20                     exist. See OD-COMPLETENESS-001.\n\n\
     exit codes: 0 nothing blocking, 1 findings that can fail a build, 2 usage,\n\
     \x20           5 unreadable tree, 6 nothing was judged";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when an argument is not understood.
pub fn Parse(arguments: &[String]) -> Result<CheckCommand, String>
{
    if let Some(unknown) = arguments
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(CheckCommand {
        root: Named_Value(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    });
}

/// Runs the rules and renders what they say.
pub fn Run(
    command: &CheckCommand,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> ExitCode
{
    if !command.root.is_dir()
    {
        let _ignored = writeln!(
            stderr,
            "cannot read `{}`: not a directory",
            command.root.display()
        );
        return ExitCode::Unreadable;
    }

    let sources = Read_Sources(&command.root);

    if sources.is_empty()
    {
        let _ignored = writeln!(
            stderr,
            "no Rust source found under `{}`, so nothing was judged.\n\
             A clean result here would mean only that the walk found nothing.",
            command.root.display()
        );
        return ExitCode::Vacuous;
    }

    let findings = Check_Completeness_Mirrors(&sources);

    return Report(&findings, sources.len(), stdout);
}

/// Renders the findings and decides the exit code.
fn Report(findings: &[Finding], examined: usize, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ignored = writeln!(stdout, "{}", finding.Describe());
    }

    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();

    // The count of what was looked at is part of the result, not decoration. "0 findings"
    // over 4 files and "0 findings" over 400 are different claims, and a reader who
    // cannot tell them apart cannot tell a clean tree from a broken walk.
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} finding(s), {blocking} of which can fail a build",
        examined,
        findings.len()
    );

    return if blocking > 0
    {
        ExitCode::Violations
    }
    else
    {
        ExitCode::Ok
    };
}

/// Every `.rs` file under `root`, with its text.
///
/// `target` is skipped: it holds generated source that nobody authored, and judging a
/// build artifact would report findings against code the author cannot edit.
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

            if path.is_dir()
            {
                let skipped = path
                    .file_name()
                    .is_some_and(|name| return name == "target" || name == ".git");

                if !skipped
                {
                    pending.push(path);
                }
                continue;
            }

            if path.extension().is_some_and(|extension| return extension == "rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                sources.push(SourceFile::New(Relative(root, &path), text));
            }
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
fn Relative(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Root_Should_Default_To_Here()
    {
        assert_eq!(Parse(&[]).expect("no arguments is valid").root, PathBuf::from("."));
    }

    #[test]
    fn Test_A_Given_Root_Should_Win()
    {
        let arguments = vec!["--root".to_owned(), "somewhere".to_owned()];

        assert_eq!(
            Parse(&arguments).expect("--root is valid").root,
            PathBuf::from("somewhere")
        );
    }

    /// A mistyped flag must not be silently ignored into a default. `nomos check --rooot x`
    /// walking the current directory instead would report on the wrong tree and say
    /// nothing about it.
    #[test]
    fn Test_An_Unknown_Flag_Should_Refuse()
    {
        let arguments = vec!["--rooot".to_owned(), "x".to_owned()];

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--rooot"), "{error}");
        assert!(error.contains("usage"), "{error}");
    }

    /// A tree that is not there is not a clean tree.
    #[test]
    fn Test_A_Missing_Root_Should_Be_Unreadable()
    {
        let command = CheckCommand {
            root: PathBuf::from("no-such-directory-anywhere"),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        assert_eq!(Run(&command, &mut stdout, &mut stderr), ExitCode::Unreadable);
    }

    #[test]
    fn Test_A_Blocking_Finding_Should_Exit_Nonzero()
    {
        let findings = Check_Completeness_Mirrors(&[SourceFile::New(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
        )]);
        let mut stdout = Vec::new();

        assert_eq!(Report(&findings, 1, &mut stdout), ExitCode::Violations);
    }

    /// An advisory finding is reported and does not stop anybody. Thirteen of them exist
    /// in this workspace today, and a gate that can never be green is one everybody
    /// learns to bypass.
    #[test]
    fn Test_An_Advisory_Finding_Should_Not_Fail_The_Run()
    {
        let findings =
            Check_Completeness_Mirrors(&[SourceFile::New("a.rs", "pub const T: &[&str] = &[];\n")]);
        let mut stdout = Vec::new();

        assert_eq!(Report(&findings, 1, &mut stdout), ExitCode::Ok);
        assert!(!findings.is_empty(), "there is something to report");
    }

    /// The count is part of the result. Without it, a broken walk and a clean tree render
    /// the same line.
    #[test]
    fn Test_The_Report_Should_Say_How_Much_Was_Looked_At()
    {
        let mut stdout = Vec::new();

        let _code = Report(&[], 41, &mut stdout);

        let rendered = String::from_utf8(stdout).expect("output is utf-8");

        assert!(rendered.contains("41 file(s) examined"), "{rendered}");
    }
}
