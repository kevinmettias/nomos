//! Reading `nomos.syntax.items.v1` back out of a fact's payload.
//!
//! The schema is line-oriented and tab-separated: one `unexpanded` header carrying a
//! count, then one `item` record per declaration, in source order. Both providers of
//! `nomos.cap.syntax.items` write it and this module reads it.
//!
//! # This is the third reader of a schema with no written grammar
//!
//! `nomos-lang-rust` and `nomos-lang-rust-scan` each author these bytes, and
//! `tests/integration/src/surface.rs` already reads them back. Adding a third is a real
//! cost, taken knowingly rather than absorbed: the schema is a [`nomos_contracts::SchemaId`]
//! string in `nomos-cap-syntax` and nothing else, so the *shape* of an answer two providers
//! are bound by is the part of the agreement with no home. `P10-SYNTAX-SCHEMA` holds the
//! consolidation. It is not taken here because a contract earns a home below its parties
//! once more than one party names it, and moving the reader would pull
//! `crates/capabilities/nomos-cap-syntax` into this item's territory for a consumer that
//! does not exist yet.
//!
//! # Why a refusal and not an empty set
//!
//! A decoder that returns "no items" for bytes it could not read reproduces one layer up
//! the exact defect [`crate::Reading::Unparseable`] exists to prevent one layer down: a
//! subject that was never read rendering identically to a subject that declared nothing.
//! Every path out of [`Check_Names_In`] that is not a successfully decoded payload is a
//! [`PayloadRefusal`] the caller has to handle, and the caller turns it into a finding.

use std::collections::BTreeSet;

/// The label an item record carries when it is a function.
const FUNCTION: &str = "Function";

/// The visibility label the payload puts on an item form that declares no visibility.
///
/// A trait method signature and an `impl` block both carry it. It is the only mark in the
/// payload that separates `fn Test_X(&self);` in a trait — a signature, which runs nothing —
/// from a definition, and [`Check_Names_In`] relies on that.
const NOT_APPLICABLE: &str = "NotApplicable";

/// The prefix that makes a function a check, by this workspace's naming convention.
const CHECK_PREFIX: &str = "Test_";

/// Fields in an `item` record, tag included.
const ITEM_FIELDS: usize = 5;

/// Fields in the `unexpanded` header, tag included.
const HEADER_FIELDS: usize = 2;

/// Why a payload this build was handed could not be read.
///
/// Carries where, for the same reason [`nomos_contracts::Applicability`] exists: a caller
/// told only that something failed has been handed a number nobody can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PayloadRefusal
{
    /// The bytes are not UTF-8. The schema is tabs around UTF-8 identifiers, so they are
    /// not this schema.
    NotUtf8,
    /// The payload does not begin with the `unexpanded` header.
    ///
    /// Includes the empty payload. A fact carrying no bytes at all is not a file that
    /// declared nothing — that is a header and no `item` records — and reading it as one
    /// would make a provider that wrote nothing indistinguishable from a file with nothing
    /// in it.
    NoHeader,
    /// A record tag this build does not understand.
    ///
    /// The likeliest cause is a payload written by a newer schema than this reader knows,
    /// which is precisely the case where guessing is worst: the unknown record is where
    /// the information this reader is missing would be.
    UnknownRecord
    {
        tag: String,
        line: usize,
    },
    /// A record with the right tag and the wrong shape.
    WrongFieldCount
    {
        tag: String,
        expected: usize,
        found: usize,
        line: usize,
    },
    /// A field that must be a number and is not.
    UnreadableNumber
    {
        field: &'static str,
        value: String,
        line: usize,
    },
}

impl PayloadRefusal
{
    /// What went wrong, in terms somebody can act on.
    pub(crate) fn Describe(&self) -> String
    {
        return match self
        {
            Self::NotUtf8 => "the payload is not UTF-8, so it is not this schema".to_owned(),
            Self::NoHeader => "the payload does not begin with an `unexpanded` header, so it \
                               is either empty or not this schema"
                .to_owned(),
            Self::UnknownRecord { tag, line } => format!(
                "line {line} carries the record tag `{tag}`, which this build does not \
                 understand"
            ),
            Self::WrongFieldCount {
                tag,
                expected,
                found,
                line,
            } => format!(
                "line {line} is a `{tag}` record with {found} field(s) where this build \
                 expects {expected}"
            ),
            Self::UnreadableNumber { field, value, line } => {
                format!("line {line} carries `{value}` where `{field}` must be a number")
            }
        };
    }
}

