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
//! is whether the claim is true. That is exactly the split `enforcement.rs` is built on
//! — naming an enforcer is a claim about existence, whether it runs is a claim about
//! execution, and only the second can be checked against reality.
//!
//! # Why this parses instead of scanning lines
//!
//! It scanned lines first, and three runs against this workspace reported this crate's
//! own test fixtures as real universes — one of them as a phantom mirror. A trimmed
//! continuation line of a multi-line string literal is indistinguishable from a
//! declaration, and each attempt to tell them apart by text — stopping at
//! `#[cfg(test)]`, then tracking quote parity, then tracking raw strings on top of that
//! — fixed the case in front of it and left the state machine drifting on the next one.
//!
//! A rule that reports its own test data is not one anybody reads the output of, and "is
//! this a declaration" is a question `syn` already answers exactly. What remains of the
//! textual approach is the mirror claim, and it is read from parsed doc attributes rather
//! than from lines that look like comments.
//!
//! # Why the parser is *here* and nowhere else in this crate
//!
//! This module is the whole of `nomos-rules`' dependency on `syn`, and it is the second
//! Rust front end in a workspace that already has one behind `nomos.cap.syntax.items`.
//! `D-134` created it and gave a reason — replayability — that proves a rule takes its
//! subject as an argument and does not prove that the argument must be text.
//! `OD-RULES-001` withdraws that inference, moves check-name resolution onto the fact
//! layer, and keeps the parser here for a reason that is a measurement rather than an
//! inference.
//!
//! The measurement: discovery needs two things the agreed payload does not carry.
//!
//! | What discovery needs | In `nomos.syntax.items.v1` |
//! |---|---|
//! | that a `pub const` is of *slice* type | **no** — there is no type field, and `pub const LIMIT: usize` and `pub const TABLES: &[&str]` encode identically |
//! | the doc comment at the declaration site | **no** — there is no doc field, in `SyntaxItem` or in the encoding |
//!
//! Neither is a field addition, which is why the schema version is not cut here and not
//! for want of time. `nomos-lang-rust-scan` cannot read doc comments at all — it skips
//! comment lines and associates nothing with the item below them — so a schema in which
//! "this item has no doc comment" and "this provider does not read doc comments" are the
//! same bytes would turn every universe read by the scanner into one declaring no mirror:
//! a phantom silently downgraded to an admitted gap, which is absence becoming success in
//! the one field this rule's whole severity ordering turns on. A version has to carry what
//! each provider *observed*, bounded by its guarantee.
//!
//! **The end condition, stated so this is an exception and not a habit:** when
//! `nomos.syntax.items.v1` is superseded by a schema carrying an item's documentation and
//! its declared type shape, this module reads facts too, `syn` and `proc-macro2` leave
//! `Cargo.toml`, and `nomos-rules` depends on no parser at all. `P10-SYNTAX-V2` holds that
//! work.

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

/// What reading one file produced.
///
/// A parse failure is not an empty file. `Applicability::Unparseable` exists in
/// `nomos-contracts` for exactly this distinction, and collapsing the two here would put
/// "nothing to judge" and "could not be read" behind one value before the rule ever got
/// the chance to tell them apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    /// The file parsed, and these are its universes. Possibly none.
    Parsed(Vec<DeclaredUniverse>),
    /// The file did not parse. Nothing is claimed about what it contains.
    Unparseable
    {
        /// What the parser objected to.
        because: String,
    },
}

/// The marker a universe declares its mirror with.
const MIRROR_MARKER: &str = "Mirrored by ";

/// The declarations in one file.
///
/// Deliberately over-inclusive on discovery. A list that turns out to need no mirror is
/// cheap to classify once; a list that is never surfaced is the defect this exists to
/// prevent.
#[must_use]
pub fn Read_Universes(path: &str, text: &str) -> Reading
{
    let file = match syn::parse_file(text)
    {
        Ok(parsed) => parsed,
        Err(refusal) =>
        {
            return Reading::Unparseable {
                because: format!("{refusal} at line {}", refusal.span().start().line),
            };
        }
    };

    let mut found = Vec::new();
    Universes_In_Items(path, &file.items, &mut found);

    found.sort();
    found.dedup();
    return Reading::Parsed(found);
}

/// The declarations in one file, or none when it does not parse.
///
/// The convenience form, for callers with nothing useful to do with the difference. The
/// rule itself does not use it: reporting an unreadable file as a clean one is the defect
/// this workspace keeps finding.
#[must_use]
pub fn Universes_In(path: &str, text: &str) -> Vec<DeclaredUniverse>
{
    return match Read_Universes(path, text)
    {
        Reading::Parsed(universes) => universes,
        Reading::Unparseable { .. } => Vec::new(),
    };
}

