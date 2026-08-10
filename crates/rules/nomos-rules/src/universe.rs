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

use nomos_cap_syntax::{
    Function_Arity, PayloadItem, SyntaxPayload, FUNCTION, IMPLEMENTATION, INHERENT, SLICE,
};

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

/// One list that some completeness guard quantifies over.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredUniverse
{
    /// Repo-relative, forward slashes. For reporting; never identity.
    pub path: String,
    /// `GOVERNING_RECORD_IDS`, or `Table::All` for an enumeration.
    ///
    /// This is the stable name. A universe that moves file keeps it, which is what lets
    /// a finding about one survive a refactor instead of closing and reopening.
    pub name: String,
    /// How it is written down.
    pub kind: UniverseKind,
    /// The mirror this universe claims, if it claims one.
    ///
    /// A claim, not a fact. Whether the named check exists is what the rule resolves,
    /// and a claim that resolves to nothing is worse than no claim at all — it reads as
    /// coverage while checking nothing.
    pub claimed_mirror: Option<String>,
}

/// What reading one file's syntax fact produced.
///
/// Two variants, and the second is not "the file was empty". A provider that could not
/// observe documentation has told this module nothing about mirrors, and reporting that as
/// a file whose universes all declare none would turn every phantom under that provider
/// into an admitted gap. `Applicability::Unparseable` exists in `nomos-contracts` for the
/// neighbouring distinction and the rule maps this onto it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    /// The provider observed what discovery needs, and these are the universes. Possibly
    /// none, which is a real answer about the file.
    Observed(Vec<DeclaredUniverse>),
    /// The provider that answered cannot see what discovery reads. Nothing is claimed
    /// about what the file contains.
    Unobserved
    {
        /// Which field, so a finding can say what would have to change.
        because: String,
    },
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
pub fn Read_Universes(path: &str, payload: &SyntaxPayload) -> Reading
{
    if let Some(field) = Unobserved_Field(payload)
    {
        return Reading::Unobserved {
            because: format!(
                "the provider that answered did not observe each item's {field}, which is \
                 what a declared universe and its claimed mirror are read from"
            ),
        };
    }

    let mut found = Vec::new();

    for (index, item) in payload.items.iter().enumerate()
    {
        if let Some(universe) = Constant_Universe(path, item)
        {
            found.push(universe);
            continue;
        }

        if let Some(universe) = Enumeration_Universe(path, payload, index, item)
        {
            found.push(universe);
        }
    }

    found.sort();
    found.dedup();
    return Reading::Observed(found);
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

/// A public constant whose declared type is a list.
///
/// Public only, and that is a deliberate narrowing rather than an accident of
/// implementation. A completeness guard built on a list the rest of the workspace cannot
/// see fails within one module, where the declaration and its uses are read together; the
/// three instances `OD-COMPLETENESS-001` analyses were all public lists consumed from
/// somewhere else, which is what let each of them go wrong unnoticed for months.
///
/// Widening to private lists takes the workspace from twelve unmirrored universes to
/// forty, every one of which needs a human to say what would go wrong — that is somebody's
/// next item, not a side effect of this one. Both figures were measured on 2026-08-09;
/// `OD-COMPLETENESS-002` records that an earlier sentence said thirty-four, which no longer
/// matched anything measurable.
fn Constant_Universe(path: &str, item: &PayloadItem) -> Option<DeclaredUniverse>
{
    if item.kind != "Constant" || !item.Is_Public() || item.shape.Value() != Some(SLICE)
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

/// An `All()` in an inherent implementation names the type's own variant list.
///
/// Inherent implementations only. `impl Display for Table` does not own the variant list,
/// and attributing an `All()` found there to `Table` would name the wrong universe — a
/// distinction that survives into the payload only because the schema carries an
/// implementation's shape and the items are in source order.
///
/// Arity zero, for the same reason it always was: `All(&self)` is an accessor on an
/// instance and not the type's list of itself.
fn Enumeration_Universe(
    path: &str,
    payload: &SyntaxPayload,
    index: usize,
    item: &PayloadItem,
) -> Option<DeclaredUniverse>
{
    if item.kind != FUNCTION || item.Own_Name() != ALL || Function_Arity(&item.shape) != Some(0)
    {
        return None;
    }

    let owner = payload.Enclosing(index)?;
    if owner.kind != IMPLEMENTATION || owner.shape.Value() != Some(INHERENT)
    {
        return None;
    }

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

/// The mirror named in an item's documentation, if one is named.
///
/// Reads the *first* claim rather than the last. A doc comment that names two mirrors is
/// an authoring mistake, and taking the first makes the rule deterministic about which
/// one it resolves instead of depending on comment order in a way nobody would predict.
fn Claimed_Mirror(documentation: Option<&str>) -> Option<String>
{
    for line in documentation?.lines()
    {
        let Some(after) = line.split_once(MIRROR_MARKER).map(|(_, rest)| return rest)
        else
        {
            continue;
        };

        let Some(quoted) = after.strip_prefix('`')
        else
        {
            continue;
        };

        let Some((name, _)) = quoted.split_once('`')
        else
        {
            continue;
        };

        if !name.trim().is_empty()
        {
            return Some(name.trim().to_owned());
        }
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_syntax::Parse_Payload;

    /// A payload built from item records, so a fixture reads as the bytes a provider wrote.
    fn Payload(records: &str) -> SyntaxPayload
    {
        return Parse_Payload(format!("unexpanded\t0\n{records}").as_bytes())
            .expect("the fixture is written in the schema");
    }

    #[test]
    fn Test_A_Constant_Slice_Should_Be_Found()
    {
        let found = Universes_In(
            "a.rs",
            &Payload("item\t0\tConstant\tPublic\tGOVERNING_RECORD_IDS\t.\t+slice\n"),
        );

        assert_eq!(found.len(), 1);
        assert_eq!(
            found.first().map(|universe| universe.kind),
            Some(UniverseKind::Constant)
        );
    }

    #[test]
    fn Test_An_All_Should_Be_Attributed_To_Its_Type()
    {
        let found = Universes_In(
            "a.rs",
            &Payload(
                "item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tFunction\tPublic\tTable::All\t.\t+fn/0\n",
            ),
        );

        assert_eq!(
            found.first().map(|universe| universe.name.clone()),
            Some("Table::All".to_owned())
        );
        assert_eq!(
            found.first().map(|universe| universe.kind),
            Some(UniverseKind::Enumeration)
        );
    }

    /// A trait implementation does not own the type's variant list, so attributing an
    /// `All()` to it would name the wrong universe. Two `impl` blocks for one type carry
    /// the same qualified name, and only the record each member follows tells them apart.
    #[test]
    fn Test_A_Trait_Impl_Should_Not_Claim_The_Type()
    {
        let found = Universes_In(
            "a.rs",
            &Payload(
                "item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tImplementation\tNotApplicable\tOther\t.\t+trait\n\
                 item\t2\tFunction\tNotApplicable\tOther::All\t.\t+fn/0\n",
            ),
        );

        assert!(found.is_empty(), "{found:?}");
    }

    /// An accessor on an instance is not the type's list of itself.
    #[test]
    fn Test_An_All_That_Takes_A_Receiver_Should_Not_Be_A_Universe()
    {
        let found = Universes_In(
            "a.rs",
            &Payload(
                "item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tFunction\tPublic\tTable::All\t.\t+fn/1\n",
            ),
        );

        assert!(found.is_empty(), "{found:?}");
    }

    /// A scalar constant is not a universe. Matching it would bury the real ones.
    #[test]
    fn Test_A_Scalar_Constant_Should_Not_Be_A_Universe()
    {
        assert!(Universes_In(
            "a.rs",
            &Payload("item\t0\tConstant\tPublic\tLIMIT\t.\t+value\n")
        )
        .is_empty());
    }

    /// A list the rest of the workspace cannot see is out of this rule's declared scope.
    #[test]
    fn Test_A_Private_List_Should_Not_Be_A_Universe()
    {
        assert!(Universes_In(
            "a.rs",
            &Payload("item\t0\tConstant\tPrivate\tTABLES\t.\t+slice\n")
        )
        .is_empty());
    }

    #[test]
    fn Test_A_Declared_Mirror_Should_Be_Read_Off_The_Doc_Comment()
    {
        let found = Universes_In(
            "a.rs",
            &Payload(
                "item\t0\tConstant\tPublic\tTABLES\t+ A list.\\n Mirrored by `Test_Every_Row`.\t+slice\n",
            ),
        );

        assert_eq!(
            found.first().and_then(|universe| universe.claimed_mirror.clone()),
            Some("Test_Every_Row".to_owned())
        );
    }

    /// A list with no doc comment claims no mirror, and that is a real answer about the
    /// source rather than a failure to look.
    #[test]
    fn Test_A_List_With_No_Documentation_Should_Claim_No_Mirror()
    {
        let found = Universes_In(
            "a.rs",
            &Payload("item\t0\tConstant\tPublic\tTABLES\t.\t+slice\n"),
        );

        assert_eq!(found.first().and_then(|universe| universe.claimed_mirror.clone()), None);
    }

    /// The property v2 exists for, at the consumer end.
    ///
    /// The same list, read through a provider that cannot see doc comments, must not come
    /// back as a list that declares no mirror. It comes back as nothing observed, and the
    /// rule turns that into a finding rather than into silence.
    #[test]
    fn Test_A_Payload_From_A_Blind_Provider_Should_Not_Read_As_No_Mirror()
    {
        let blind = Payload("item\t0\tConstant\tPublic\tTABLES\t-\t-\n");

        let reading = Read_Universes("a.rs", &blind);

        assert!(
            matches!(reading, Reading::Unobserved { ref because } if because.contains("documentation")),
            "{reading:?}"
        );
        assert!(
            Universes_In("a.rs", &blind).is_empty(),
            "the convenience form yields nothing, which is why the rule does not use it"
        );
    }

    /// A file that declares nothing is not a file nobody could read.
    #[test]
    fn Test_A_File_That_Declares_Nothing_Should_Be_Observed_And_Empty()
    {
        assert_eq!(Read_Universes("a.rs", &Payload("")), Reading::Observed(Vec::new()));
    }
}
