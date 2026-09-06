//! A type alias that names a domain value but adds no type of its own, ported from
//! code-standards' `check-domain-type-alias` (shared package `rules/general/style/shared/
//! domaintypealias`, Rust front end `rust_type_alias.go`, Go front end `go_type_alias.go`).
//!
//! `type EntityId = u64;` tells the compiler nothing: `EntityId` *is* `u64`, the same type
//! wearing a second spelling, so every other `u64` in the program — a tick, an index, a
//! byte count — passes where an entity's identity was meant to go. A distinct type
//! (`struct EntityId(u64);` in Rust, `type EntityId uint64` in Go) costs one line and turns
//! that mixup into a compile error.
//!
//! Four code-standards rule-doc ids name this judgment: a cross-language canonical,
//! `domain-values-are-distinct-types`, plus one companion doc per language. This module
//! reports under the canonical id, the same choice this crate already made for
//! [`crate::Check_Cross_Language_Correspondence`].
//!
//! # What is exempt
//!
//! A generic alias (`type Result<T> = ...;`) or a compound one (`type Handler = fn(i32);`)
//! names no domain value — the right-hand side must be an *exact* bare identifier from a
//! fixed per-language scalar list, never a generic or compound spelling, which is what an
//! alias is actually for. In Rust, an *associated type* — `type Error = u8;` written inside
//! an `impl` or a `trait` body — is a different declaration wearing the same eight
//! characters: the contract named the type, not the author, and firing there would demand
//! a compile error. Go has no such construct, so its side never tracks nesting at all.
//!
//! # This crate's second whole-file brace-depth scan
//!
//! [`super::enum_shape`] established the shape (a pending header committed to an open
//! state once its brace actually arrives, cleared once depth drops back to or below where
//! it opened) for tracking one enum's own depth. This module generalizes it to "is the
//! current line inside any still-open `impl`/`trait` block, whatever else opens and closes
//! around it" — still one `Option<usize>` slot, since Rust does not nest one `impl`/`trait`
//! block inside another in any code this corpus would produce.
//!
//! That scan is not a nicety here: this repository's own source consistently writes an
//! opening brace on its own line after a header (the identical Allman-brace shape
//! `enum_shape.rs`'s own module doc already found necessary against a real corpus), so a
//! same-line-brace assumption would make this rule silently useless against this very
//! workspace, the same failure the real Go tool's own history already records once.
//!
//! No repository-configurable dimension — the Go source's own comment states directly
//! that "there is nothing here for a repository to calibrate" — so a leaf module, not a
//! capability.

use crate::{GO_LANGUAGE, RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// This crate's chosen canonical rule id among the several code-standards ids that name
/// this judgment.
pub const DOMAIN_VALUES_ARE_DISTINCT_TYPES: &str = "domain-values-are-distinct-types";

/// `rust_primitive_aliases` in `rust_type_alias.go`.
const RUST_PRIMITIVE_ALIASES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "bool", "char", "String",
];

/// `go_primitive_aliases` in `go_type_alias.go`.
const GO_PRIMITIVE_ALIASES: &[&str] = &[
    "bool", "string", "int", "int8", "int16", "int32", "int64", "uint", "uint8", "uint16", "uint32", "uint64", "uintptr", "byte", "rune",
    "float32", "float64", "complex64", "complex128",
];

/// One `type Name = Aliased` declaration found by the scan.
struct Alias
{
    line_index: usize,
    contract_bound: bool,
}

/// Reports every type alias whose right-hand side is a bare primitive scalar, in a source
/// declared outside a Rust `impl`/`trait` body (Go has no such exemption).
#[must_use]
pub fn Check_Domain_Values_Are_Distinct_Types(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Own_Implementation_File(source)
        {
            continue;
        }

        let source_findings = Alias_Findings_For_Source(source);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// This file's own path. Every fixture below spells a real alias/impl/trait shape inside a
/// Rust string literal, which would otherwise self-match when this crate checks its own
/// workspace — the same self-exemption every other `*_text.rs`-shaped rule here carries for
/// the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "crates/rules/nomos-rules/src/checks/domain_type_alias.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/") == OWN_IMPLEMENTATION_FILE;
}

fn Alias_Findings_For_Source(source: &SourceFile) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();

    if source.Is_Written_In(RUST_LANGUAGE)
    {
        let aliases = Rust_Aliases_In(&lines);
        return Alias_Findings_In(source, &aliases);
    }

    if source.Is_Written_In(GO_LANGUAGE)
    {
        let aliases = Go_Aliases_In(&lines);
        return Alias_Findings_In(source, &aliases);
    }

    return Vec::new();
}

