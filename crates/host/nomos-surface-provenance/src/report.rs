//! Rendering findings as text a person reads, never as a verdict a machine acts on.

use crate::evaluate::CrateFinding;
use crate::git::{Since, Until};
use std::fmt::Write as _;

/// The whole report for one run: every crate checked, in the order it was checked.
pub(crate) fn Render_Report(since: Since<'_>, until: Until<'_>, findings: &[CrateFinding]) -> String
{
    let mut text = format!(
        "OD-STORE-002 surface/records report -- range {}..{}\n\n\
         A finding names a crate whose tests/contract/surface/<crate>.txt blob differs \
         between the two endpoints above, in a range where no commit touched \
         docs/records/. This is Derived evidence, not a violation: a rebless-only commit \
         (no design decided, just a snapshot catching up to a change made elsewhere) reads \
         identically to an undocumented one from git history alone. Read the commits named \
         below before treating either as record-worthy.\n\n",
        since.0, until.0
    );

    let flagged: Vec<&CrateFinding> = findings.iter().filter(|finding| finding.Is_A_Finding()).collect();
    let clean: Vec<&CrateFinding> = findings.iter().filter(|finding| !finding.Is_A_Finding()).collect();

    Render_Flagged(&mut text, &flagged);
    Render_Clean(&mut text, &clean);

    let _ = writeln!(text, "{} crate(s) checked, {} finding(s).", findings.len(), flagged.len());

    return text;
}

/// Every flagged crate's `FINDING` block, or the one line saying none were flagged.
fn Render_Flagged(text: &mut String, flagged: &[&CrateFinding])
{
    if flagged.is_empty()
    {
        text.push_str("no crate's surface changed with no docs/records/ commit in range.\n");
    }
    for finding in flagged
    {
        Append_Finding(text, finding);
    }
}

/// One `FINDING` block: the crate's name, why, and every commit that touched its
/// snapshot in this range.
fn Append_Finding(text: &mut String, finding: &CrateFinding)
{
    // `write!` into a `String` never fails; the `Result` exists only because
    // `std::fmt::Write` is shared with fallible sinks.
    let _ = writeln!(text, "FINDING {}", finding.krate);
    text.push_str("  surface changed; no commit under docs/records/ landed in this range\n");
    text.push_str("  commits touching the surface snapshot:\n");
    for commit in &finding.surface_commits
    {
        let _ = writeln!(text, "    {}  {}", Short_Hash(&commit.hash), commit.subject);
    }
    text.push('\n');
}

/// The one summary line naming every crate that checked out clean, when any did.
fn Render_Clean(text: &mut String, clean: &[&CrateFinding])
{
    if !clean.is_empty()
    {
        let names: Vec<&str> = clean.iter().map(|finding| finding.krate.as_str()).collect();
        let _ = writeln!(text, "checked and clean: {}\n", names.join(", "));
    }
}

/// How many characters of a commit hash [`Short`] keeps: enough for a person to
/// recognize, short enough not to dominate the line.
const SHORT_HASH_LENGTH: usize = 8;

/// The first eight characters of a commit hash — enough for a person to recognize,
/// short enough not to dominate the line.
fn Short_Hash(hash: &str) -> &str
{
    return hash.get(..SHORT_HASH_LENGTH).unwrap_or(hash);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::evaluate::CommitRef;

    /// What [`Finding_With_State`] needs beyond the crate's name -- bundled into one value
    /// rather than passed as two adjacent `bool` parameters, which a call site like
    /// `Finding_With_State("nomos-model", true, false)` cannot tell apart without counting.
    struct FindingState
    {
        surface_changed: bool,
        records_touched: bool,
    }

    fn Finding_With_State(krate: &str, state: FindingState) -> CrateFinding
    {
        return CrateFinding {
            krate: krate.to_owned(),
            surface_changed: state.surface_changed,
            surface_commits: vec![CommitRef {
                hash: "deadbeefcafe".to_owned(),
                subject: "reblessed".to_owned(),
            }],
            records_touched: state.records_touched,
        };
    }

    #[test]
    fn Test_A_Finding_Is_Named_With_Its_Commits()
    {
        let findings = vec![Finding_With_State(
            "nomos-model",
            FindingState { surface_changed: true, records_touched: false },
        )];

        let text = Render_Report(Since("a"), Until("b"), &findings);

        assert!(text.contains("FINDING nomos-model"));
        assert!(text.contains("deadbeef"));
        assert!(text.contains("reblessed"));
        assert!(text.contains("1 crate(s) checked, 1 finding(s)."));
    }

    #[test]
    fn Test_A_Clean_Crate_Is_Named_But_Not_Flagged()
    {
        let findings = vec![Finding_With_State(
            "nomos-model",
            FindingState { surface_changed: true, records_touched: true },
        )];

        let text = Render_Report(Since("a"), Until("b"), &findings);

        assert!(!text.contains("FINDING"));
        assert!(text.contains("checked and clean: nomos-model"));
        assert!(text.contains("0 finding(s)."));
    }
}
