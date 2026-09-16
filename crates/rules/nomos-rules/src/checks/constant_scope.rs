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

use crate::checks::declaration_scan::{
    Brace_Delta, DeclarationBlock, Is_Ident_Char, Line_Number, Name_After_Keyword,
};
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

        let source_findings = Constant_Findings_For_Source(source);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// This file's own path. Every fixture below spells a real const/fn/impl shape inside a
/// Rust string literal, which would otherwise self-match when this crate checks its own
/// workspace — the same self-exemption every other `*_text.rs`-shaped rule here carries for
/// the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "crates/rules/nomos-rules/src/checks/constant_scope.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/") == OWN_IMPLEMENTATION_FILE;
}

fn Constant_Findings_For_Source(source: &SourceFile) -> Vec<Finding>
{
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
        return Vec::new();
    };

    return constants.into_iter().map(|constant| return Constant_Finding(source, &constant)).collect();
}

/// A stack frame for one open Rust function: the depth it opened at, and its name.
struct FunctionFrame
{
    open_depth: usize,
    name: String,
}

/// Threaded across [`Rust_Function_Constants_In`]'s one-pass scan: how deep into nested
/// braces the current line sits, the stack of still-open function frames, and a function
/// header seen but not yet committed (its brace has not arrived).
struct RustConstantScan
{
    depth: usize,
    stack: Vec<FunctionFrame>,
    pending: Option<String>,
}

fn Rust_Function_Constants_In(lines: &[&str]) -> Vec<Constant>
{
    let mut found = Vec::new();
    let mut scan = RustConstantScan { depth: 0, stack: Vec::new(), pending: None };

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(constant) = Rust_Constant_At_Line(line, index, &mut scan)
        {
            found.push(constant);
        }
    }

    return found;
}

/// Judges one line against the accumulated `scan` state, then advances that state past it.
fn Rust_Constant_At_Line(line: &str, index: usize, scan: &mut RustConstantScan) -> Option<Constant>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("#[")
    {
        return None;
    }

    if scan.pending.is_none()
    {
        scan.pending = Fn_Header_Name(line);
    }

    let mut constant = None;
    if let Some(function) = scan.stack.last()
        && Rust_Const_Name(trimmed).is_some()
    {
        constant = Some(Constant { line_index: index, function: function.name.clone() });
    }

    Advance_Rust_Constant_Scan(line, scan);

    return constant;
}

/// `\bfn\s+(\w+)` ported as a hand search — the same left-boundary-plus-mandatory-
/// whitespace shape `enum_shape.rs`'s own header search already established, so a function
/// pointer type (`fn(i32) -> i32`, no whitespace before `(`) never matches.
fn Fn_Header_Name(line: &str) -> Option<String>
{
    return Keyword_Header_Name(line, Keyword("fn"));
}

/// A Rust `const NAME: Type = ...;` declaration, ported as a hand match: the token right
/// after `const` (optionally `pub`/`pub(...)`-qualified) must be an identifier immediately
/// followed by `:`, which excludes `const fn` — a function modifier, not a value
/// declaration, since `fn` is never followed directly by a colon.
fn Rust_Const_Name(trimmed: &str) -> Option<String>
{
    let code_owned = super::code_prefix::Code_Prefix(trimmed);
    let code = code_owned.trim();
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

    return Some(name.to_owned());
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

fn Advance_Rust_Constant_Scan(line: &str, scan: &mut RustConstantScan)
{
    let (opened, closed) = Brace_Delta(line);

    if let Some(name) = scan.pending.take_if(|_| return opened > 0)
    {
        scan.stack.push(FunctionFrame { open_depth: scan.depth, name });
    }

    scan.depth = scan.depth.saturating_add(opened);
    scan.depth = scan.depth.saturating_sub(closed);

    while scan.stack.last().is_some_and(|frame| return scan.depth <= frame.open_depth)
    {
        scan.stack.pop();
    }
}

/// Threaded across [`Go_Function_Constants_In`]'s one-pass scan.
///
/// The block tracking is [`DeclarationBlock`]'s; `in_const_block` is the one thing this
/// scan carries that the enum scan next door does not, and it is why the block is a field
/// here rather than the whole struct being replaced.
struct GoConstantScan
{
    block: DeclarationBlock,
    in_const_block: bool,
}

fn Go_Function_Constants_In(lines: &[&str]) -> Vec<Constant>
{
    let mut found = Vec::new();
    let mut scan = GoConstantScan { block: DeclarationBlock::New(), in_const_block: false };

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(constant) = Go_Constant_At_Line(line, index, &mut scan)
        {
            found.push(constant);
        }
    }

    return found;
}

