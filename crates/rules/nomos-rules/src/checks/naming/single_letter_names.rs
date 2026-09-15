//! Single-letter names are forbidden where the syntax payload can see them.
//!
//! code-standards also governs locals, parameters, generic parameters and closure
//! arguments. The current syntax payload exposes declared item names and named struct
//! fields, so this rule judges that subset and leaves the rest to a richer syntax shape.
//!
//! An `impl` block whose own recorded name is one of the generic parameters that same block
//! declares is exempt, and it is the reason the exemption above is not enough on its own. The
//! walker records an `Implementation` item's name as the head of its self type, so
//! `impl<T: AsRef<[u8]>> ToHex for T` arrives here as an item named `T` -- the block's own
//! already-declared parameter, reached a second time, not an identifier anybody chose for a
//! declaration. `impl Trait for X` over a real, one-letter-named struct `X` is still reported,
//! which is why this asks the payload what the block declared rather than exempting every
//! single-letter `impl` by the shape of its name: `OD-CAPABILITY-014` put that list in the
//! `shape` field precisely because no name-only heuristic can tell the two apart.
//!
//! A `use` binding is exempt the same way [`Check_Abbreviations`]'s own does: its name was
//! chosen wherever the thing it imports was declared, not here, and for a wildcard import
//! that "name" is not a declared identifier at all -- `*`, the payload's own glob token, a
//! single character nobody authored. `_` is exempt everywhere, not only there: `const _: ()
//! = assert!(...);` is Rust's own idiom for a compile-time check nobody names, the same
//! discard token a wildcard-adjacent `use` binding can also carry.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{IMPLEMENTATION, Impl_Generics, PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const SINGLE_LETTER_NAMES: &str = "single-letter-names";

const STRUCT: &str = "Struct";

/// The payload's `kind` for a `use` binding.
///
/// Named locally rather than imported for the same reason [`Check_Abbreviations`]'s own
/// copy is: `nomos-cap-syntax` publishes an *open* kind vocabulary and exports a constant
/// only for the labels its own API needs, so a rule that cares about a third one states the
/// literal it is matching.
///
/// A `use` binding's own `Own_Name()` is not a declared identifier at all: for a wildcard
/// import (`use path::*;`) it is the literal glob token `*`, and for a discard-shaped import
/// it is `_` -- both single characters, and neither one this repository chose. Judging them
/// was 657 of this rule's own findings against this workspace, all of them one or the other.
const USE_BINDING: &str = "Use";

/// Reports declared item names and named struct fields that are a single character.
#[must_use]
pub fn Check_Single_Letter_Names(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for item in &payload.items
    {
        let violations = Item_Violations_In(path, item);
        findings.extend(violations);
    }

    return findings;
}

/// One item's own violations: skipped entirely if it is a `use` binding (a name chosen
/// wherever the binding's target was declared, not here), otherwise its own name -- unless
/// that name is one of its own generic parameters, see [`Names_Its_Own_Generic_Parameter`] --
/// and (for a struct) its fields, against the single-letter rule.
fn Item_Violations_In(path: &str, item: &PayloadItem) -> Vec<Finding>
{
    if item.kind == USE_BINDING
    {
        return Vec::new();
    }

    let mut findings = Vec::new();

    if Is_Single_Letter(item.Own_Name()) && !Names_Its_Own_Generic_Parameter(item)
    {
        let finding = Violation_Finding(path, item, item.Own_Name());
        findings.push(finding);
    }

    if item.kind == STRUCT
    {
        let field_violations = Field_Violations_In(path, item);
        findings.extend(field_violations);
    }

    return findings;
}

/// Whether `item` is an `impl` block whose own recorded name is one of the generic
/// parameters that same block declares.
///
/// Asked of the payload rather than guessed from the name. `OD-CAPABILITY-014` measured the
/// alternative and rejected it: exempting every `Implementation` item whose name is a single
/// upper-case letter would also hide `impl Trait for X` over a real struct somebody named
/// `X`, which is the case this rule exists for. A `None` here is a `shape` no `impl` block
/// wrote, and answers `false` rather than exempting on a field it could not read.
fn Names_Its_Own_Generic_Parameter(item: &PayloadItem) -> bool
{
    if item.kind != IMPLEMENTATION
    {
        return false;
    }

    return Impl_Generics(&item.shape).is_some_and(|parameters| {
        return parameters.iter().any(|parameter| return parameter == item.Own_Name());
    });
}

