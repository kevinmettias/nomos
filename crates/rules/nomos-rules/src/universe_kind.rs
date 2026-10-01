//! Finding the lists a completeness guard quantifies over, and the mirrors they claim.
//!
//! `OD-COMPLETENESS-001`: a completeness guard is only as complete as the universe it
//! quantifies over, and when that universe is *declared* rather than derived, comparing
//! the declaration against reality is the other direction.
//!
//! Discovery is mechanical. What was not mechanical, until this module, was the other
//! half — whether a given list has a check comparing it against the reality it claims to
//! enumerate. `tests/contract` answered that with a hand-written classification table,
//! and said in its own doc comment why: the question is about meaning and that crate has
//! no types to answer it with.
//!
//! This module changes the question rather than answering the old one. A universe
//! *declares* its mirror, in a doc comment, at the site:
//!
//! ```text
//! /// Mirrored by `Test_Every_Table_In_The_Schema_Should_Be_Declared`.
//! ```
//!
//! and the rule resolves that name against the real source. Meaning stays with the
//! author, which is where it was always going to have to live; what becomes mechanical
//! is whether the claim is true.
//!
//! # This module used to hold a parser, and that is the point of the version it reads
//!
//! It scanned lines first, and three runs against this workspace reported this crate's own
//! test fixtures as real universes — one of them as a phantom mirror. A trimmed
//! continuation line of a multi-line string literal is indistinguishable from a
//! declaration. So it parsed instead, with `syn`, and `nomos-rules` carried a second Rust
//! front end in a workspace that already had one behind `nomos.cap.syntax.items`.
//!
//! `OD-RULES-001` kept that parser for a measurement rather than an inference: discovery
//! needs two things the agreed payload did not carry — that a `pub const` is of *slice*
//! type, and the doc comment at the declaration site. It stated the end condition, and
//! `nomos.syntax.items.v2` is it. The payload now carries both, per item, as
//! [`nomos_cap_syntax::Observation`]s, and this module reads facts like every other part of
//! the rule. `nomos-rules` depends on no parser at all.
//!
//! # Why the two fields had to be observations
//!
//! Because "this item has no doc comment" and "this provider does not read doc comments"
//! are different answers, and one of them is a silent downgrade. `nomos-lang-rust-scan`
//! cannot read a doc comment at all. Had v2 spelled both as an empty string, every list
//! read through the scanner would have arrived here as a list declaring no mirror — a
//! phantom mirror downgraded to an admitted gap, which is absence becoming success in the
//! one field this rule's severity ordering turns on.
//!
//! So a payload whose fields were not observed produces [`Reading::Unobserved`] and never
//! an empty list of universes. `OD-SYNTAX-002` records the schema half of this.

use crate::DeclaredUniverse;
use crate::Reading;
use nomos_cap_syntax::{FUNCTION, Function_Arity, IMPLEMENTATION, Impl_Serves_A_Trait, PayloadItem, SLICE, SyntaxPayload};

/// How a universe is written down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniverseKind
{
    /// A constant slice — a list of names kept beside the thing it enumerates.
    Constant,
    /// An `All()` over an enum — a list of variants kept beside the enum.
    ///
    /// The compiler does not check it. A variant added without adding it here drops out
    /// of every guard built on `All()`, and each of those guards then passes by not
    /// looking.
    Enumeration,
}

/// The marker a universe declares its mirror with.
const MIRROR_MARKER: &str = "Mirrored by ";

/// The method name that makes an inherent implementation a declared variant list.
const ALL: &str = "All";

/// The universes one file declares, read from its syntax fact.
///
/// Deliberately over-inclusive on discovery. A list that turns out to need no mirror is
/// cheap to classify once; a list that is never surfaced is the defect this exists to
/// prevent.
#[must_use]
pub(crate) fn Read_Universes(path: &str, payload: &SyntaxPayload) -> Reading
{
    if let Some(field) = Unobserved_Field(payload)
    {
        return Unobserved_Reading(field);
    }

    let mut found = Vec::new();
    for (index, item) in payload.items.iter().enumerate()
    {
        if let Some(universe) = Declared_By(path, payload, index, item)
        {
            found.push(universe);
        }
    }

    found.sort();
    found.dedup();

    return Reading::Observed(found);
}

/// The first field discovery needs that the provider did not observe.
///
/// A payload with no items at all is not unobserved — a provider that saw nothing to
/// report about a file that declares nothing has observed exactly as much as one that
/// parsed it, and refusing there would report every empty file as unread.
fn Unobserved_Field(payload: &SyntaxPayload) -> Option<&'static str>
{
    for item in &payload.items
    {
        if !item.documentation.Was_Observed()
        {
            return Some("documentation");
        }
        if !item.shape.Was_Observed()
        {
            return Some("declared shape");
        }
    }

    return None;
}

/// A provider that answered without observing what a universe is read from.
///
/// Named as unobserved rather than reported as no universe, because a file the reader could
/// not see the doc comments of is not a file that declares nothing.
fn Unobserved_Reading(field: &str) -> Reading
{
    return Reading::Unobserved {
        because: format!(
            "the provider that answered did not observe each item's {field}, which is what a \
             declared universe and its claimed mirror are read from"
        ),
    };
}

/// The universe one item declares, if it declares one.
///
/// A constant is asked about first because it carries its own membership; an enumeration
/// has to be read against the file around it, which is why it needs the payload and not
/// only the item.
fn Declared_By(
    path: &str,
    payload: &SyntaxPayload,
    index: usize,
    item: &PayloadItem,
) -> Option<DeclaredUniverse>
{
    if let Some(universe) = Constant_Universe(path, item)
    {
        return Some(universe);
    }

    return Enumeration_Universe(path, payload, index, item);
}