/// Judges one line against the accumulated `scan` state, then advances that state past it.
fn Go_Constant_At_Line(line: &str, index: usize, scan: &mut GoConstantScan) -> Option<Constant>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
    {
        return None;
    }

    if Ready_For_A_New_Go_Function(scan)
    {
        scan.block.pending = Go_Func_Header_Name(line);
    }

    let constant = Go_Constant_In_Open_Function(trimmed, index, scan);

    if scan.block.Advance(line)
    {
        scan.in_const_block = false;
    }

    return constant;
}

/// No function is currently open or about to open, and the scan sits at the file's own top
/// level — the only place a new Go function header is looked for.
fn Ready_For_A_New_Go_Function(scan: &GoConstantScan) -> bool
{
    return scan.block.open.is_none() && scan.block.pending.is_none() && scan.block.depth == 0;
}

/// `\bfunc\s+(\w+)` — Go's own function keyword is spelled differently but the shape is
/// identical; a func literal (`func(x int) { ... }`) has no name between the keyword and
/// `(`, so it never matches and never opens a new scope.
fn Go_Func_Header_Name(line: &str) -> Option<String>
{
    return Keyword_Header_Name(line, Keyword("func"));
}

/// `trimmed`'s constant, if `scan` is inside an open function and this line declares one —
/// guard clauses throughout, since each case here is its own reason to stop, not a nested
/// refinement of the one before it.
fn Go_Constant_In_Open_Function(trimmed: &str, index: usize, scan: &mut GoConstantScan) -> Option<Constant>
{
    let (_, function) = scan.block.open.as_ref()?;
    let function = function.clone();

    if scan.in_const_block
    {
        return Go_Constant_In_Const_Block(trimmed, index, scan, function);
    }

    if trimmed == "const ("
    {
        scan.in_const_block = true;
        return None;
    }

    if Go_Const_Name(trimmed).is_some()
    {
        return Some(Constant { line_index: index, function });
    }

    return None;
}

/// One member line inside an already-open `const ( ... )` block: the block's own close,
/// a blank line, or a member that carries no `const` keyword of its own.
fn Go_Constant_In_Const_Block(trimmed: &str, index: usize, scan: &mut GoConstantScan, function: String) -> Option<Constant>
{
    if trimmed == ")"
    {
        scan.in_const_block = false;
        return None;
    }
    if trimmed.is_empty()
    {
        return None;
    }

    return Some(Constant { line_index: index, function });
}

/// A Go single-line `const name = value` or `const name Type = value` declaration (the
/// parenthesized block form is handled separately by the scan's own `in_const_block`
/// state, since each member line inside it carries no `const` keyword of its own).
fn Go_Const_Name(trimmed: &str) -> Option<String>
{
    let code_owned = super::code_prefix::Code_Prefix(trimmed);
    let code = code_owned.trim();
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

    return Some(name.to_owned());
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

/// The literal keyword [`Keyword_Header_Name`] searches for, wrapped so its parameter
/// position cannot be transposed with `line` — the text being searched — with nothing to
/// catch it.
struct Keyword<'a>(&'a str);

fn Keyword_Header_Name(line: &str, keyword: Keyword<'_>) -> Option<String>
{
    let bytes = line.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find(keyword.0))
    {
        let start = search_from.saturating_add(offset);
        let end = start.saturating_add(keyword.0.len());

        if let Some(name) = Name_After_Keyword(line, bytes, start, end)
        {
            return Some(name.to_owned());
        }

        search_from = start.saturating_add(1);
    }

    return None;
}

#[cfg(test)]
mod tests;