/// Walks a list of items, descending into inline modules.
///
/// `#[cfg(test)]` modules are descended into like any other. A list declared under test
/// is still a list; the line scanner skipped them to work around its own false positives,
/// and that reason did not survive it.
fn Universes_In_Items(path: &str, items: &[syn::Item], found: &mut Vec<DeclaredUniverse>)
{
    for item in items
    {
        match item
        {
            syn::Item::Const(constant)
                if Is_Slice(&constant.ty) && Is_Public(&constant.vis) =>
            {
                found.push(DeclaredUniverse {
                    path: path.to_owned(),
                    name: constant.ident.to_string(),
                    kind: UniverseKind::Constant,
                    claimed_mirror: Claimed_Mirror(&constant.attrs),
                });
            }
            syn::Item::Impl(block) => Universes_In_Impl(path, block, found),
            syn::Item::Mod(module) =>
            {
                if let Some((_, nested)) = module.content.as_ref()
                {
                    Universes_In_Items(path, nested, found);
                }
            }
            _ =>
            {}
        }
    }
}

/// An `All()` in an inherent implementation names the type's own variant list.
///
/// Inherent implementations only. `impl Display for Table` does not own the variant list,
/// and attributing an `All()` found there to `Table` would name the wrong universe.
fn Universes_In_Impl(path: &str, block: &syn::ItemImpl, found: &mut Vec<DeclaredUniverse>)
{
    if block.trait_.is_some()
    {
        return;
    }

    let Some(owner) = Type_Name(&block.self_ty)
    else
    {
        return;
    };

    for item in &block.items
    {
        if let syn::ImplItem::Fn(function) = item
            && function.sig.ident == "All"
            && function.sig.inputs.is_empty()
        {
            found.push(DeclaredUniverse {
                path: path.to_owned(),
                name: format!("{owner}::All"),
                kind: UniverseKind::Enumeration,
                claimed_mirror: Claimed_Mirror(&function.attrs),
            });
        }
    }
}

/// Whether a declaration is visible outside its own module.
///
/// The scope this rule judges, and a deliberate narrowing rather than an accident of
/// implementation. A completeness guard built on a list the rest of the workspace cannot
/// see fails within one module, where the declaration and its uses are read together; the
/// three instances `OD-COMPLETENESS-001` analyses were all public lists consumed from
/// somewhere else, which is what let each of them go wrong unnoticed for months.
///
/// It is also what keeps this rule's answer comparable with the classification
/// `tests/contract` already declares. Widening to private lists takes the workspace from
/// thirteen unmirrored universes to thirty-four, every one of which needs a human to say
/// what would go wrong — that is somebody's next item, not a side effect of this one.
fn Is_Public(visibility: &syn::Visibility) -> bool
{
    return matches!(visibility, syn::Visibility::Public(_));
}

/// Whether a type is a slice reference — the shape a declared list has.
///
/// `&[&str]` and `&'static [Self]` are universes. `&str` and `usize` are not: a scalar
/// constant is not a list, and matching one would bury the real ones.
fn Is_Slice(kind: &syn::Type) -> bool
{
    return match kind
    {
        syn::Type::Reference(reference) => Is_Slice(&reference.elem),
        syn::Type::Slice(_) | syn::Type::Array(_) => true,
        _ => false,
    };
}

/// The last segment of a path type, which is the name a universe is known by.
fn Type_Name(kind: &syn::Type) -> Option<String>
{
    let syn::Type::Path(path) = kind
    else
    {
        return None;
    };

    return path
        .path
        .segments
        .last()
        .map(|segment| return segment.ident.to_string());
}