/// Every check name the payload declares.
///
/// A check is a test function by this workspace's naming convention: a `Function` item
/// whose own name begins with `Test_`. Two filters, and each one is load-bearing.
///
/// The name is the **last** `::` segment of the record's qualified name, because the
/// payload qualifies an item by its syntactic nesting — a check written in `mod tests`
/// arrives as `tests::Test_X` and a check written in an `impl` block arrives as
/// `Table::Test_X`. Matching the qualified form against a universe's claimed mirror would
/// resolve neither, and every check in this workspace lives inside something.
///
/// The visibility filter drops trait-method signatures. `NotApplicable` is the only mark
/// the payload puts on them and a signature is not a check that runs, so a
/// `fn Test_X(&self);` declared in a trait must not resolve a claim of coverage. That is a
/// shape convention rather than something the schema states — it works because
/// `visit_trait_item_fn` records `NotApplicable` and every other function form records the
/// visibility it declares — and it is written down here so nobody later reads it as a
/// guarantee. Naming the distinction properly is `P10-SYNTAX-SCHEMA`'s.
///
/// # Errors
///
/// [`PayloadRefusal`] for anything this build cannot read. Never an empty set standing in
/// for a failed read.
pub(crate) fn Check_Names_In(bytes: &[u8]) -> Result<BTreeSet<String>, PayloadRefusal>
{
    let Ok(text) = core::str::from_utf8(bytes)
    else
    {
        return Err(PayloadRefusal::NotUtf8);
    };

    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut seen_header = false;

    for (offset, line) in text.lines().enumerate()
    {
        let at = offset.saturating_add(1);
        let fields: Vec<&str> = line.split('\t').collect();
        // `split` yields at least one element for every input, including the empty one,
        // so an absent tag is an empty tag and reaches `UnknownRecord` rather than a
        // silent skip.
        let tag = fields.first().copied().unwrap_or_default();

        match tag
        {
            "unexpanded" =>
            {
                Expect_Fields(tag, &fields, HEADER_FIELDS, at)?;
                Number(fields.get(1).copied().unwrap_or_default(), "unexpanded", at)?;
                seen_header = true;
            }
            "item" =>
            {
                Expect_Fields(tag, &fields, ITEM_FIELDS, at)?;
                Number(fields.get(1).copied().unwrap_or_default(), "ordinal", at)?;

                let kind = fields.get(2).copied().unwrap_or_default();
                let visibility = fields.get(3).copied().unwrap_or_default();
                let qualified = fields.get(4).copied().unwrap_or_default();

                if kind == FUNCTION && visibility != NOT_APPLICABLE
                {
                    let name = qualified.rsplit("::").next().unwrap_or(qualified);
                    if name.starts_with(CHECK_PREFIX)
                    {
                        names.insert(name.to_owned());
                    }
                }
            }
            other =>
            {
                return Err(PayloadRefusal::UnknownRecord {
                    tag: other.to_owned(),
                    line: at,
                });
            }
        }
    }

    if !seen_header
    {
        return Err(PayloadRefusal::NoHeader);
    }

    return Ok(names);
}

/// Refuses a record whose field count is not the one this build reads.
fn Expect_Fields(
    tag: &str,
    fields: &[&str],
    expected: usize,
    line: usize,
) -> Result<(), PayloadRefusal>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(PayloadRefusal::WrongFieldCount {
        tag: tag.to_owned(),
        expected,
        found: fields.len(),
        line,
    });
}