/// A public constant whose declared type is a list.
///
/// Public only, and that is a deliberate narrowing rather than an accident of
/// implementation. A completeness guard built on a list the rest of the workspace cannot
/// see fails within one module, where the declaration and its uses are read together; the
/// three instances `OD-COMPLETENESS-001` analyses were all public lists consumed from
/// somewhere else, which is what let each of them go wrong unnoticed for months.
///
/// Widening to private lists multiplies the holes by roughly an order of magnitude, and every
/// one of them needs a human to say what would go wrong — that is somebody's next item, not
/// a side effect of this one. The pair measured on 2026-08-09 was twelve unmirrored universes
/// widening to forty; both halves have moved since, and the public figure is now the
/// `UNMIRRORED_TOTAL` that `tests/contract/tests/completeness_universes/table.rs` declares and
/// checks, so read those two numbers as the dated measurement they were rather than as the
/// count today. `OD-COMPLETENESS-002` records that an earlier sentence said thirty-four, which
/// no longer matched anything measurable; this one went the same way.
fn Constant_Universe(path: &str, item: &PayloadItem) -> Option<DeclaredUniverse>
{
    if !Is_Constant_Universe_Declaration(item)
    {
        return None;
    }

    return Some(DeclaredUniverse {
        path: path.to_owned(),
        name: item.Own_Name().to_owned(),
        kind: UniverseKind::Constant,
        claimed_mirror: Claimed_Mirror(item.documentation.Value()),
    });
}

/// Whether an item is a public constant holding a list.
///
/// The three together are the whole of what a declared constant universe is: a private one
/// is out of scope by the narrowing above, and a scalar constant is not a list of anything.
fn Is_Constant_Universe_Declaration(item: &PayloadItem) -> bool
{
    return item.kind == "Constant" && item.Is_Public() && item.shape.Value() == Some(SLICE);
}

fn Enumeration_Universe(
    path: &str,
    payload: &SyntaxPayload,
    index: usize,
    item: &PayloadItem,
) -> Option<DeclaredUniverse>
{
    if !Is_The_Variant_List(item)
    {
        return None;
    }

    let owner = Inherent_Impl_Owner(payload, index)?;

    // The type and the method, and not the modules above them. A universe's name is what a
    // finding is keyed on, and it stayed stable across the move to facts because the two
    // trailing segments are what the parser used to build by hand.
    return Some(DeclaredUniverse {
        path: path.to_owned(),
        name: format!("{}::{ALL}", owner.Own_Name()),
        kind: UniverseKind::Enumeration,
        claimed_mirror: Claimed_Mirror(item.documentation.Value()),
    });
}

/// An `All()` in an inherent implementation names the type's own variant list.
///
/// Inherent implementations only. `impl Display for Table` does not own the variant list,
/// and attributing an `All()` found there to `Table` would name the wrong universe — a
/// distinction that survives into the payload only because the schema carries an
/// implementation's shape and the items are in source order.
///
/// Arity zero, for the same reason it always was: `All(&self)` is an accessor on an
/// instance and not the type's list of itself.
/// Whether an item is the zero-argument `All()` that names a type's variants.
///
/// Arity zero is part of the shape rather than a detail: `All(&self)` is an accessor on an
/// instance, and it does not enumerate the type.
fn Is_The_Variant_List(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && item.Own_Name() == ALL && Function_Arity(&item.shape) == Some(0);
}

/// The item enclosing `index` when that item is an inherent implementation — the only place a
/// variant list may be declared, and `None` for anything else.
fn Inherent_Impl_Owner(payload: &SyntaxPayload, index: usize) -> Option<&PayloadItem>
{
    let owner = payload.Enclosing(index)?;
    // Read through the typed reader rather than compared against `INHERENT`, for the reason
    // `abbreviations.rs`'s own carry gives: since `OD-CAPABILITY-014` an `impl` block's shape
    // carries its generic type parameters behind that label, and an equality test against the
    // bare constant would stop seeing `impl<T> Holder<T>`'s own `ALL` as a declared universe.
    if owner.kind != IMPLEMENTATION || Impl_Serves_A_Trait(&owner.shape) != Some(false)
    {
        return None;
    }

    return Some(owner);
}

/// The universes one file declares, or none when the provider could not observe them.
///
/// The convenience form, for callers with nothing useful to do with the difference. The
/// rule itself does not use it: reporting an unobserved file as a clean one is the defect
/// this workspace keeps finding.
#[must_use]
pub fn Universes_In(path: &str, payload: &SyntaxPayload) -> Vec<DeclaredUniverse>
{
    return match Read_Universes(path, payload)
           {
        Reading::Observed(universes) => universes,
        Reading::Unobserved { .. } => Vec::new(),
           };
}

/// The mirror named in an item's documentation, if one is named.
///
/// Reads the *first* claim rather than the last. A doc comment that names two mirrors is
/// an authoring mistake, and taking the first makes the rule deterministic about which
/// one it resolves instead of depending on comment order in a way nobody would predict.
fn Claimed_Mirror(documentation: Option<&str>) -> Option<String>
{
    for line in documentation?.lines()
    {
        if let Some(name) = Named_On(line)
        {
            return Some(name);
        }
    }

    return None;
}

/// The mirror one line names, if it names one.
///
/// The name has to be in backticks and non-empty. A marker followed by prose is a sentence
/// about mirrors rather than a claim to have one, and admitting it would invent a check
/// name out of whatever word came next.
fn Named_On(line: &str) -> Option<String>
{
    let (_, after) = line.split_once(MIRROR_MARKER)?;
    let quoted = after.strip_prefix('`')?;
    let (name, _) = quoted.split_once('`')?;

    if name.trim().is_empty()
    {
        return None;
    }

    return Some(name.trim().to_owned());
}

#[cfg(test)]
mod tests;
