//! The two ways a test is made to pass without being made to work, from code-standards'
//! `check-flaky-tests` (shared package `rules/general/testing/shared/flakiness`).
//!
//! Two rule ids, [`SLEEP_BASED_SYNCHRONIZATION`] and [`ZERO_FLAKE_POLICY`]. Neither has a
//! repository-configurable dimension -- both vocabularies below are fixed lists in
//! code-standards itself, with no `standards.json` block -- so this is a leaf module, the
//! same test `concurrency_text.rs` already applied to its own family, not a fifth
//! `OD-RULES-011` capability.
//!
//! `zero-flake-policy.md` names two prohibitions -- retry-until-green and silent disabling
//! with no reason -- but `check-flaky-tests`' own `judge.go` only ever emits
//! `FAULT_SLEEP_SYNC` or `FAULT_RETRY_UNTIL_GREEN`; the silent-disabling half is a
//! different tool's territory (`a-skipped-test-states-why`, `a-disabled-test-states-why`,
//! already ported), not this one's. So [`Check_A_Test_Does_Not_Retry_Until_Green`] carries
//! the whole of [`ZERO_FLAKE_POLICY`] here.
//!
//! # Where this reads narrower than the Go source
//!
//! `rust_flakiness.go` and `go_flakiness.go` both resolve a real parse tree (tree-sitter
//! for Rust, `go/ast` for Go) to scope a sleep call to the exact function it sits in, and
//! `rust_Retry_Attribute` reads an attribute as the function's literal previous sibling.
//! This module reads text and lines instead, and narrows in three ways worth naming rather
//! than guessing past:
//!
//! *Test scope* is file-level and marker-onward, matching this crate's established
//! trade-off (`concurrency_text.rs`'s own doc names the same choice for the opposite
//! direction -- there, a file is entirely exempt or entirely judged). Here: a source
//! already recognised by [`super::Is_Test_Or_Example_Source`] (a `tests/`/`examples/`
//! path, or a `_test.rs`/`_tests.rs` file) is judged from its first line, matching Rust's
//! own `is_In_Integration_Test_Crate` exactly. Any other source is judged from its first
//! `#[cfg(test)]` or `#[test]` line onward, to EOF -- the common Rust idiom of an inline
//! `#[cfg(test)] mod tests { ... }` at the bottom of an otherwise-production file, which a
//! path-only exemption would miss entirely. That is not a corner: `check-flaky-tests`' own
//! doc measured 50 sleep calls across 20 files of a real corpus, named as sitting in
//! `lock_free_semaphore.rs`, `disruptor.rs`, `latch.rs` -- ordinary source files, not
//! anything a `tests/` path would catch. Once a marker is seen the rest of the file is
//! judged without tracking the block's closing brace, which means a helper function
//! written below the test module but still inside the file is judged too; narrower would
//! require exactly the brace-depth tracking this crate has consistently declined to build
//! for this shape.
//!
//! *Sleep-call matching* is a path-suffix text search (`Has_Left_Boundary` before,
//! `Is_Call_Shape` after -- the same two-sided check `concurrency_text.rs`'s
//! `Find_Ordering_Variant` already applies to `Ordering::` variants), not a resolved call
//! expression. `rust_sleep_calls`'s own comment already measures the gap this shares with
//! the real tool: `use tokio::time::sleep;` followed by a bare `sleep(d)` is invisible to
//! both, because neither resolves `use` aliases. The direction of the gap is silence, the
//! same one code-standards' own comment accepts.
//!
//! *Go* is judged whole-file rather than only inside `Test`-prefixed functions -- a
//! `_test.go` file's non-`Test`-prefixed helpers are judged too, which the real tool does
//! not do. Retry-until-green is Rust-only, which is not a narrowing but a port of
//! `go_flakiness.go`'s own stated design: Go has no retry attribute syntax, and the real
//! tool explicitly declines to guess at a loop shape instead.
//!
//! # The escape hatch
//!
//! `flakiness.ALLOW_MARKER` is `"flakiness: allow"`, read same-line only
//! (`marker.Line_Has_Marker`, `kernel/foundation/marker/marker.go`) -- unlike the
//! atomic-ordering family's marker, no reason text is required by the real implementation,
//! so none is required here either; the doc's own prose asking for a reason predates this
//! and is read as illustration, not requirement, the same choice `concurrency_text.rs`
//! already made once for a stale claim.