/// The mirror named in an item's documentation, if one is named.
///
/// Reads the *first* claim rather than the last. A doc comment that names two mirrors is
/// an authoring mistake, and taking the first makes the rule deterministic about which
/// one it resolves instead of depending on comment order in a way nobody would predict.
fn Claimed_Mirror(attributes: &[syn::Attribute]) -> Option<String>
{
    for line in Documentation(attributes)
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

/// Every `#[doc = "…"]` line on an item, in order.
///
/// `///` is `#[doc]` after parsing, so both spellings are read and neither has to be
/// recognised as text.
fn Documentation(attributes: &[syn::Attribute]) -> Vec<String>
{
    let mut lines = Vec::new();

    for attribute in attributes
    {
        if !attribute.path().is_ident("doc")
        {
            continue;
        }

        if let syn::Meta::NameValue(pair) = &attribute.meta
            && let syn::Expr::Lit(literal) = &pair.value
            && let syn::Lit::Str(text) = &literal.lit
        {
            lines.push(text.value());
        }
    }

    return lines;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Constant_Slice_Should_Be_Found()
    {
        let found = Universes_In("a.rs", "pub const GOVERNING_RECORD_IDS: &[&str] = &[];");

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
            "impl Table { pub const fn All() -> &'static [Self] { &[] } }",
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
    /// `All()` to it would name the wrong universe.
    #[test]
    fn Test_A_Trait_Impl_Should_Not_Claim_The_Type()
    {
        let found = Universes_In(
            "a.rs",
            "impl Table { } impl Display for Other { fn All() {} }",
        );

        assert!(found.is_empty(), "{found:?}");
    }

    /// A scalar constant is not a universe. Matching it would bury the real ones.
    #[test]
    fn Test_A_Scalar_Constant_Should_Not_Be_A_Universe()
    {
        assert!(Universes_In("a.rs", "pub const LIMIT: usize = 2_000;").is_empty());
        assert!(Universes_In("a.rs", "pub const NAME: &str = \"x\";").is_empty());
    }

    #[test]
    fn Test_A_Declared_Mirror_Should_Be_Read_Off_The_Doc_Comment()
    {
        let found = Universes_In(
            "a.rs",
            "/// The tables.\n\
             ///\n\
             /// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];",
        );

        assert_eq!(
            found.first().and_then(|universe| universe.claimed_mirror.clone()),
            Some("Test_Every_Table_Should_Be_Declared".to_owned())
        );
    }

    /// The claim belongs to the item it documents. Reading it off nearby text would let
    /// one annotation silently mirror every universe below it in the file.
    #[test]
    fn Test_A_Claim_Should_Not_Carry_To_The_Next_Item()
    {
        let found = Universes_In(
            "a.rs",
            "/// Mirrored by `Test_X`.\n\
             pub const FIRST: &[&str] = &[];\n\
             /// No claim here.\n\
             pub const SECOND: &[&str] = &[];",
        );

        assert_eq!(found.len(), 2);

        let claims: Vec<Option<String>> = found
            .iter()
            .map(|universe| return universe.claimed_mirror.clone())
            .collect();

        assert_eq!(claims, vec![Some("Test_X".to_owned()), None]);
    }

    /// Prose about mirroring is not a claim. Only the backticked form is, because a
    /// sentence that happens to contain the words would otherwise read as coverage.
    #[test]
    fn Test_Unquoted_Prose_Should_Not_Be_A_Claim()
    {
        let found = Universes_In(
            "a.rs",
            "/// Mirrored by nothing in particular.\npub const TABLES: &[&str] = &[];",
        );

        assert_eq!(
            found.first().and_then(|universe| universe.claimed_mirror.clone()),
            None
        );
    }

    #[test]
    fn Test_The_First_Of_Two_Claims_Should_Win()
    {
        let found = Universes_In(
            "a.rs",
            "/// Mirrored by `Test_A`.\n\
             /// Mirrored by `Test_B`.\n\
             pub const TABLES: &[&str] = &[];",
        );

        assert_eq!(
            found.first().and_then(|universe| universe.claimed_mirror.clone()),
            Some("Test_A".to_owned())
        );
    }

    /// ---- the reason this parses instead of scanning lines ----
    ///
    /// Three runs of the line scanner against this workspace reported this crate's own
    /// fixtures as real universes, one of them as a phantom mirror. Below are the two
    /// shapes that defeated it: a multi-line string literal whose continuation line
    /// begins with `pub const`, and a raw string containing an ordinary one. A parser
    /// cannot make this mistake, and this test is what stops anybody trading the parser
    /// back for a cheaper scan.
    #[test]
    fn Test_A_Declaration_Inside_A_String_Should_Not_Be_A_Universe()
    {
        let found = Universes_In(
            "a.rs",
            r##"pub const REAL: &[&str] = &[];

fn Fixture()
{
    let escaped = "impl Table\n\
                   pub const NOT_REAL: &[&str] = &[];";
    let raw = r#"/// Mirrored by `Test_Nowhere`.
pub const ALSO_NOT_REAL: &[&str] = &[];"#;
}
"##,
        );

        let names: Vec<&str> = found
            .iter()
            .map(|universe| return universe.name.as_str())
            .collect();

        assert_eq!(names, vec!["REAL"], "a fixture is not a declaration");
    }

    /// A list declared under test is still a list.
    #[test]
    fn Test_A_Universe_Inside_A_Test_Module_Should_Still_Be_Found()
    {
        let found = Universes_In(
            "a.rs",
            "#[cfg(test)] mod tests { pub const FIXTURE_NAMES: &[&str] = &[]; }",
        );

        assert_eq!(
            found.first().map(|universe| universe.name.clone()),
            Some("FIXTURE_NAMES".to_owned())
        );
    }

    /// A file that does not parse is not a file with nothing in it. Collapsing the two is
    /// how a check reports clean over what it could not read.
    #[test]
    fn Test_An_Unparseable_File_Should_Say_So_Rather_Than_Look_Empty()
    {
        let reading = Read_Universes("a.rs", "pub const ??? = ;");

        assert!(
            matches!(reading, Reading::Unparseable { .. }),
            "{reading:?} must not be an empty parse"
        );
    }

    #[test]
    fn Test_An_Empty_File_Should_Parse_To_Nothing()
    {
        assert_eq!(Read_Universes("a.rs", ""), Reading::Parsed(Vec::new()));
    }
}
