//! Which of a file's declarations are checks, read out of a syntax fact.
//!
//! The payload is `nomos.syntax.items.v1` and this module no longer decodes it.
//! `nomos-cap-syntax` carries the grammar and the one reader every consumer uses; what is
//! left here is the part that is this crate's own — deciding which of the items a file
//! declares is a check that runs.
//!
//! # This used to be the third reader
//!
//! Both providers author these bytes, `tests/integration` read them back, and so did this
//! module, each with its own idea of what a well-formed payload is. Three answers to that
//! question is how a payload comes to decode differently under one capability. The reader
//! moved to the contract crate under `P10-SYNTAX-SCHEMA`, where it belongs for the same
//! reason the ceiling and the schema identifier already lived there: an agreement is not
//! the property of one party to it, and a consumer is a party.
//!
//! # Why a refusal and not an empty set
//!
//! Unchanged, and it is the reason the reader is fallible at all. A decoder that returns
//! "no items" for bytes it could not read reproduces one layer up the exact defect
//! [`crate::Reading::Unparseable`] exists to prevent one layer down: a subject that was
//! never read rendering identically to a subject that declared nothing. Every path out of
//! [`Check_Names_In`] that is not a successfully decoded payload is a refusal the caller
//! has to handle, and the caller turns it into a finding.

use nomos_cap_syntax::{Parse_Payload, PayloadRefusal, FUNCTION};
use std::collections::BTreeSet;

/// The prefix that makes a function a check, by this workspace's naming convention.
const CHECK_PREFIX: &str = "Test_";

/// Every check name the payload declares.
///
/// A check is a test function by this workspace's naming convention: a `Function` item
/// whose own name begins with `Test_`. Two filters, and each one is load-bearing.
///
/// The name is the item's **own** name rather than its qualified one, because the payload
/// qualifies an item by its syntactic nesting — a check written in `mod tests` arrives as
/// `tests::Test_X` and one written in an `impl` block arrives as `Table::Test_X`. Matching
/// the qualified form against a universe's claimed mirror would resolve neither, and every
/// check in this workspace lives inside something.
///
/// The visibility filter drops trait-method signatures. A `fn Test_X(&self);` declared in a
/// trait runs nothing and must not resolve a claim of coverage, and the mark the schema
/// reserves for a form that declares no visibility is what identifies one.
///
/// **That filter is sound only because of the floor this crate requires.** The schema
/// states what the mark means and explicitly declines to state that its absence means
/// anything: a provider that cannot see an enclosing trait writes `Private` for the same
/// signature, and its payload conforms. So this reads an observation only a sufficiently
/// strong provider makes, and `OD-RULES-001`'s guarantee floor is what keeps the blind
/// spelling from ever reaching here. A caller that lowered the floor would not get a
/// slightly weaker answer; it would get signatures counted as checks.
///
/// # Errors
///
/// [`PayloadRefusal`] for anything the schema's reader cannot read. Never an empty set
/// standing in for a failed read.
pub(crate) fn Check_Names_In(bytes: &[u8]) -> Result<BTreeSet<String>, PayloadRefusal>
{
    let payload = Parse_Payload(bytes)?;
    let mut names: BTreeSet<String> = BTreeSet::new();

    for item in &payload.items
    {
        if item.kind != FUNCTION || item.Declares_No_Visibility()
        {
            continue;
        }

        let name = item.Own_Name();
        if name.starts_with(CHECK_PREFIX)
        {
            names.insert(name.to_owned());
        }
    }

    return Ok(names);
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
    /// satisfied by a filter that admits nothing at all, so the set this is supposed to
    /// produce has to be asserted somewhere.
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

    /// A signature is not a check that runs. Resolving a claim of coverage against one
    /// would let a universe name a trait method nobody implements.
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

    /// The property this layer must not lose when the decoding moved out of it.
    ///
    /// The shapes of unreadable payload are the schema's to enumerate and it does. What is
    /// asserted here is that a refusal still arrives at *this* caller as a refusal: the
    /// defect being prevented is a check index that is silently short, and it would come
    /// back the moment this function swallowed one.
    #[test]
    fn Test_A_Payload_This_Build_Cannot_Read_Should_Not_Decode_To_No_Checks()
    {
        assert_eq!(Check_Names_In(&[0xFF, 0xFE]), Err(PayloadRefusal::NotUtf8));
        assert_eq!(Check_Names_In(b""), Err(PayloadRefusal::NoHeader));

        let unknown = Check_Names_In(b"unexpanded\t0\nregion\t0\t3\n")
            .expect_err("a record tag this build does not know must refuse");
        assert!(unknown.Describe().contains("region"), "{unknown:?}");
    }
}