use crate::{GO_LANGUAGE, RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards sleep-as-synchronization rule id.
pub const SLEEP_BASED_SYNCHRONIZATION: &str = "sleep-based-synchronization";
/// The code-standards zero-flake-policy rule id -- here, its retry-until-green half only.
pub const ZERO_FLAKE_POLICY: &str = "zero-flake-policy";

/// The literal marker this family's escape hatch requires, same-line, no reason needed.
const FLAKINESS_ALLOW_MARKER: &str = "flakiness: allow";

/// `rust_sleep_calls` in `rust_flakiness.go`, in the same order.
const RUST_SLEEP_CALLS: &[&str] = &["thread::sleep", "tokio::time::sleep", "async_std::task::sleep", "spin_sleep::sleep"];

/// `go_flakiness.go` matches only `time.Sleep`.
const GO_SLEEP_CALLS: &[&str] = &["time.Sleep"];

/// `rust_retry_attributes` in `rust_flakiness.go`, in the same order.
const RUST_RETRY_ATTRIBUTES: &[&str] = &["retry", "flaky_test", "flaky", "retry_test"];

/// Reports a sleep standing in for synchronization in a test -- `FAULT_SLEEP_SYNC`.
#[must_use]
pub fn Check_Sleep_Is_Not_Synchronization(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Own_Implementation_File(source)
        {
            continue;
        }

        let Some(vocabulary) = Sleep_Vocabulary_For(source)
        else
        {
            continue;
        };

        let source_findings = Sleep_Findings_In(source, vocabulary);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// The sleep vocabulary `source` is judged against, or `None` if it is not in scope at all
/// -- every Rust source, but only a Go source whose own filename already marks it a test.
fn Sleep_Vocabulary_For(source: &SourceFile) -> Option<&'static [&'static str]>
{
    if source.Is_Written_In(RUST_LANGUAGE)
    {
        return Some(RUST_SLEEP_CALLS);
    }

    if source.Is_Written_In(GO_LANGUAGE) && Is_Go_Test_File(source)
    {
        return Some(GO_SLEEP_CALLS);
    }

    return None;
}

fn Sleep_Findings_In(source: &SourceFile, vocabulary: &[&str]) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let Some(scan_from) = Test_Scan_Start(source, &lines)
    else
    {
        return Vec::new();
    };

    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(scan_from)
    {
        if let Some(finding) = Sleep_Finding_At(source, index, line, vocabulary)
        {
            findings.push(finding);
        }
    }

    return findings;
}

/// The first in-scope line index, or [`None`] if nothing in this file is test scope.
fn Test_Scan_Start(source: &SourceFile, lines: &[&str]) -> Option<usize>
{
    if super::Is_Test_Or_Example_Source(source) || Is_Go_Test_File(source)
    {
        return Some(0);
    }

    for (index, line) in lines.iter().enumerate()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") || trimmed == "#[test]"
        {
            return Some(index);
        }
    }

    return None;
}

fn Sleep_Finding_At(source: &SourceFile, index: usize, line: &str, vocabulary: &[&str]) -> Option<Finding>
{
    let code = Code_Prefix(line);
    let call = Sleep_Match_In(code, vocabulary)?;

    if Has_Allow_Marker(line)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding {
        rule: RuleId::New(SLEEP_BASED_SYNCHRONIZATION),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{call}(...)` at {}:{line_number} is a sleep standing in for synchronization in a test", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    });
}

/// The code before any `//` line comment -- this crate's established convention
/// (`rust_text::Code_Prefix`), duplicated here per this crate's per-file helper convention.
fn Code_Prefix(line: &str) -> &str
{
    return line.split("//").next().unwrap_or(line);
}

fn Sleep_Match_In<'a>(code: &str, vocabulary: &[&'a str]) -> Option<&'a str>
{
    let mut earliest: Option<(usize, &'a str)> = None;

    for &call in vocabulary
    {
        if let Some(start) = Find_Sleep_Call(code, Call(call))
            && earliest.is_none_or(|(earliest_start, _)| return start < earliest_start)
        {
            earliest = Some((start, call));
        }
    }

    return earliest.map(|(_, call)| return call);
}

/// The literal sleep-call path [`Find_Sleep_Call`] searches for, wrapped so its parameter
/// position cannot be transposed with `code` — the text being searched — with nothing to
/// catch it.
struct Call<'a>(&'a str);

