//! Judging an already-decoded syntax payload against the `Pascal_Snake_Case` convention.
//!
//! A pure function of an already-decoded payload, so the naming judgment itself is
//! testable against hand-written fixture text the way [`crate::facts::Check_Names_In`]
//! is — no registry, no store, no reader.

use nomos_cap_syntax::{PayloadItem, SyntaxPayload, FUNCTION, IMPLEMENTATION, TRAIT};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// The one literal exemption: every binary's entry point is spelled `main`, fixed by the
/// language rather than by this workspace's naming choice.
const MAIN: &str = "main";

/// Every function `payload` declares that does not conform, as findings.
#[must_use]
pub(super) fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, item) in payload.items.iter().enumerate()
    {
        let is_out_of_scope = item.kind != FUNCTION || item.Own_Name() == MAIN || Is_Trait_Method(payload, index);
        if is_out_of_scope
        {
            continue;
        }

        if !Is_Pascal_Snake_Case(item.Own_Name())
        {
            let violation = Violation_Finding(path, item);
            findings.push(violation);
        }
    }

    return findings;
}

/// Whether the item at `ordinal` is declared inside a trait implementation.
///
/// Its name is fixed by the trait it implements — often a foreign one, such as `Display`
/// or `Iterator` — and the compiler forces an exact match regardless of this workspace's
/// own convention. The same [`SyntaxPayload::Enclosing`] and shape check
/// `crate::universe_kind::Enumeration_Universe` already uses to tell an inherent `impl` from a
/// trait one.
fn Is_Trait_Method(payload: &SyntaxPayload, ordinal: usize) -> bool
{
    let Some(owner) = payload.Enclosing(ordinal)
    else
    {
        return false;
    };

    return owner.kind == IMPLEMENTATION && owner.shape.Value() == Some(TRAIT);
}

/// Whether `name` is `Pascal_Snake_Case`: every `_`-separated segment starts with an
/// uppercase ASCII letter, or is entirely ASCII digits (`Test_CHK_003_...` is real in this
/// workspace and its numeric segment is not a casing violation).
///
/// A single leading underscore is stripped first — Rust's own convention for "intentionally
/// unused," orthogonal to this workspace's casing choice and not something `README.md`'s
/// Conventions section speaks to.
#[must_use]
fn Is_Pascal_Snake_Case(name: &str) -> bool
{
    let name = name.strip_prefix('_').unwrap_or(name);

    if name.is_empty()
    {
        return false;
    }

    return name.split('_').all(|segment| {
        if segment.is_empty()
        {
            return false;
        }

        if segment.chars().all(|character| return character.is_ascii_digit())
        {
            return true;
        }

        return segment.chars().next().is_some_and(|first| return first.is_ascii_uppercase());
    });
}