/// Threaded across [`Rust_Aliases_In`]'s one-pass scan: how deep into nested braces the
/// current line sits, and whether that depth is inside a still-open `impl`/`trait` body.
struct RustAliasScan
{
    depth: usize,
    contract_open: Option<usize>,
    pending_contract: bool,
}

/// Rust needs the whole-file scan: an alias is judged everywhere it appears, including
/// inside a `mod` block, but `contract_bound` depends on whether it sits inside a still-
/// open `impl`/`trait` body.
fn Rust_Aliases_In(lines: &[&str]) -> Vec<Alias>
{
    let mut found = Vec::new();
    let mut scan = RustAliasScan { depth: 0, contract_open: None, pending_contract: false };

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(alias) = Rust_Alias_At_Line(line, index, &mut scan)
        {
            found.push(alias);
        }
    }

    return found;
}

/// Judges one line against the accumulated `scan` state, then advances that state past it —
/// a comment or attribute line is judged and skipped without advancing the brace depth at
/// all, matching this module's "no struct-body boundary tracking" simplification.
fn Rust_Alias_At_Line(line: &str, index: usize, scan: &mut RustAliasScan) -> Option<Alias>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("#[")
    {
        return None;
    }

    if Starts_A_New_Contract_Header(scan, line)
    {
        scan.pending_contract = true;
    }

    let alias = Rust_Type_Alias_Match(trimmed)
        .filter(|aliased| return Is_Primitive(aliased, RUST_PRIMITIVE_ALIASES))
        .map(|_| return Alias { line_index: index, contract_bound: scan.contract_open.is_some() });

    Advance_Rust_Alias_Scan(line, scan);

    return alias;
}

/// This line opens a new `impl`/`trait` body worth tracking: none is already open or about
/// to open, and this line is itself the header.
fn Starts_A_New_Contract_Header(scan: &RustAliasScan, line: &str) -> bool
{
    return scan.contract_open.is_none() && !scan.pending_contract && Is_Contract_Header(line);
}

/// Whether `impl` or `trait` appears as a whole word on this line — a Rust `impl`/`trait`
/// header may carry generics or a `for` clause before its brace, so this only needs to spot
/// the keyword, not parse the rest.
fn Is_Contract_Header(line: &str) -> bool
{
    return Contains_Word(line, Word("impl")) || Contains_Word(line, Word("trait"));
}

/// The literal word [`Contains_Word`] searches for, wrapped so its parameter position
/// cannot be transposed with `line` — the text being searched — with nothing to catch it.
struct Word<'a>(&'a str);

