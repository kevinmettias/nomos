//! A struct field whose validated range contradicts the type storing it, ported from
//! code-standards' `check-scalar-range` (shared package `rules/general/style/shared/
//! scalarrange`, Rust front end `rust_scalar_range.go`).
//!
//! `#[validate(range(min = 0, max = 255))] port: i64,` states two things about `port` — its
//! range, and its type — and they disagree: nothing the range permits requires 64 signed
//! bits, and nothing in it can go negative. The rule is that a number known to sit inside a
//! range is stored in the narrowest type that holds it, and in an unsigned type when the
//! range cannot go below zero.
//!
//! Two code-standards rule-doc ids name the two distinct predicates —
//! [`NONNEGATIVE_STORAGE_IS_UNSIGNED`] and [`A_KNOWN_RANGE_PICKS_ITS_TYPE`] — and
//! code-standards' own finding model carries no per-doc rule id at all (a tool's
//! `Enforces:` line is the only place multiple ids are named). This port is two separate
//! functions, one per predicate, the same choice `concurrency_text.rs`'s three functions
//! already made reading one shared value: a field violating both predicates gets two
//! separate findings, not one merged one.
//!
//! No repository-configurable dimension — the bound is always the author's own
//! already-written attribute, never a default a repository would state differently — so a
//! leaf module, not a capability.
//!
//! # Scoped to Rust only this increment
//!
//! Go's companion front end reads a different `validate:"min=N,max=N"` struct-tag string
//! syntax that needs its own separate parsing, not a trivial extension of the
//! attribute-clause reader here, and is left as a natural follow-up.
//!
//! # A simplification found to be safe, not merely convenient
//!
//! The real front end pairs an attribute with the field beneath it by sibling position
//! inside a parsed `field_declaration_list`. This port approximates that as a linear scan
//! with no struct-body boundary tracking at all: a `#[validate(range(...))]` attribute is
//! accumulated across consecutive attribute lines and consumed by the next non-attribute,
//! non-blank line, wherever in the file that happens. A function parameter list formatted
//! one argument per line has the identical `name: Type,` shape a struct field does, but
//! nobody writes this specific attribute above a parameter — the vocabulary itself is
//! narrow enough that the boundary tracking the real parser buys is not needed here.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards non-negative-storage rule id.
pub const NONNEGATIVE_STORAGE_IS_UNSIGNED: &str = "nonnegative-storage-is-unsigned";
/// The code-standards known-range rule id.
pub const A_KNOWN_RANGE_PICKS_ITS_TYPE: &str = "a-known-range-picks-its-type";

/// The widest integer this rule reasons about — bounds are parsed as `i64`, so a 64-bit
/// candidate holds any value they can carry.
const MAX_SCALAR_BITS: u32 = 64;

/// `rust_declared` in `rust_scalar_range.go`: every scalar a field may be declared as.
/// `(spelling, bits, signed, is_float)`.
const RUST_DECLARED_SCALARS: &[(&str, u32, bool, bool)] = &[
    ("u8", 8, false, false),
    ("i8", 8, true, false),
    ("u16", 16, false, false),
    ("i16", 16, true, false),
    ("u32", 32, false, false),
    ("i32", 32, true, false),
    ("u64", 64, false, false),
    ("i64", 64, true, false),
    ("usize", 16, false, false),
    ("isize", 16, true, false),
    ("u128", 128, false, false),
    ("i128", 128, true, false),
    ("f32", 32, true, true),
    ("f64", 64, true, true),
];

/// `rust_remedies` in `rust_scalar_range.go`: the fixed-width candidates a remedy may be
/// drawn from. `(spelling, bits, signed)`.
const RUST_REMEDIES: &[(&str, u32, bool)] =
    &[("u8", 8, false), ("i8", 8, true), ("u16", 16, false), ("i16", 16, true), ("u32", 32, false), ("i32", 32, true), ("u64", 64, false), ("i64", 64, true)];

/// One struct field a `#[validate(range(...))]` attribute stated a bound for.
struct Bound
{
    line_index: usize,
    member: String,
    declared_spelling: &'static str,
    declared_bits: u32,
    declared_signed: bool,
    min: Option<i64>,
    max: Option<i64>,
}

/// Reports a signed field whose stated minimum cannot go below zero.
#[must_use]
pub fn Check_Nonnegative_Storage_Is_Unsigned(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, NONNEGATIVE_STORAGE_IS_UNSIGNED, |bound| {
        return bound.declared_signed && bound.min.is_some_and(|min| return min >= 0);
    });
}

/// Reports a field wider than its stated range needs.
#[must_use]
pub fn Check_A_Known_Range_Picks_Its_Type(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, A_KNOWN_RANGE_PICKS_ITS_TYPE, |bound| {
        let (Some(min), Some(max)) = (bound.min, bound.max) else { return false };
        return Narrowest_That_Holds(min, max).is_some_and(|(_, bits, _)| return bits < bound.declared_bits);
    });
}

