//! A named constant declared inside the function that reads it, ported from
//! code-standards' `check-constant-scope` (shared package `rules/general/architecture/
//! shared/constantscope`, Rust front end `rust_constant_scope.go`, Go front end
//! `go_constant_scope.go`).
//!
//! A binding's scope is a claim about a *lifetime*; a constant has none — it exists at
//! compile time. What a constant's scope claims is where the value is *authoritative*, and
//! one declared inside the single function that reads it today says "this number belongs
//! to this function," which is almost always false: the next caller does not think to
//! reach into another function's body, and copies the literal instead. Module scope (and,
//! in Rust, an associated const inside an `impl`) is where a constant answers for itself.
//!
//! One rule-doc id, no per-language companions, no repository-configurable dimension at
//! all — the shared package's own doc comment states directly that it "needs no
//! calibration," so this is a leaf module, not a capability.
//!
//! # This crate's first stack-based brace-depth scan
//!
//! [`super::enum_shape`] and [`super::domain_type_alias`] each track a single open
//! construct with one `Option<usize>` slot, because neither an enum body nor an
//! `impl`/`trait` body nests in code this corpus would produce. A Rust function *does*
//! nest — a named `fn` can be declared inside another `fn`'s body — so this scan carries a
//! stack of open function frames instead of one slot.
//!
//! # One deliberate divergence from the real tool's own behavior
//!
//! `rust_Collect_Function_Constants` walks the *whole* tree for every function item,
//! nested ones included, and then walks each one's own body independently. A constant
//! inside a doubly-nested function is therefore reported once per enclosing function level
//! in the real tool — which reads as a side effect of an unconditional tree walk, not a
//! design the rule's own doc argues for anywhere. This port tracks a real stack (needed
//! regardless, to know whether a line sits inside any function at all) but attributes each
//! finding to only the innermost open frame — a documented narrowing, not a literal replay
//! of what looks like a tool quirk.
//!
//! Go needs no stack at all: Go has no nested named function declarations (only anonymous
//! closures, which this scan's header search cannot match, since it requires a name), so a
//! single optional slot suffices, gated to only start looking for a new header at brace
//! depth zero — the same restriction `go_Over_Function_Bodies` gets for free by walking
//! only `parsed.Decls`, the file's own top level.

