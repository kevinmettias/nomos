//! Script discipline rules from code-standards.
//!
//! The repository-wide standard has more to say about scripts, including executable bits
//! and sourced shell libraries. This crate is handed source text, not file modes, so
//! [`Check_Scripts_Use_A_Portable_Shebang`] and [`Check_A_Script_Declares_Its_Purpose`]
//! judge the exact subset visible from a file's first lines: shebang portability and the
//! purpose comment immediately after the shebang/blank-line prefix.
//!
//! [`Check_Executed_Scripts_Set_Nounset`] is a fourth rule in the same file, and the last
//! of `check-script-discipline`'s own text-local rules: an executed shebang script that
//! never enables `set -u` (or `-euo pipefail`, or the long `-o nounset`) is flagged on line
//! 1, unless it is a *sourced library* — a file whose only top-level statements define
//! (a function header, or a `NAME=value`/`readonly`/`declare`/`typeset`/`export`/`local`
//! declaration) and never *do*. Ported from `breach.go`'s `missing_nounset`/`has_nounset`/
//! `is_Sourced_Library`, including the multi-line quote-state tracking `quote_State_After`
//! carries so a single-quoted literal spanning lines is never misread as a top-level
//! statement — the defect that file's own doc comment records finding in a real sourced
//! library. Gated the same way its two siblings above are, by [`Is_Shebang_Script`], since
//! this crate has no policy-driven script/extension walk to filter by path instead.
//!
//! [`Check_Declared_Tooling_Language_For_Scripts`] is different in shape: it needs a
//! repository's own declaration, so it reads `nomos.cap.scripting.policy` — a third
//! `OD-RULES-011` instance, mirroring `checks::structure::Resolve_Limit` and `checks::
//! naming::Resolve_Case`, except its own capability's absence (or an undeclared tooling
//! language) resolves to *no findings at all* rather than a hardcoded prior value: this
//! rule never existed before this capability did, so there is no earlier default to fall
//! back to, and `check-script-discipline`'s own `spec.go` states the reason directly — a
//! repository that has not declared a tooling language has opted out, not defaulted in.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_scripting_policy::ScriptingPolicyPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards portable-shebang rule id.
pub const SCRIPTS_USE_A_PORTABLE_SHEBANG: &str = "scripts-use-a-portable-shebang";
/// The code-standards script-purpose rule id.
pub const A_SCRIPT_DECLARES_ITS_PURPOSE: &str = "a-script-declares-its-purpose";
/// The code-standards executed-scripts-nounset rule id.
pub const EXECUTED_SCRIPTS_SET_NOUNSET: &str = "executed-scripts-set-nounset";
/// The code-standards declared-tooling-language rule id.
pub const DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS: &str = "declared-tooling-language-for-scripts";