fn Field_Violations_In(path: &str, item: &PayloadItem) -> Vec<Finding>
{
    let Some(fields) = Struct_Fields(&item.shape)
    else
    {
        return Vec::new();
    };

    return fields
        .iter()
        .filter(|(name, _type_name)| return Is_Single_Letter(name))
        .map(|(name, _type_name)| return Violation_Finding(path, item, name))
        .collect();
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(SINGLE_LETTER_NAMES);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's single-letter names could not be judged");
    return finding;
}

/// `_` is exempt regardless of what declared it: `const _: () = assert!(...);` is Rust's own
/// idiom for a compile-time check nobody references by name, not a human choosing a
/// one-letter name for brevity, and it is the only value where that distinction holds for
/// every item kind rather than only for a use binding.
fn Is_Single_Letter(name: &str) -> bool
{
    return name != "_" && name.chars().count() == 1;
}

fn Violation_Finding(path: &str, item: &PayloadItem, name: &str) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}::{name}", item.qualified_name);

    return Finding {
        rule: RuleId::New(SINGLE_LETTER_NAMES),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{name}` is a single-letter name outside the local-variable exception this payload can judge"),
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Single_Letter_Item_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tX\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "X");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Single_Letter_Struct_Field()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tPoint\t.\t+fields\\nx\\tf64\n");

        let findings = Violations_In(&payload, "src/point.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "x");
    }

    /// A wildcard import's own glob token, `*` -- 639 of this rule's 657 findings against
    /// this workspace before this exemption existed.
    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Wildcard_Use_Bindings_Own_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tUse\tPrivate\t*\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "an import names something declared elsewhere: {findings:?}");
    }

    /// A discard-shaped import's own name, `_` -- the rest of this rule's 657 findings.
    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Discard_Use_Bindings_Own_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tUse\tPrivate\t_\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `const _: () = assert!(...);` -- a real declaration, not a `use` binding, whose name
    /// is still the language's own discard token rather than a human's one-letter choice.
    /// `crates/kernel/nomos-model/src/digest.rs`'s own compile-time assertion is this shape.
    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Discard_Named_Declaration()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\t_\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Multi_Letter_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPrivate\tRun\t.\t+fn/0\n\
             item\t1\tStruct\tPublic\tPoint\t.\t+fields\\nx_coord\\tf64\n",
        );

        let findings = Violations_In(&payload, "src/point.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }


    /// `hex` 0.4.3's own `impl<T: AsRef<[u8]>> ToHex for T`, which
    /// `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON` surfaced against a real
    /// third-party crate: an entirely ordinary Rust idiom this workspace's own tree does not
    /// happen to write.
    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Blanket_Impls_Own_Generic_Parameter()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tT\t.\t+trait\\ngenerics\\nT\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "the block declared T itself: {findings:?}");
    }

    /// The case the exemption must not swallow, and the reason it reads the payload's own
    /// generic list rather than the shape of the name. `X` here is a real type somebody
    /// named, and an `impl` block over it is not where that name was chosen -- but the
    /// declaration `X` is still a single-letter name, and a rule that exempted every
    /// one-letter `impl` would report neither.
    #[test]
    fn Test_Violations_In_Should_Still_Judge_An_Impl_Over_A_Real_Single_Letter_Type()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tImplementation\tNotApplicable\tX\t.\t+trait\\ngenerics\\nT\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "X is not among the parameters this block declared: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "X");
    }

    /// An `impl` block declaring no generics at all still reaches the rule, and its own name
    /// is still judged -- the empty list is not the absent one.
    #[test]
    fn Test_Violations_In_Should_Still_Judge_A_Non_Generic_Impls_Own_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tX\t.\t+inherent\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "X");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