/// A finding for one function whose name does not conform.
fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(super::NAMING_CONVENTION),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "`{}` is not Pascal_Snake_Case: README.md's Conventions section requires \
             function names to be Pascal_Snake_Case, and Cargo.toml disables rustc's own \
             non_snake_case lint specifically because this workspace uses a different \
             convention — nothing else was checking it.",
            item.Own_Name()
        ),
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    mod casing
    {
        use super::Is_Pascal_Snake_Case;

        #[test]
        fn Test_A_Single_Word_Starting_Uppercase_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("New"));
        }

        #[test]
        fn Test_Two_Segments_Both_Uppercase_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("As_Str"));
        }

        #[test]
        fn Test_A_Numeric_Segment_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("Test_CHK_003_Something_Should_Hold"));
        }

        #[test]
        fn Test_Ordinary_Lower_Snake_Case_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("as_str"));
        }

        #[test]
        fn Test_A_Lowercase_Second_Segment_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("Foo_bar"));
        }

        #[test]
        fn Test_A_Single_Leading_Underscore_Should_Be_Stripped_Before_Judging()
        {
            assert!(Is_Pascal_Snake_Case("_Unused"));
            assert!(!Is_Pascal_Snake_Case("_unused"));
        }

        #[test]
        fn Test_A_Double_Underscore_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("Foo__Bar"));
        }

        #[test]
        fn Test_An_Empty_Name_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case(""));
        }
    }

    mod scanning
    {
        use super::{GateCategory, SyntaxPayload, Violations_In};

        /// Names conforming to `Pascal_Snake_Case` under every shape `casing`'s own table
        /// exercises: a bare word, two segments, a numeric segment, and a stripped leading
        /// underscore.
        fn Conforming_Function_Names() -> Vec<&'static str>
        {
            return vec!["Good_Name", "New", "As_Str", "Test_CHK_003_Something_Should_Hold", "_Unused"];
        }

        #[test]
        fn Test_A_Conforming_Function_Should_Produce_No_Finding()
        {
            for name in Conforming_Function_Names()
            {
                let payload = Payload_From_Text(&format!("unexpanded\t0\nitem\t0\tFunction\tPublic\t{name}\t.\t+fn/0\n"));

                let findings = Violations_In(&payload, "src/lib.rs");

                assert!(findings.is_empty(), "{name}: {findings:?}");
            }
        }

        #[test]
        fn Test_Violations_In_Should_Produce_One_Finding_For_A_Non_Conforming_Function()
        {
            let payload = Payload_From_Text(
                "unexpanded\t0\n\
                 item\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert_eq!(findings.len(), 1, "{findings:?}");
            let found = findings.first().expect("asserted len 1 above");
            assert_eq!(found.subject_name, "bad_name");
            assert_eq!(found.gate, GateCategory::Advisory);
        }

        /// Qualified names whose *own* name is `main`, for [`Test_Main_Should_Be_Exempt`] —
        /// visibility and nesting vary; the exemption is keyed on the item's own name alone.
        fn Main_Qualified_Names() -> Vec<(&'static str, &'static str)>
        {
            return vec![
                ("Private", "main"),
                ("Public", "main"),
                ("Private", "tests::main"),
                ("Public", "examples::demo::main"),
            ];
        }

        #[test]
        fn Test_Main_Should_Be_Exempt()
        {
            for (visibility, qualified_name) in Main_Qualified_Names()
            {
                let payload = Payload_From_Text(&format!("unexpanded\t0\nitem\t0\tFunction\t{visibility}\t{qualified_name}\t.\t+fn/0\n"));

                let findings = Violations_In(&payload, "src/main.rs");

                assert!(findings.is_empty(), "{qualified_name}: {findings:?}");
            }
        }

        /// Foreign trait/method pairs whose method name the trait fixes, for
        /// [`Test_A_Trait_Methods_Non_Conforming_Name_Should_Be_Exempt`] — the compiler forces
        /// each of these exact spellings regardless of this workspace's own convention.
        fn Foreign_Trait_Methods() -> Vec<(&'static str, &'static str)>
        {
            return vec![("Display", "fmt"), ("Iterator", "next"), ("Clone", "clone"), ("Drop", "drop")];
        }

        #[test]
        fn Test_A_Trait_Methods_Non_Conforming_Name_Should_Be_Exempt()
        {
            for (trait_name, method) in Foreign_Trait_Methods()
            {
                let payload = Payload_From_Text(&format!(
                    "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\t{trait_name}\t.\t+trait\nitem\t1\tFunction\tPublic\t{trait_name}::{method}\t.\t+fn/1\n"
                ));

                let findings = Violations_In(&payload, "src/lib.rs");

                assert!(findings.is_empty(), "{trait_name}::{method}: a trait method's fixed name was judged: {findings:?}");
            }
        }

        #[test]
        fn Test_An_Inherent_Methods_Non_Conforming_Name_Should_Not_Be_Exempt()
        {
            let payload = Payload_From_Text(
                "unexpanded\t0\n\
                 item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tFunction\tPublic\tTable::bad_name\t.\t+fn/1\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert_eq!(findings.len(), 1, "an inherent method's own name was exempted: {findings:?}");
        }

        #[test]
        fn Test_A_File_Declaring_Nothing_Should_Produce_No_Finding()
        {
            let payload = Payload_From_Text("unexpanded\t0\n");

            assert!(Violations_In(&payload, "src/lib.rs").is_empty());
        }

        fn Payload_From_Text(text: &str) -> SyntaxPayload
        {
            return nomos_cap_syntax::Parse_Payload(text.as_bytes())
                .expect("this fixture payload is well formed");
        }
    }
}
