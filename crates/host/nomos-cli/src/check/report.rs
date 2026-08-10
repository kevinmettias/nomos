//! Saying what was found, over how much, and what would fail a build.

use super::{Finding, Write, ExitCode};

/// How much of the world this run actually saw.
///
/// Two denominators and not one. "0 findings over 400 files" and "0 findings over 400
/// files none of which produced a fact" are different claims, and the second is a broken
/// run — the prototype reported the first shape for a check that had walked nothing, and
/// the defect was invisible because the report had no place to put the number that would
/// have shown it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Examined
{
    /// Files the walk read.
    pub(super) files: usize,
    /// Files a syntax fact was materialized for.
    pub(super) facts: usize,
}

/// Renders the findings and decides the exit code.
pub(super) fn Report(findings: &[Finding], examined: Examined, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ignored = writeln!(stdout, "{}", finding.Describe());
    }

    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();
    Counts(findings.len(), blocking, examined, stdout);

    if blocking > 0
    {
        return ExitCode::Violations;
    }

    return ExitCode::Ok;
}

/// What was looked at, which is part of the result and not decoration.
///
/// "0 findings" over 4 files and "0 findings" over 400 are different claims, and so are
/// "400 files" and "400 files, 12 of which produced a fact" — a reader who cannot tell
/// them apart cannot tell a clean tree from a broken walk.
pub(super) fn Counts(found: usize, blocking: usize, examined: Examined, stdout: &mut impl Write)
{
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} with a syntax fact, {found} finding(s), {blocking} of \
         which can fail a build",
        examined.files,
        examined.facts
    );
}