fn Findings_For(sources: &[SourceFile], rule: &str, matches: fn(&Bound) -> bool) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            let lines: Vec<&str> = source.text.lines().collect();
            for bound in Bounded_Fields_In(&lines)
            {
                if matches(&bound)
                {
                    findings.push(Bound_Finding(source, rule, &bound));
                }
            }
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Bound_Finding(source: &SourceFile, rule: &str, bound: &Bound) -> Finding
{
    let line_number = Line_Number(bound.line_index);
    let span = match (bound.min, bound.max)
    {
        (Some(min), Some(max)) => format!("{min}..={max}"),
        (Some(min), None) => format!("{min} and above"),
        (None, Some(max)) => format!("{max} and below"),
        (None, None) => "an unstated range".to_owned(),
    };
    let summary = if let Some((remedy, remedy_bits, _)) = Narrowest_That_Holds(bound.min.unwrap_or(0), bound.max.unwrap_or(0))
        && bound.min.is_some()
        && bound.max.is_some()
        && remedy_bits < bound.declared_bits
    {
        format!(
            "`{}` at {}:{line_number} stays within {span}, but is stored in `{}` ({} bits) for a range that needs {remedy}",
            bound.member, source.path, bound.declared_spelling, bound.declared_bits
        )
    }
    else
    {
        format!(
            "`{}` at {}:{line_number} stays within {span}, which cannot go below zero, but is stored in the signed type `{}`",
            bound.member, source.path, bound.declared_spelling
        )
    };

    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary,
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

/// Picks the cheapest candidate in [`RUST_REMEDIES`] that holds `[min, max]` — unsigned
/// preferred at equal width, ported from `Narrowest_That_Holds`/`is_cheaper`.
fn Narrowest_That_Holds(min: i64, max: i64) -> Option<(&'static str, u32, bool)>
{
    let mut best: Option<(&'static str, u32, bool)> = None;

    for &(spelling, bits, signed) in RUST_REMEDIES
    {
        if !Can_Hold_Range(bits, signed, min, max)
        {
            continue;
        }

        best = match best
        {
            None => Some((spelling, bits, signed)),
            Some((_, best_bits, best_signed)) if bits < best_bits || (bits == best_bits && !signed && best_signed) => Some((spelling, bits, signed)),
            keep => keep,
        };
    }

    return best;
}

/// Ported from `can_Hold_Range`, using `i128` intermediates so the ceiling computation
/// never overflows for any width up to [`MAX_SCALAR_BITS`].
fn Can_Hold_Range(bits: u32, signed: bool, min: i64, max: i64) -> bool
{
    if bits == 0 || bits > MAX_SCALAR_BITS
    {
        return false;
    }

    if !signed
    {
        if min < 0
        {
            return false;
        }
        if bits == MAX_SCALAR_BITS
        {
            return true;
        }
        let ceiling: i128 = (1i128 << bits).saturating_sub(1);
        return i128::from(max) <= ceiling;
    }

    if bits == MAX_SCALAR_BITS
    {
        return true;
    }
    let ceiling: i128 = (1i128 << bits.saturating_sub(1)).saturating_sub(1);
    return i128::from(min) >= ceiling.saturating_neg().saturating_sub(1) && i128::from(max) <= ceiling;
}

/// The linear attribute-then-field scan: a validate-range attribute is accumulated across
/// consecutive attribute lines and consumed (whether or not it pairs with a real field) by
/// the next non-attribute, non-blank line.
fn Bounded_Fields_In(lines: &[&str]) -> Vec<Bound>
{
    let mut found = Vec::new();
    let mut pending: Option<(Option<i64>, Option<i64>)> = None;

    for (index, line) in lines.iter().enumerate()
    {
        let trimmed = line.trim();
        if trimmed.is_empty()
        {
            continue;
        }

        if trimmed.starts_with("#[")
        {
            if let Some(range) = Range_Attribute(trimmed)
            {
                pending = Some(range);
            }
            continue;
        }

        let Some((min, max)) = pending.take()
        else
        {
            continue;
        };

        if let Some((name, declared_type)) = Field_Declaration(trimmed)
            && let Some(&(spelling, bits, signed, is_float)) = RUST_DECLARED_SCALARS.iter().find(|&&(candidate, ..)| return candidate == declared_type)
            && !is_float
        {
            found.push(Bound {
                line_index: index,
                member: name.to_owned(),
                declared_spelling: spelling,
                declared_bits: bits,
                declared_signed: signed,
                min,
                max,
            });
        }
    }

    return found;
}

/// `#[validate(range(min = N, max = M))]` — reads the bound out of an attribute line,
/// requiring the `validate` and `range` vocabulary, a balanced-paren clause body, and at
/// least one of `min`/`max` parsed as a plain base-ten integer with `min <= max` when both
/// are stated.
fn Range_Attribute(line: &str) -> Option<(Option<i64>, Option<i64>)>
{
    if !line.contains("validate")
    {
        return None;
    }

    let inside = Clause_Body(line, "range")?;
    let (min, max) = Min_Max(inside);
    if min.is_none() && max.is_none()
    {
        return None;
    }
    if let (Some(min_value), Some(max_value)) = (min, max)
        && min_value > max_value
    {
        return None;
    }

    return Some((min, max));
}

/// What sits inside `name(...)`, matching the closing paren at the same nesting depth
/// rather than the first one — `range(min = 0, max = 255)` sits nested inside
/// `validate(...)`.
fn Clause_Body<'a>(text: &'a str, name: &str) -> Option<&'a str>
{
    let prefix = format!("{name}(");
    let opening = text.find(&prefix)?;
    let start = opening.saturating_add(prefix.len());
    let rest = text.get(start..)?;

    let mut depth: i32 = 1;
    for (offset, character) in rest.char_indices()
    {
        match character
        {
            '(' => depth = depth.saturating_add(1),
            ')' =>
            {
                depth = depth.saturating_sub(1);
                if depth == 0
                {
                    return rest.get(..offset);
                }
            }
            _ =>
            {}
        }
    }

    return None;
}

fn Min_Max(inside: &str) -> (Option<i64>, Option<i64>)
{
    let mut min = None;
    let mut max = None;

    for clause in inside.split(',')
    {
        let Some((key, value)) = clause.split_once('=')
        else
        {
            continue;
        };
        let Ok(parsed) = value.trim().parse::<i64>()
        else
        {
            continue;
        };

        match key.trim()
        {
            "min" => min = Some(parsed),
            "max" => max = Some(parsed),
            _ =>
            {}
        }
    }

    return (min, max);
}

/// `name: Type` (optionally `pub`/`pub(...)`-qualified, trailing comma optional).
fn Field_Declaration(trimmed: &str) -> Option<(&str, &str)>
{
    let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
    let code = code.strip_suffix(',').map_or(code, str::trim_end);
    let after_visibility = Strip_Rust_Visibility(code);
    let (name, declared_type) = after_visibility.split_once(':')?;
    let name = name.trim();
    let declared_type = declared_type.trim();

    if name.is_empty() || !name.chars().all(|character| return character.is_alphanumeric() || character == '_')
    {
        return None;
    }
    if declared_type.is_empty()
    {
        return None;
    }

    return Some((name, declared_type));
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

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// This file's own path. Every fixture below spells a real attribute/field shape inside a
/// Rust string literal, which would otherwise self-match when this crate checks its own
/// workspace — the same self-exemption every other `*_text.rs`-shaped rule here carries for
/// the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "checks/scalar_range.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/").ends_with(OWN_IMPLEMENTATION_FILE);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(nomos_model::Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }

    #[test]
    fn Test_Check_Nonnegative_Storage_Is_Unsigned_Should_Report_A_Signed_Field_With_A_Nonnegative_Minimum()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n");
        let findings = Check_Nonnegative_Storage_Is_Unsigned(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(NONNEGATIVE_STORAGE_IS_UNSIGNED));
    }

    #[test]
    fn Test_Check_Nonnegative_Storage_Is_Unsigned_Should_Accept_An_Unsigned_Field_With_The_Same_Bound()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: u64,\n}\n");
        let findings = Check_Nonnegative_Storage_Is_Unsigned(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Report_A_Field_Wider_Than_Its_Bound_Needs()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n");
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings.first().expect("asserted len 1 above").summary.contains("u16"), "{:?}", findings.first());
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Accept_A_Field_At_Its_Narrowest_Type()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 0, max = 65535))]\n    port: u16,\n}\n");
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Never_Bound_A_Float_Field()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 0, max = 1))]\n    ratio: f64,\n}\n");
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert!(findings.is_empty(), "floats are projected and never reported: {findings:?}");
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Ignore_An_Attribute_With_No_Range_Clause()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[serde(rename = \"port\")]\n    port: i64,\n}\n");
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Ignore_An_Inverted_Range()
    {
        let source = Source("src/port.rs", "struct Config {\n    #[validate(range(min = 100, max = 0))]\n    port: i64,\n}\n");
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Known_Range_Picks_Its_Type_Should_Pair_With_The_Second_Of_Two_Attributes()
    {
        let source = Source(
            "src/port.rs",
            "struct Config {\n    #[serde(rename = \"port\")]\n    #[validate(range(min = 0, max = 65535))]\n    port: i64,\n}\n",
        );
        let findings = Check_A_Known_Range_Picks_Its_Type(&[source]);

        assert_eq!(findings.len(), 1, "the range clause is the second attribute, not the first: {findings:?}");
    }
}