/// A path-suffix match: the byte before `call` must not be an identifier byte (so
/// `worker_thread::sleep` does not match `thread::sleep`, while a wider qualification like
/// `mything::thread::sleep` still does, matching `is_Sleep_Path`'s own segment-alignment
/// semantics), and the first non-whitespace byte after it must be `(`.
fn Find_Sleep_Call(code: &str, call: Call<'_>) -> Option<usize>
{
    let bytes = code.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = code.get(search_from..).and_then(|rest| return rest.find(call.0))
    {
        let start = search_from.saturating_add(offset);
        let end = start.saturating_add(call.0.len());
        if Has_Left_Boundary(bytes, start) && Followed_By_A_Call_Paren(code, end)
        {
            return Some(start);
        }
        search_from = start.saturating_add(1);
    }

    return None;
}

fn Has_Left_Boundary(bytes: &[u8], start: usize) -> bool
{
    return start.checked_sub(1).and_then(|previous| return bytes.get(previous)).is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

fn Is_Ident_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

fn Followed_By_A_Call_Paren(code: &str, end: usize) -> bool
{
    return code.get(end..).is_some_and(|rest| return rest.trim_start().starts_with('('));
}

/// Reports a test carrying a retry-until-green attribute -- `FAULT_RETRY_UNTIL_GREEN`.
#[must_use]
pub fn Check_A_Test_Does_Not_Retry_Until_Green(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Retry_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Retry_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(finding) = Retry_Finding_At(source, &lines, index, line)
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Retry_Finding_At(source: &SourceFile, lines: &[&str], index: usize, line: &str) -> Option<Finding>
{
    let attribute = Retry_Attribute_Name(line.trim())?;

    if Has_Allow_Marker(line) || !Decorates_A_Function(lines, index)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding {
        rule: RuleId::New(ZERO_FLAKE_POLICY),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "`#[{attribute}]` at {}:{line_number} retries a test until it passes, which reports the best of N attempts rather than the truth",
            source.path
        ),
        locations: vec![format!("{}:{line_number}", source.path)],
    });
}

/// A retry-vocabulary attribute name, stripped of its `#[...]` shell and any `(...)`/`=...`
/// argument, the same way `rust_Retry_Attribute` isolates the bare name before comparing.
fn Retry_Attribute_Name(trimmed: &str) -> Option<&'static str>
{
    let without_prefix = trimmed.strip_prefix("#[")?;
    let without_suffix = without_prefix.strip_suffix(']')?;
    let bare = without_suffix.trim();
    let name_end = bare.find(['(', '=']).unwrap_or(bare.len());
    let name = bare.get(..name_end)?.trim();
    return RUST_RETRY_ATTRIBUTES.iter().find(|&&retry| return retry == name).copied();
}

/// The attribute stack is contiguous (`rust_Retry_Attribute`'s own comment names this): the
/// next line that is neither blank nor another `#[...]` attribute must be a function
/// declaration, or this attribute decorates something else entirely.
fn Decorates_A_Function(lines: &[&str], attribute_index: usize) -> bool
{
    for line in lines.iter().skip(attribute_index.saturating_add(1))
    {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#[")
        {
            continue;
        }
        return trimmed.contains("fn ") || trimmed.contains("fn(");
    }

    return false;
}

fn Is_Go_Test_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with("_test.go");
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// The marker must lead a `//` comment on the same line as the fault, matching
/// `marker.Line_Has_Marker`'s own same-line-only reading -- no reason text required, unlike
/// the atomic-ordering family's marker.
fn Has_Allow_Marker(line: &str) -> bool
{
    return line.split_once("//").is_some_and(|(_, after)| return after.trim_start().starts_with(FLAKINESS_ALLOW_MARKER));
}

/// This file's own path. Every fixture below spells a real sleep call or retry attribute
/// inside a Rust string literal, which would otherwise self-match when this crate checks
/// its own workspace -- the same self-exemption `concurrency_text.rs`, `rust_text.rs` and
/// `security_text.rs` each carry for the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "checks/flakiness_text.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with(OWN_IMPLEMENTATION_FILE);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Report_A_Qualified_Sleep_In_An_Integration_Test()
    {
        let source = Source("tests/lock.rs", "thread::sleep(Duration::from_millis(50));\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SLEEP_BASED_SYNCHRONIZATION));
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Report_A_Sleep_Inside_An_Inline_Cfg_Test_Module()
    {
        let source = Source(
            "src/latch.rs",
            "pub fn is_open(&self) -> bool { true }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn opens() {\n        thread::sleep(Duration::from_millis(50));\n    }\n}\n",
        );
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Ignore_A_Sleep_In_Production_Code()
    {
        let source = Source("src/rate_limiter.rs", "thread::sleep(backoff);\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Ignore_A_Path_That_Merely_Ends_In_The_Same_Letters()
    {
        let source = Source("tests/worker.rs", "worker_thread::sleep(backoff);\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Accept_A_Same_Line_Allow_Marker()
    {
        let source = Source("tests/lock.rs", "thread::sleep(debounce); // flakiness: allow this waits on the debounce window under test\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Report_A_Qualified_Time_Sleep_In_A_Go_Test_File()
    {
        let source = Source("worker_test.go", "func TestReady(t *testing.T) {\n\ttime.Sleep(50 * time.Millisecond)\n}\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Sleep_Is_Not_Synchronization_Should_Ignore_Go_Sleep_Outside_A_Test_File()
    {
        let source = Source("worker.go", "func Ready() {\n\ttime.Sleep(50 * time.Millisecond)\n}\n");
        let findings = Check_Sleep_Is_Not_Synchronization(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Test_Does_Not_Retry_Until_Green_Should_Report_A_Retry_Attribute()
    {
        let source = Source("src/counter.rs", "#[cfg(test)]\nmod tests {\n    #[retry(3)]\n    #[test]\n    fn test_it() {\n        assert!(true);\n    }\n}\n");
        let findings = Check_A_Test_Does_Not_Retry_Until_Green(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(ZERO_FLAKE_POLICY));
    }

    #[test]
    fn Test_Check_A_Test_Does_Not_Retry_Until_Green_Should_Report_The_Bare_Flaky_Attribute()
    {
        let source = Source("src/counter.rs", "#[flaky]\nfn test_it() {\n    assert!(true);\n}\n");
        let findings = Check_A_Test_Does_Not_Retry_Until_Green(&[source]);
        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Test_Does_Not_Retry_Until_Green_Should_Ignore_An_Unrelated_Attribute()
    {
        let source = Source("src/counter.rs", "#[derive(Debug)]\nstruct Counter;\n");
        let findings = Check_A_Test_Does_Not_Retry_Until_Green(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Test_Does_Not_Retry_Until_Green_Should_Ignore_A_Retry_Named_Attribute_On_A_Non_Function()
    {
        let source = Source("src/counter.rs", "#[retry]\nstruct RetryPolicy;\n");
        let findings = Check_A_Test_Does_Not_Retry_Until_Green(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Test_Does_Not_Retry_Until_Green_Should_Accept_A_Same_Line_Allow_Marker()
    {
        let source = Source(
            "src/counter.rs",
            "#[retry(3)] // flakiness: allow this test hits a real external endpoint\nfn test_it() {\n    assert!(true);\n}\n",
        );
        let findings = Check_A_Test_Does_Not_Retry_Until_Green(&[source]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