/// Reports shebang scripts whose first line hardcodes an interpreter path.
#[must_use]
pub fn Check_Scripts_Use_A_Portable_Shebang(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let Some(interpreter) = Shebang_Interpreter(source)
        else
        {
            continue;
        };

        if !interpreter.starts_with("/usr/bin/env ")
        {
            findings.push(Finding_For_Source(
                source,
                SCRIPTS_USE_A_PORTABLE_SHEBANG,
                "has a hardcoded shebang; use `#!/usr/bin/env <interpreter>`",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports shebang scripts whose first nonblank line after the shebang is not a comment.
#[must_use]
pub fn Check_A_Script_Declares_Its_Purpose(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Shebang_Script(source)
        {
            continue;
        }

        if !Has_Purpose_Comment(source)
        {
            findings.push(Finding_For_Source(
                source,
                A_SCRIPT_DECLARES_ITS_PURPOSE,
                "does not declare its purpose in the first nonblank line after the shebang",
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports executed shebang scripts that never enable nounset, excusing a sourced library
/// (one whose body only defines and never does) from the exemption `check-script-
/// discipline`'s own `breach.go` states: `-u` there would be the caller's shell option to
/// change, not the library's.
#[must_use]
pub fn Check_Executed_Scripts_Set_Nounset(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Shebang_Script(source)
        {
            continue;
        }

        let lines: Vec<&str> = source.text.split('\n').collect();
        if Is_Sourced_Library(&source.text) || Has_Nounset(&lines)
        {
            continue;
        }

        findings.push(Finding_For_Source(
            source,
            EXECUTED_SCRIPTS_SET_NOUNSET,
            "must `set -u` (nounset), so a misspelled variable name stops it instead of expanding to nothing; that is the \
             difference between an error and a launcher silently operating on the wrong path",
        ));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Reports files whose path ends in an extension the repository has declared forbidden,
/// once the repository has opted in by declaring its tooling language. Ported from
/// `check-script-discipline`'s own `Is_Forbidden`: the path alone convicts a file, nothing
/// is read.
#[must_use]
pub fn Check_Declared_Tooling_Language_For_Scripts(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let Some(policy) = Resolve_Scripting_Policy(facts)
    else
    {
        return Vec::new();
    };
    let Some(language) = &policy.tooling_language
    else
    {
        return Vec::new();
    };

    let mut findings = Vec::new();
    for source in sources
    {
        if Is_Forbidden_Extension(&source.path, &policy.forbidden_extensions)
        {
            findings.push(Finding_For_Source(
                source,
                DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS,
                &format!("is written in a language this repository's declared tooling language ({language}) does not use"),
            ));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// This crate's own floor for `nomos.cap.scripting.policy` — stated at the capability's
/// own ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement` and
/// `checks::structure::Limits_Policy_Requirement` exactly, for the third sibling
/// capability.
fn Scripting_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_scripting_policy::Capability(),
        nomos_cap_scripting_policy::CONTRACT_VERSION,
        nomos_cap_scripting_policy::Ceiling(),
    );
}

/// Resolves a repository's own declared scripting policy — `None` on any `Require`
/// failure, per `OD-CAPABILITY-004`/`OD-RULES-011`'s settled optional-read pattern: this
/// capability is optional, and its absence must never surface as a `Finding` or this
/// capability's own `Applicability`.
fn Resolve_Scripting_Policy(facts: &mut dyn FactReader) -> Option<ScriptingPolicyPayload>
{
    let subject = nomos_model::Subject_Of_Path("");
    let fact = facts
        .Require(&nomos_cap_scripting_policy::Capability(), &subject, InputDigest::Of(&[]), &Scripting_Policy_Requirement())
        .ok()?;

    return nomos_cap_scripting_policy::Parse_Payload(&fact.payload.bytes).ok();
}

fn Is_Forbidden_Extension(path: &str, forbidden_extensions: &[String]) -> bool
{
    let lowered = path.to_lowercase();
    return forbidden_extensions
        .iter()
        .any(|extension| return lowered.ends_with(&extension.to_lowercase()));
}

fn First_Line(source: &SourceFile) -> Option<&str>
{
    return source.text.lines().next();
}

/// The interpreter a source's first line names, if that line is a shebang at all.
///
/// A shebang names its interpreter by absolute path, so `#!` is followed -- after the
/// optional space some conventions write -- by `/`. The two bytes alone do not distinguish
/// a script from a file whose language spells something else the same way: Rust's inner
/// attributes begin `#![`, so every crate root opening with `#![forbid(unsafe_code)]` read
/// as a script here, and was then reported for a hardcoded interpreter it does not have and
/// a purpose comment a Rust file has no place to put. Requiring the path keeps the rules
/// language-agnostic, which is the reason they judge a first line rather than an extension.
fn Shebang_Interpreter(source: &SourceFile) -> Option<&str>
{
    let path = First_Line(source)?.strip_prefix("#!")?.trim_start_matches([' ', '\t']);

    if !path.starts_with('/')
    {
        return None;
    }

    return Some(path);
}

fn Is_Shebang_Script(source: &SourceFile) -> bool
{
    return Shebang_Interpreter(source).is_some();
}

fn Has_Purpose_Comment(source: &SourceFile) -> bool
{
    return source
        .text
        .lines()
        .skip(1)
        .find(|line| return !line.trim().is_empty())
        .is_some_and(|line| return line.trim_start().starts_with('#'));
}

/// Whether any line enables nounset, in any spelling bash accepts: `set -u`, a combined
/// group such as `set -euo pipefail`, or the long `set -o nounset`. A `set +u`, which turns
/// the option *off*, is deliberately not a match — it begins with `+`, not `-`.
fn Has_Nounset(lines: &[&str]) -> bool
{
    const KEYWORD_AND_ONE_FLAG: usize = 2;

    for raw in lines
    {
        let code = Code_On(raw);
        let fields: Vec<&str> = code.split_whitespace().collect();
        if fields.len() < KEYWORD_AND_ONE_FLAG || fields.first() != Some(&"set")
        {
            continue;
        }

        for (index, field) in fields.iter().enumerate().skip(1)
        {
            if *field == "-o" && fields.get(index.saturating_add(1)) == Some(&"nounset")
            {
                return true;
            }
            // A short-flag group (`-u`, `-eu`, `-euo`). `-o` introduces a long name and is
            // handled above; `+u` begins with `+` and never reaches this branch.
            if field.starts_with('-') && !field.starts_with("-o") && field.contains('u')
            {
                return true;
            }
        }
    }

    return false;
}

/// A file meant to be *sourced* rather than executed: one whose body only defines —
/// functions and constants — and never *does*. The signal is the absence of any top-level
/// (column-zero) statement that runs something; the first one flips the file from library
/// to executed and ends the scan early.
fn Is_Sourced_Library(source: &str) -> bool
{
    let mut saw_definition = false;
    let mut inside = QuoteState::None;

    for raw in source.split('\n')
    {
        let continuation = inside != QuoteState::None;
        inside = Quote_State_After(raw, inside);
        if continuation
        {
            // Inside a literal that opened on an earlier line: never a top-level statement.
            continue;
        }

        if raw.starts_with(' ') || raw.starts_with('\t')
        {
            // Indented: a function body, a heredoc, or a nested block.
            continue;
        }

        let code = Code_On(raw).trim();
        if code.is_empty()
        {
            continue;
        }
        if code == "{" || code == "}" || code == "(" || code == ")"
        {
            // A function body's braces, or a multi-line array's parentheses, standing alone.
            continue;
        }
        if Is_Function_Header(code) || Is_Declaration(code)
        {
            saw_definition = true;
            continue;
        }

        return false;
    }

    return saw_definition;
}

/// A top-level line that defines a function rather than runs a command: `name() {`, the
/// bare `name()`, or the `function name` spelling.
fn Is_Function_Header(code: &str) -> bool
{
    if code.starts_with("function ")
    {
        return true;
    }

    let Some(paren) = code.find('(')
    else
    {
        return false;
    };
    if paren == 0
    {
        return false;
    }

    let name = &code[..paren];
    if name.trim() != name
    {
        // Whitespace before `(`: a call or a subshell, not a definition header.
        return false;
    }
    if !name.chars().all(|symbol| return symbol.is_ascii_alphanumeric() || symbol == '_' || symbol == '-')
    {
        return false;
    }

    return code[paren..].starts_with("()");
}

/// A top-level line that introduces a name without running anything: a `readonly`,
/// `declare`, `typeset`, `export` or `local` keyword, or a bare `NAME=value` (including
/// `NAME+=` and `NAME[i]=`).
fn Is_Declaration(code: &str) -> bool
{
    const KEYWORDS: [&str; 5] = ["readonly ", "declare ", "typeset ", "export ", "local "];
    if KEYWORDS.iter().any(|keyword| return code.starts_with(*keyword))
    {
        return true;
    }

    let Some(equals) = code.find('=')
    else
    {
        return false;
    };
    if equals == 0
    {
        return false;
    }

    let before_equals = &code[..equals];
    let mut name = before_equals.strip_suffix('+').unwrap_or(before_equals);
    if let Some(index) = name.find('[')
    {
        name = &name[..index];
    }
    if name.is_empty()
    {
        return false;
    }

    for (position, symbol) in name.chars().enumerate()
    {
        let letter = symbol.is_ascii_alphabetic();
        if position == 0 && !(letter || symbol == '_')
        {
            return false;
        }
        if !(letter || symbol == '_' || symbol.is_ascii_digit())
        {
            return false;
        }
    }

    return true;
}

/// Which kind of string literal, if any, is still open at the end of a line. A closed set
/// rather than two bools, because "inside single" and "inside double" are mutually
/// exclusive and two bools admit a fourth state that cannot exist.
#[derive(Clone, Copy, PartialEq, Eq)]
enum QuoteState
{
    /// No literal is open; the next line begins in code.
    None,
    /// A `'...'` literal is open. Shell gives single quotes no escape at all, so only the
    /// next `'` can close one.
    Single,
    /// A `"..."` literal is open. Backslash escapes inside these, so a `\"` does not close it.
    Double,
}

/// Which literal, if any, is still open once `line` has been read, given whichever was open
/// when it began. Stops at an unquoted `#`, because the rest of the line is a comment and a
/// comment's contents are not code — without that, an ordinary apostrophe in prose
/// (`# don't do this`) would open a single-quoted string that never closes.
fn Quote_State_After(line: &str, opening: QuoteState) -> QuoteState
{
    let bytes = line.as_bytes();
    let mut state = opening;
    let mut index = 0usize;

    while let Some(&character) = bytes.get(index)
    {
        match state
        {
            QuoteState::Single =>
            {
                if character == b'\''
                {
                    state = QuoteState::None;
                }
                index = index.saturating_add(1);
            }
            QuoteState::Double =>
            {
                if character == b'\\'
                {
                    index = index.saturating_add(2); // the escaped character cannot close anything
                    continue;
                }
                if character == b'"'
                {
                    state = QuoteState::None;
                }
                index = index.saturating_add(1);
            }
            QuoteState::None =>
            {
                if character == b'\\'
                {
                    index = index.saturating_add(2); // escapes the next character, quote or not
                    continue;
                }
                else if character == b'#' && Begins_A_Word(line, index)
                {
                    return state; // a comment: nothing after it is code
                }
                else if character == b'\''
                {
                    state = QuoteState::Single;
                }
                else if character == b'"'
                {
                    state = QuoteState::Double;
                }
                index = index.saturating_add(1);
            }
        }
    }

    return state;
}

/// Whether the byte at `index` starts a word — it is first on the line, or the character
/// before it is a space or a tab. Shell only treats `#` as a comment there.
fn Begins_A_Word(line: &str, index: usize) -> bool
{
    if index == 0
    {
        return true;
    }
    return line
        .as_bytes()
        .get(index.saturating_sub(1))
        .is_some_and(|&byte| return byte == b' ' || byte == b'\t');
}

/// A line with its trailing `#...` comment stripped, when that `#` starts a word and sits
/// outside a double-quoted literal opened and closed on this same line.
fn Code_On(line: &str) -> &str
{
    match Unquoted_Hash_Index(line)
    {
        Some(index) => &line[..index],
        None => line,
    }
}

/// The index of the first `#` in `line` that is not inside a double-quoted literal, or
/// `None` if every occurrence is. An odd count of `"` before a position means one literal is
/// still open there.
fn Unquoted_Hash_Index(line: &str) -> Option<usize>
{
    let mut offset = 0usize;
    loop
    {
        let found = line.get(offset..)?.find('#')?;
        let at = offset.saturating_add(found);
        if line.get(..at)?.matches('"').count() % 2 == 0
        {
            return Some(at);
        }
        offset = at.saturating_add(1);
    }
}

fn Finding_For_Source(source: &SourceFile, rule: &str, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, TestOffering};
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_cap_scripting_policy::Encode_Payload;
    use nomos_capability::Registry;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Report_A_Hardcoded_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/bin/bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(SCRIPTS_USE_A_PORTABLE_SHEBANG));
    }

    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Accept_A_Portable_Shebang()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A Rust crate root opening on an inner attribute. `#![forbid(unsafe_code)]` begins
    /// with the same two bytes a shebang does, and this is the shape that made three of
    /// this workspace own crate roots report as scripts with a hardcoded interpreter.
    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Ignore_A_Rust_Inner_Attribute()
    {
        let source = Source("src/lib.rs", "#![forbid(unsafe_code)]\n\npub fn Check() {}\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The same inner attribute under the other rule: neither may read it as a script,
    /// because a Rust file has nowhere to put the purpose comment this one asks for.
    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_A_Rust_Inner_Attribute()
    {
        let source = Source("src/lib.rs", "#![forbid(unsafe_code)]\n\npub fn Check() {}\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A space between the two bytes and the path is a shebang some conventions write, and
    /// it stays one: the discriminator is the absolute path, not the absence of a space.
    #[test]
    fn Test_Check_Scripts_Use_A_Portable_Shebang_Should_Still_Report_A_Spaced_Hardcoded_Shebang()
    {
        let source = Source("scripts/check.sh", "#! /bin/bash\n# check -- run checks\n");

        let findings = Check_Scripts_Use_A_Portable_Shebang(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Report_Code_As_The_First_Content()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(A_SCRIPT_DECLARES_ITS_PURPOSE));
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Accept_A_Comment_After_Blanks()
    {
        let source = Source("scripts/check.sh", "#!/usr/bin/env bash\n\n# check -- run checks\nset -u\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Script_Declares_Its_Purpose_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source("src/lib.rs", "fn Check() {}\n");

        let findings = Check_A_Script_Declares_Its_Purpose(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_A_Clean_Launcher()
    {
        let source = Source(
            "scripts/build.sh",
            "#!/usr/bin/env bash\n# build.sh -- set the toolchain and hand off to the real gate\nset -u\nexec ./bin/gate \"$@\"\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Report_A_Missing_Nounset()
    {
        let source = Source("scripts/build.sh", "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nexec ./bin/gate \"$@\"\n");

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(EXECUTED_SCRIPTS_SET_NOUNSET));
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_A_Combined_Flag_Group()
    {
        let source = Source(
            "scripts/build.sh",
            "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset -euo pipefail\nexec ./bin/gate \"$@\"\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_The_Long_Form()
    {
        let source = Source(
            "scripts/build.sh",
            "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset -o nounset\nexec ./bin/gate \"$@\"\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Report_Nounset_Turned_Off()
    {
        let source = Source("scripts/build.sh", "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset +u\nexec ./bin/gate \"$@\"\n");

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Exempt_A_Sourced_Library()
    {
        let source = Source(
            "scripts/toolchain.sh",
            "#!/usr/bin/env bash\n# toolchain.sh -- SOURCE this; do not execute it\nensure_go() {\n  command -v go\n}\nrun_gate() {\n  exec go run \"$1\"\n}\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "a sourced library owes no nounset of its own: {findings:?}");
    }

    /// The one case `is_Sourced_Library`'s own doc names as the defect this state machine
    /// exists to fix: a single-quoted literal whose second line opens at column zero must
    /// not be misread as a top-level statement that flips the file to executed.
    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Not_Misread_A_Multiline_Literal_As_Code()
    {
        let source = Source(
            "scripts/toolchain.sh",
            "#!/usr/bin/env bash\n# toolchain.sh -- SOURCE this; do not execute it\ndescribe() {\n  printf '%s\n'     \"vendored grammar\"\n}\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "the literal's continuation line must not read as a top-level statement: {findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Treat_A_Top_Level_Call_As_Executed()
    {
        let source = Source(
            "scripts/build.sh",
            "#!/usr/bin/env bash\n# build.sh\nmain() {\n  exec ./bin/gate \"$@\"\n}\nmain \"$@\"\n",
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "a function call at the top level makes the file executed, not a library: {findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source("src/lib.rs", "fn Check() {}\n");

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_The_Capability_Is_Unmaterialized()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an unconfigured repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_No_Findings_When_No_Language_Is_Declared()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, None, vec![".ps1".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "an opted-out repository must not be judged: {findings:?}");
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Report_A_Forbidden_Extension()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, Some("rust".to_owned()), vec![".ps1".to_owned(), ".sh".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("tooling/deploy.ps1", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS));
        assert!(found.summary.contains("rust"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_Declared_Tooling_Language_For_Scripts_Should_Accept_A_File_Outside_The_Forbidden_List()
    {
        let TestOffering { mut store, registry, offer } = Scripting_Offering();
        Materialize_Scripting_Fact(&mut store, &offer, Some("rust".to_owned()), vec![".ps1".to_owned()]);
        let mut facts = Reader::On(&store, &registry, test_support::Test_Context());
        let source = Source("scripts/setup.sh", "");

        let findings = Check_Declared_Tooling_Language_For_Scripts(&[source], &mut facts);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Scripting_Offering() -> TestOffering
    {
        return test_support::Offering(
            nomos_cap_scripting_policy::Capability_Contract(),
            nomos_cap_scripting_policy::Capability(),
            nomos_cap_scripting_policy::CONTRACT_VERSION,
            "nomos.test.scripting.provides",
            nomos_cap_scripting_policy::Ceiling(),
        );
    }

    fn Materialize_Scripting_Fact(
        store: &mut MemoryFactStore,
        offer: &nomos_capability::ProviderOffer,
        tooling_language: Option<String>,
        forbidden_extensions: Vec<String>,
    )
    {
        let payload = ScriptingPolicyPayload { tooling_language, forbidden_extensions };
        test_support::Materialize(
            store,
            nomos_model::Subject_Of_Path(""),
            offer,
            InputDigest::Of(&[]),
            nomos_cap_scripting_policy::Payload_Schema(),
            Encode_Payload(&payload),
        );
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