use crate::{GO_LANGUAGE, RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE: &str = "constants-are-the-exception-to-function-scope-use";

/// One constant found inside a function body, tagged with the name of the (innermost)
/// enclosing function.
struct Constant
{
    line_index: usize,
    function: String,
}

/// Reports every constant declared inside a function body, in Rust or Go source.
#[must_use]
pub fn Check_Constants_Are_The_Exception_To_Function_Scope_Use(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Own_Implementation_File(source)
        {
            continue;
        }

        let lines: Vec<&str> = source.text.lines().collect();
        let constants = if source.Is_Written_In(RUST_LANGUAGE)
        {
            Rust_Function_Constants_In(&lines)
        }
        else if source.Is_Written_In(GO_LANGUAGE)
        {
            Go_Function_Constants_In(&lines)
        }
        else
        {
            continue;
        };

        findings.extend(constants.into_iter().map(|constant| return Constant_Finding(source, &constant)));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Constant_Finding(source: &SourceFile, constant: &Constant) -> Finding
{
    let line_number = Line_Number(constant.line_index);
    return Finding {
        rule: RuleId::New(CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!(
            "the constant at {}:{line_number} is declared inside `{}`, which claims the value belongs to that function; lift it to module scope",
            source.path, constant.function
        ),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

/// A stack frame for one open Rust function: the depth it opened at, and its name.
struct FunctionFrame
{
    open_depth: usize,
    name: String,
}

fn Rust_Function_Constants_In(lines: &[&str]) -> Vec<Constant>
{
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut stack: Vec<FunctionFrame> = Vec::new();
    let mut pending: Option<String> = None;

    for (index, line) in lines.iter().enumerate()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("#[")
        {
            continue;
        }

        if pending.is_none()
        {
            pending = Fn_Header_Name(line);
        }

        if let Some(function) = stack.last()
            && Rust_Const_Name(trimmed).is_some()
        {
            found.push(Constant { line_index: index, function: function.name.clone() });
        }

        let opened = line.matches('{').count();
        let closed = line.matches('}').count();

        if let Some(name) = pending.take_if(|_| return opened > 0)
        {
            stack.push(FunctionFrame { open_depth: depth, name });
        }

        depth = depth.saturating_add(opened);
        depth = depth.saturating_sub(closed);

        while stack.last().is_some_and(|frame| return depth <= frame.open_depth)
        {
            stack.pop();
        }
    }

    return found;
}

fn Go_Function_Constants_In(lines: &[&str]) -> Vec<Constant>
{
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut open: Option<(usize, String)> = None;
    let mut pending: Option<String> = None;
    let mut in_const_block = false;

    for (index, line) in lines.iter().enumerate()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("//")
        {
            continue;
        }

        if open.is_none() && pending.is_none() && depth == 0
        {
            pending = Go_Func_Header_Name(line);
        }

        if let Some((_, function)) = &open
        {
            if in_const_block
            {
                if trimmed == ")"
                {
                    in_const_block = false;
                }
                else if !trimmed.is_empty()
                {
                    found.push(Constant { line_index: index, function: function.clone() });
                }
            }
            else if trimmed == "const ("
            {
                in_const_block = true;
            }
            else if Go_Const_Name(trimmed).is_some()
            {
                found.push(Constant { line_index: index, function: function.clone() });
            }
        }

        let opened = line.matches('{').count();
        let closed = line.matches('}').count();

        if let Some(name) = pending.take_if(|_| return opened > 0)
        {
            open = Some((depth, name));
        }

        depth = depth.saturating_add(opened);
        depth = depth.saturating_sub(closed);

        if open.as_ref().is_some_and(|(open_depth, _)| return depth <= *open_depth)
        {
            open = None;
            in_const_block = false;
        }
    }

    return found;
}

/// `\bfn\s+(\w+)` ported as a hand search — the same left-boundary-plus-mandatory-
/// whitespace shape `enum_shape.rs`'s own header search already established, so a function
/// pointer type (`fn(i32) -> i32`, no whitespace before `(`) never matches.
fn Fn_Header_Name(line: &str) -> Option<String>
{
    Keyword_Header_Name(line, "fn")
}

/// `\bfunc\s+(\w+)` — Go's own function keyword is spelled differently but the shape is
/// identical; a func literal (`func(x int) { ... }`) has no name between the keyword and
/// `(`, so it never matches and never opens a new scope.
fn Go_Func_Header_Name(line: &str) -> Option<String>
{
    Keyword_Header_Name(line, "func")
}

fn Keyword_Header_Name(line: &str, keyword: &str) -> Option<String>
{
    let bytes = line.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find(keyword))
    {
        let start = search_from.saturating_add(offset);
        let end = start.saturating_add(keyword.len());

        if Has_Left_Boundary(bytes, start)
            && let Some(after) = line.get(end..)
            && after.starts_with(char::is_whitespace)
        {
            let trimmed = after.trim_start();
            let name_len = trimmed.find(|character: char| return !(character.is_alphanumeric() || character == '_')).unwrap_or(trimmed.len());
            if name_len > 0
            {
                return trimmed.get(..name_len).map(str::to_owned);
            }
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

fn Is_Ident_Char(character: char) -> bool
{
    return character.is_alphanumeric() || character == '_';
}

/// A Rust `const NAME: Type = ...;` declaration, ported as a hand match: the token right
/// after `const` (optionally `pub`/`pub(...)`-qualified) must be an identifier immediately
/// followed by `:`, which excludes `const fn` — a function modifier, not a value
/// declaration, since `fn` is never followed directly by a colon.
fn Rust_Const_Name(trimmed: &str) -> Option<&str>
{
    let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
    let after_visibility = Strip_Rust_Visibility(code);
    let after_const = after_visibility.strip_prefix("const ")?.trim_start();
    let name_len = after_const.find(|character: char| return !Is_Ident_Char(character)).unwrap_or(after_const.len());
    let name = after_const.get(..name_len)?;
    if name.is_empty()
    {
        return None;
    }

    let rest = after_const.get(name_len..)?.trim_start();
    if !rest.starts_with(':')
    {
        return None;
    }

    return Some(name);
}

fn Strip_Rust_Visibility(code: &str) -> &str
{
    let Some(after_pub) = code.strip_prefix("pub") else { return code };

    if let Some(after_paren) = after_pub.strip_prefix('(')
        && let Some(close) = after_paren.find(')')
    {
        return after_paren.get(close.saturating_add(1)..).unwrap_or("").trim_start();
    }

    return after_pub.trim_start();
}

/// A Go single-line `const name = value` or `const name Type = value` declaration (the
/// parenthesized block form is handled separately by the scan's own `in_const_block`
/// state, since each member line inside it carries no `const` keyword of its own).
fn Go_Const_Name(trimmed: &str) -> Option<&str>
{
    let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
    let after_const = code.strip_prefix("const ")?.trim_start();
    if after_const.starts_with('(')
    {
        return None;
    }

    let name_len = after_const.find(|character: char| return !Is_Ident_Char(character)).unwrap_or(after_const.len());
    let name = after_const.get(..name_len)?;
    if name.is_empty() || name == "_"
    {
        return None;
    }

    return Some(name);
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// This file's own path. Every fixture below spells a real const/fn/impl shape inside a
/// Rust string literal, which would otherwise self-match when this crate checks its own
/// workspace — the same self-exemption every other `*_text.rs`-shaped rule here carries for
/// the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "checks/constant_scope.rs";

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

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Rust_Const_Inside_A_Function()
    {
        let source = Source("src/timer.rs", "fn tick()\n{\n    const FRAME_INTERVAL_MS: u32 = 16;\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(CONSTANTS_ARE_THE_EXCEPTION_TO_FUNCTION_SCOPE_USE));
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_A_Module_Scope_Const()
    {
        let source = Source("src/timer.rs", "const FRAME_INTERVAL_MS: u32 = 16;\n\nfn tick()\n{\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert!(findings.is_empty(), "module scope is where constants live: {findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_An_Associated_Const()
    {
        let source = Source("src/timer.rs", "impl Timer\n{\n    const FRAME_INTERVAL_MS: u32 = 16;\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert!(findings.is_empty(), "an associated const belongs to the type, not any function: {findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Not_Mistake_A_Const_Fn_For_A_Value()
    {
        let source = Source("src/timer.rs", "fn outer()\n{\n    const fn helper() -> u32 { 16 }\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert!(findings.is_empty(), "const fn is a function modifier, not a value declaration: {findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Doubly_Nested_Const_Only_Once()
    {
        let source = Source(
            "src/timer.rs",
            "fn outer()\n{\n    fn inner()\n    {\n        const FRAME_INTERVAL_MS: u32 = 16;\n    }\n}\n",
        );
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert_eq!(findings.len(), 1, "attributed to the innermost function only, not once per nesting level: {findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Go_Const_Inside_A_Function()
    {
        let source = Source("timer.go", "func Tick() {\n\tconst frameIntervalMs = 16\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Ignore_A_Go_Package_Scope_Const()
    {
        let source = Source("timer.go", "const frameIntervalMs = 16\n\nfunc Tick() {\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Constants_Are_The_Exception_To_Function_Scope_Use_Should_Report_A_Go_Const_Block_Inside_A_Function()
    {
        let source = Source("timer.go", "func Tick() {\n\tconst (\n\t\tframeIntervalMs = 16\n\t\tmaxFrames = 60\n\t)\n}\n");
        let findings = Check_Constants_Are_The_Exception_To_Function_Scope_Use(&[source]);

        assert_eq!(findings.len(), 2, "{findings:?}");
    }
}