fn Contains_Word(line: &str, word: Word<'_>) -> bool
{
    let bytes = line.as_bytes();
    let mut search_from = 0usize;

    while let Some(offset) = line.get(search_from..).and_then(|rest| return rest.find(word.0))
    {
        let start = search_from.saturating_add(offset);
        let end = start.saturating_add(word.0.len());

        if Has_Left_Boundary(bytes, start) && Right_Boundary_Ends_The_Word(line, end)
        {
            return true;
        }

        search_from = start.saturating_add(1);
    }

    return false;
}

fn Has_Left_Boundary(bytes: &[u8], start: usize) -> bool
{
    return start.checked_sub(1).and_then(|previous| return bytes.get(previous)).is_none_or(|&byte| return !Is_Ident_Byte(byte));
}

fn Is_Ident_Byte(byte: u8) -> bool
{
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// Whether `end` (the byte offset just past a candidate word match) sits at a word boundary
/// — the line runs out there, or the next character does not continue an identifier.
fn Right_Boundary_Ends_The_Word(line: &str, end: usize) -> bool
{
    return line.get(end..).is_none_or(|rest| return !rest.starts_with(Is_Ident_Char));
}

/// `type Name = Aliased;` (optionally `pub`/`pub(...)`-qualified), single line, ported as a
/// hand match rather than a `regex` dependency this crate has never taken on. Returns the
/// right-hand side, trimmed, with generics or compound spellings left exactly as written so
/// [`Is_Primitive`]'s exact-match check can tell a bare scalar from anything else.
fn Rust_Type_Alias_Match(trimmed: &str) -> Option<String>
{
    let code_owned = super::code_prefix::Code_Prefix(trimmed);
    let code = code_owned.trim();
    let after_visibility = Strip_Rust_Visibility(code);
    let after_type = after_visibility.strip_prefix("type ")?.trim_start();
    let (name_and_generics, rest) = after_type.split_once('=')?;

    let name = name_and_generics.trim_end().split('<').next().unwrap_or("");
    if name.is_empty() || !Is_Valid_Identifier(name)
    {
        return None;
    }

    let aliased = rest.trim().trim_end().strip_suffix(';')?.trim();
    if aliased.is_empty()
    {
        return None;
    }

    return Some(aliased.to_owned());
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

fn Advance_Rust_Alias_Scan(line: &str, scan: &mut RustAliasScan)
{
    let opened = line.matches('{').count();
    let closed = line.matches('}').count();

    if scan.pending_contract && opened > 0
    {
        scan.contract_open = Some(scan.depth);
        scan.pending_contract = false;
    }

    scan.depth = scan.depth.saturating_add(opened);
    scan.depth = scan.depth.saturating_sub(closed);

    if scan.contract_open.is_some_and(|open_depth| return scan.depth <= open_depth)
    {
        scan.contract_open = None;
    }
}

fn Alias_Findings_In(source: &SourceFile, aliases: &[Alias]) -> Vec<Finding>
{
    return aliases
        .iter()
        .filter(|alias| return !alias.contract_bound)
        .map(|alias| {
            let line_number = Line_Number(alias.line_index);
            return Finding {
                rule: RuleId::New(DOMAIN_VALUES_ARE_DISTINCT_TYPES),
                subject: source.subject,
                subject_name: format!("{}:{line_number}", source.path),
                applicability: Applicability::Supported,
                evidence: EvidenceClass::Derived,
                gate: GateCategory::Blocking,
                summary: format!(
                    "the type alias at {}:{line_number} names a domain value but does not type it; give it a distinct type instead",
                    source.path
                ),
                locations: vec![format!("{}:{line_number}", source.path)],
            };
        })
        .collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// Go never sets `contract_bound` — the language has no associated-type construct, so an
/// alias inside a function body or a generic constraint is still the author's own choice.
fn Go_Aliases_In(lines: &[&str]) -> Vec<Alias>
{
    let mut found = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("//")
        {
            continue;
        }

        if Go_Type_Alias_Match(trimmed).is_some_and(|aliased| return Is_Primitive(&aliased, GO_PRIMITIVE_ALIASES))
        {
            found.push(Alias { line_index: index, contract_bound: false });
        }
    }

    return found;
}

/// `type Name = Aliased` — Go never terminates a declaration with a semicolon, and the
/// presence of the `=` is the whole distinction from a defined type (`type Name Aliased`,
/// which never matches here at all).
fn Go_Type_Alias_Match(trimmed: &str) -> Option<String>
{
    let code_owned = super::code_prefix::Code_Prefix(trimmed);
    let code = code_owned.trim();
    let after_type = code.strip_prefix("type ")?.trim_start();
    let (name, rest) = after_type.split_once('=')?;

    let name = name.trim();
    if name.is_empty() || !Is_Valid_Identifier(name)
    {
        return None;
    }

    let aliased = rest.trim();
    if aliased.is_empty() || !Is_Valid_Identifier(aliased)
    {
        return None;
    }

    return Some(aliased.to_owned());
}

fn Is_Ident_Char(character: char) -> bool
{
    return character.is_alphanumeric() || character == '_';
}

fn Is_Primitive(aliased: &str, vocabulary: &[&str]) -> bool
{
    return vocabulary.contains(&aliased);
}

fn Is_Valid_Identifier(name: &str) -> bool
{
    return name.chars().all(Is_Ident_Char);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Report_A_Rust_Primitive_Alias()
    {
        let source = Source("src/entity.rs", "type EntityId = u64;\n");
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(DOMAIN_VALUES_ARE_DISTINCT_TYPES));
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Ignore_A_Generic_Alias()
    {
        let source = Source("src/entity.rs", "type Result<T> = std::result::Result<T, Error>;\n");
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert!(findings.is_empty(), "a generic alias abbreviates a shape, not a scalar: {findings:?}");
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Ignore_A_Compound_Alias()
    {
        let source = Source("src/entity.rs", "type Handler = fn(i32);\n");
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Ignore_An_Associated_Type_With_An_Allman_Brace()
    {
        let source = Source(
            "src/codec.rs",
            "impl Encode for Packet\n{\n    type Error = u8;\n}\n",
        );
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert!(findings.is_empty(), "an associated type is the trait's own vocabulary: {findings:?}");
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Still_Judge_An_Alias_After_The_Impl_Closes()
    {
        let source = Source(
            "src/codec.rs",
            "impl Encode for Packet\n{\n    type Error = u8;\n}\n\ntype Ticket = u64;\n",
        );
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert_eq!(findings.len(), 1, "only the alias outside the impl fires: {findings:?}");
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Report_A_Go_Primitive_Alias()
    {
        let source = Source("entity.go", "type EntityId = uint64\n");
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_Domain_Values_Are_Distinct_Types_Should_Ignore_A_Go_Defined_Type()
    {
        let source = Source("entity.go", "type EntityId uint64\n");
        let findings = Check_Domain_Values_Are_Distinct_Types(&[source]);

        assert!(findings.is_empty(), "no equals sign means this is the remedy, not the defect: {findings:?}");
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}