/// Refuses a field that must be a number and is not.
///
/// The value itself is discarded. This reader needs neither the ordinal nor the count of
/// unexpanded regions; what it needs is for a record it cannot parse to stop the read
/// rather than to be walked past, because a payload half of which decodes is a check index
/// that is silently short.
fn Number(value: &str, field: &'static str, line: usize) -> Result<(), PayloadRefusal>
{
    if value.parse::<u32>().is_ok()
    {
        return Ok(());
    }

    return Err(PayloadRefusal::UnreadableNumber {
        field,
        value: value.to_owned(),
        line,
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Names(payload: &str) -> BTreeSet<String>
    {
        return Check_Names_In(payload.as_bytes()).expect("this payload is well formed");
    }

    /// The positive control, and it is not optional. Every negative control below is
    /// satisfied by a decoder that decodes nothing at all, so the set this reader is
    /// supposed to produce has to be asserted somewhere.
    #[test]
    fn Test_A_Test_Function_In_A_Test_Module_Should_Be_A_Check()
    {
        let names = Names(
            "unexpanded\t0\n\
             item\t0\tModule\tPrivate\ttests\n\
             item\t1\tFunction\tPrivate\ttests::Test_Something_Should_Hold\n",
        );

        assert!(names.contains("Test_Something_Should_Hold"), "{names:?}");
        assert_eq!(names.len(), 1, "only the function is a check: {names:?}");
    }

    /// A check declared in an `impl` block arrives qualified by the self type. Matching
    /// the qualified form would resolve nothing, because every check in this workspace
    /// lives inside something.
    #[test]
    fn Test_A_Check_In_An_Implementation_Should_Be_Found_By_Its_Own_Name()
    {
        let names = Names("unexpanded\t0\nitem\t0\tFunction\tPublic\tTable::Test_Every_Row\n");

        assert!(names.contains("Test_Every_Row"), "{names:?}");
    }

    /// A signature is not a check that runs. The payload's only mark on one is
    /// `NotApplicable`, and resolving a claim of coverage against it would let a universe
    /// name a trait method nobody implements.
    #[test]
    fn Test_A_Trait_Method_Signature_Should_Not_Be_A_Check()
    {
        let names = Names(
            "unexpanded\t0\n\
             item\t0\tTrait\tPublic\tJudged\n\
             item\t1\tFunction\tNotApplicable\tJudged::Test_Declared_Only\n",
        );

        assert!(names.is_empty(), "a trait method signature resolved a claim: {names:?}");
    }

    /// A file that genuinely declares nothing is a header and no records, and that is a
    /// real answer rather than a failed read.
    #[test]
    fn Test_A_File_That_Declares_Nothing_Should_Decode_To_No_Names()
    {
        assert!(Names("unexpanded\t0\n").is_empty());
    }

    /// The negative control this module exists for. Four shapes of unreadable payload,
    /// and none of them may arrive at the caller as an empty set — a decoder that returns
    /// "no items" for bytes it could not read makes a subject that was never read
    /// indistinguishable from a subject that declared nothing.
    #[test]
    fn Test_A_Payload_This_Build_Cannot_Read_Should_Not_Decode_To_No_Items()
    {
        assert_eq!(Check_Names_In(&[0xFF, 0xFE]), Err(PayloadRefusal::NotUtf8));
        assert_eq!(Check_Names_In(b""), Err(PayloadRefusal::NoHeader));
        assert_eq!(
            Check_Names_In(b"item\t0\tFunction\tPublic\tTest_X\n"),
            Err(PayloadRefusal::NoHeader)
        );

        let unknown = Check_Names_In(b"unexpanded\t0\nregion\t0\t3\n")
            .expect_err("a record tag this build does not know must refuse");
        assert!(
            matches!(unknown, PayloadRefusal::UnknownRecord { ref tag, line: 2 } if tag == "region"),
            "{unknown:?}"
        );

        let short = Check_Names_In(b"unexpanded\t0\nitem\t0\tFunction\tTest_X\n")
            .expect_err("a record with the wrong field count must refuse");
        assert!(
            matches!(
                short,
                PayloadRefusal::WrongFieldCount {
                    found: 4,
                    expected: 5,
                    line: 2,
                    ..
                }
            ),
            "{short:?}"
        );

        let unnumbered = Check_Names_In(b"unexpanded\tmany\n")
            .expect_err("a count that is not a number must refuse");
        assert!(
            matches!(unnumbered, PayloadRefusal::UnreadableNumber { field: "unexpanded", .. }),
            "{unnumbered:?}"
        );
    }

    /// Every refusal says where and what, because a caller told only that something
    /// failed cannot act on it — and this text reaches a finding's summary.
    #[test]
    fn Test_A_Refusal_Should_Say_What_It_Refused()
    {
        let refusal = Check_Names_In(b"unexpanded\t0\nregion\t0\n")
            .expect_err("an unknown tag refuses");

        let described = refusal.Describe();

        assert!(described.contains("region"), "{described}");
        assert!(described.contains('2'), "{described}");
    }
}
