//! Reading a `#[validate(range(...))]` attribute and the field declaration beneath it.
//!
//! The whole text-local half of the rule: one `Bound` per field an attribute stated a range
//! for, and the linear scan that pairs them. Grouped apart from the two rules in
//! [`super`] for the same reason this crate groups every other reading half apart — the
//! pairing has no opinion about whether a bound is *wrong*, and the rules have no opinion
//! about how a bound is spelled.

use crate::checks::code_prefix::Code_Prefix;

/// `rust_declared` in `rust_scalar_range.go`: every scalar a field may be declared as.
/// `(spelling, bits, signed, is_float)`.
pub(super) const RUST_DECLARED_SCALARS: &[(&str, u32, bool, bool)] = &[
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

/// One struct field a `#[validate(range(...))]` attribute stated a bound for.
pub(super) struct Bound
{
    pub(super) line_index: usize,
    pub(super) member: String,
    pub(super) declared_spelling: &'static str,
    pub(super) declared_bits: u32,
    pub(super) declared_signed: bool,
    pub(super) min: Option<i64>,
    pub(super) max: Option<i64>,
}

fn Is_Ident_Char(character: char) -> bool
{
    return character.is_alphanumeric() || character == '_';
}

/// The linear attribute-then-field scan: a validate-range attribute is accumulated across
/// consecutive attribute lines and consumed (whether or not it pairs with a real field) by
/// the next non-attribute, non-blank line.
pub(super) fn Bounded_Fields_In(lines: &[&str]) -> Vec<Bound>
{
    let mut found = Vec::new();
    let mut pending: Option<(Option<i64>, Option<i64>)> = None;

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(bound) = Bound_At_Line(line, index, &mut pending)
        {
            found.push(bound);
        }
    }

    return found;
}

/// Processes one line of the linear attribute-then-field scan, threading `pending` (the
/// most recently accumulated `#[validate(range(...))]` bound, not yet consumed) across
/// calls: records a new bound on an attribute line, and consumes one — whether or not it
/// pairs with a real field — on the next non-attribute, non-blank line.
fn Bound_At_Line(line: &str, index: usize, pending: &mut Option<(Option<i64>, Option<i64>)>) -> Option<Bound>
{
    let trimmed = line.trim();
    if trimmed.is_empty()
    {
        return None;
    }

    if trimmed.starts_with("#[")
    {
        if let Some(range) = Range_Attribute(trimmed)
        {
            *pending = Some(range);
        }
        return None;
    }

    let (min, max) = pending.take()?;
    return Declared_Scalar_Bound(trimmed, index, min, max);
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
    let bound = Min_Max(inside);
    if bound.min.is_none() && bound.max.is_none()
    {
        return None;
    }
    if let (Some(min_value), Some(max_value)) = (bound.min, bound.max)
        && min_value > max_value
    {
        return None;
    }

    return Some((bound.min, bound.max));
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

    let close = Matching_Close_Paren_Offset(rest)?;
    return rest.get(..close);
}

/// The byte offset of the `)` that closes the already-open (depth 1) parenthesis run
/// starting at `rest`, tracking nested parens rather than stopping at the first `)` —
/// `range(min = 0, max = 255)` sits nested inside `validate(...)`.
fn Matching_Close_Paren_Offset(rest: &str) -> Option<usize>
{
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
                    return Some(offset);
                }
            }
            _ =>
            {}
        }
    }

    return None;
}

fn Min_Max(inside: &str) -> MinMax
{
    let mut min = None;
    let mut max = None;

    for clause in inside.split(',')
    {
        Apply_Clause(clause, &mut min, &mut max);
    }

    return MinMax { min, max };
}

/// `Min_Max`'s result, named so its two same-typed members cannot be swapped at a call site
/// without the compiler noticing.
struct MinMax
{
    min: Option<i64>,
    max: Option<i64>,
}

/// Parses one `key = value` clause and, if `key` is `min`/`max` and `value` parses as a
/// plain base-ten integer, records it into the matching output.
fn Apply_Clause(clause: &str, min: &mut Option<i64>, max: &mut Option<i64>)
{
    let Some((key, value)) = clause.split_once('=')
    else
    {
        return;
    };
    let Ok(parsed) = value.trim().parse::<i64>()
    else
    {
        return;
    };

    match key.trim()
    {
        "min" => *min = Some(parsed),
        "max" => *max = Some(parsed),
        _ =>
        {}
    }
}

/// The declared-scalar `Bound` for `trimmed`, if it names a non-float field from
/// [`RUST_DECLARED_SCALARS`] — `None` for a line that does not pair with the accumulated
/// attribute after all (a non-field line, most often).
fn Declared_Scalar_Bound(trimmed: &str, index: usize, min: Option<i64>, max: Option<i64>) -> Option<Bound>
{
    let (name, declared_type) = Field_Declaration(trimmed)?;
    let &(spelling, bits, signed, is_float) = RUST_DECLARED_SCALARS.iter().find(|&&(candidate, ..)| return candidate == declared_type)?;
    if is_float
    {
        return None;
    }

    return Some(Bound {
        line_index: index,
        member: name,
        declared_spelling: spelling,
        declared_bits: bits,
        declared_signed: signed,
        min,
        max,
    });
}

/// `name: Type` (optionally `pub`/`pub(...)`-qualified, trailing comma optional).
fn Field_Declaration(trimmed: &str) -> Option<(String, String)>
{
    let code_owned = Code_Prefix(trimmed);
    let code = code_owned.trim();
    let code = code.strip_suffix(',').map_or(code, str::trim_end);
    let after_visibility = Strip_Rust_Visibility(code);
    let (name, declared_type) = after_visibility.split_once(':')?;
    let name = name.trim();
    let declared_type = declared_type.trim();

    if name.is_empty() || !Is_Valid_Identifier(name)
    {
        return None;
    }
    if declared_type.is_empty()
    {
        return None;
    }

    return Some((name.to_owned(), declared_type.to_owned()));
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

fn Is_Valid_Identifier(name: &str) -> bool
{
    return name.chars().all(Is_Ident_Char);
}
