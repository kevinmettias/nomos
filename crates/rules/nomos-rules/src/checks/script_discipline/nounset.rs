//! `script_discipline.rs`'s executed-scripts-need-nounset rule, with the comment stripping and
//! the `set`-flag reading it rests on; whether a file is a sourced library rather than a
//! program is [`sourced_library`]'s decision.

mod sourced_library;

use super::{Because, EXECUTED_SCRIPTS_SET_NOUNSET, Finding_For_Source, Is_Shebang_Script, Rule};
use crate::SourceFile;
use nomos_contracts::Finding;

/// Reports executed shebang scripts that never enable nounset, excusing a sourced library
/// (one whose body only defines and never does) from the exemption `check-script-
/// discipline`'s own `breach.go` states: `-u` there would be the caller's shell option to
/// change, not the library's.
#[must_use]
pub fn Check_Executed_Scripts_Set_Nounset(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = sources.iter().filter_map(Nounset_Finding_For).collect();

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// `Some` when `source` is an executed shebang script that never enables nounset -- `None`
/// for a non-script, a sourced library (exempted above), or a script that already does.
fn Nounset_Finding_For(source: &SourceFile) -> Option<Finding>
{
    use sourced_library::Is_Sourced_Library;

    if !Is_Shebang_Script(source)
    {
        return None;
    }

    let lines: Vec<&str> = source.text.split('\n').collect();
    if Is_Sourced_Library(&source.text) || Has_Nounset(&lines)
    {
        return None;
    }

    let finding = Finding_For_Source(
        source,
        Rule(EXECUTED_SCRIPTS_SET_NOUNSET),
        Because(
            "must `set -u` (nounset), so a misspelled variable name stops it instead of expanding to nothing; that is the \
             difference between an error and a launcher silently operating on the wrong path",
        ),
    );
    return Some(finding);
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

        if Fields_Enable_Nounset(&fields)
        {
            return true;
        }
    }

    return false;
}

/// A line with its trailing `#...` comment stripped, when that `#` starts a word and sits
/// outside a double-quoted literal opened and closed on this same line.
fn Code_On(line: &str) -> &str
{
    return match Unquoted_Hash_Index(line)
    {
        Some(index) => &line[..index],
        None => line,
    };
}

/// The index of the first `#` in `line` that is not inside a double-quoted literal, or
/// `None` if every occurrence is. An odd count of `"` before a position means one literal is
/// still open there.
///
/// A `while let` over the line's remainder rather than a bare `loop`: the scan advances one
/// byte per non-matching `#`, so running past the end of the line is a real way out, and the
/// `return None` below is where it lands rather than a diverging tail expression the reader
/// has to reconstruct.
fn Unquoted_Hash_Index(line: &str) -> Option<usize>
{
    const QUOTES_PER_PAIR: usize = 2;

    let mut offset = 0usize;
    while let Some(remainder) = line.get(offset..)
    {
        let found = remainder.find('#')?;
        let at = offset.saturating_add(found);
        if line.get(..at)?.matches('"').count() % QUOTES_PER_PAIR == 0
        {
            return Some(at);
        }
        offset = at.saturating_add(1);
    }

    return None;
}

/// Whether `fields` (a `set ...` line's whitespace-split words) enables nounset in any
/// spelling: the long `-o nounset`, or a short-flag group that contains `u`.
fn Fields_Enable_Nounset(fields: &[&str]) -> bool
{
    for (index, field) in fields.iter().enumerate().skip(1)
    {
        if *field == "-o" && fields.get(index.saturating_add(1)) == Some(&"nounset")
        {
            return true;
        }

        // A short-flag group (`-u`, `-eu`, `-euo`). `-o` introduces a long name and is
        // handled above; `+u` begins with `+` and never reaches this branch.
        let is_short_flag_group_with_u = field.starts_with('-') && !field.starts_with("-o") && field.contains('u');
        if is_short_flag_group_with_u
        {
            return true;
        }
    }

    return false;
}

#[cfg(test)]
mod tests
{
    use super::super::tests::{Source, SourceText};
    use super::*;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_A_Clean_Launcher()
    {
        let source = Source(
            SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh -- set the toolchain and hand off to the real gate\nset -u\nexec ./bin/gate \"$@\"\n" },
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Report_A_Missing_Nounset()
    {
        let source = Source(SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nexec ./bin/gate \"$@\"\n" });

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(EXECUTED_SCRIPTS_SET_NOUNSET));
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_A_Combined_Flag_Group()
    {
        let source = Source(
            SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset -euo pipefail\nexec ./bin/gate \"$@\"\n" },
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Accept_The_Long_Form()
    {
        let source = Source(
            SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset -o nounset\nexec ./bin/gate \"$@\"\n" },
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Report_Nounset_Turned_Off()
    {
        let source = Source(SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh -- hand off to the gate\nset +u\nexec ./bin/gate \"$@\"\n" });

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Exempt_A_Sourced_Library()
    {
        let source = Source(
            SourceText { path: "scripts/toolchain.sh", text: "#!/usr/bin/env bash\n# toolchain.sh -- SOURCE this; do not execute it\nensure_go() {\n command -v go\n}\nrun_gate() {\n exec go run \"$1\"\n}\n" },
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
            SourceText { path: "scripts/toolchain.sh", text: "#!/usr/bin/env bash\n# toolchain.sh -- SOURCE this; do not execute it\ndescribe() {\n printf '%s\n' \"vendored grammar\"\n}\n" },
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "the literal's continuation line must not read as a top-level statement: {findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Treat_A_Top_Level_Call_As_Executed()
    {
        let source = Source(
            SourceText { path: "scripts/build.sh", text: "#!/usr/bin/env bash\n# build.sh\nmain() {\n exec ./bin/gate \"$@\"\n}\nmain \"$@\"\n" },
        );

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert_eq!(findings.len(), 1, "a function call at the top level makes the file executed, not a library: {findings:?}");
    }

    #[test]
    fn Test_Check_Executed_Scripts_Set_Nounset_Should_Ignore_Non_Shebang_Files()
    {
        let source = Source(SourceText { path: "src/lib.rs", text: "fn Check() {}\n" });

        let findings = Check_Executed_Scripts_Set_Nounset(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }
}
