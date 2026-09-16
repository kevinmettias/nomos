//! Judging an already-decoded syntax payload against a resolved naming [`Case`].
//!
//! A pure function of an already-decoded payload, so the naming judgment itself is
//! testable against hand-written fixture text the way [`crate::facts::Check_Names_In`]
//! is — no registry, no store, no reader.

use crate::checks::finding_shape::Qualified_Name_Finding;
use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::{PayloadItem, SyntaxPayload, FUNCTION, IMPLEMENTATION, Impl_Serves_A_Trait};
use nomos_contracts::{Finding, GateCategory};

/// The one literal exemption: every binary's entry point is spelled `main`, fixed by the
/// language rather than by this workspace's naming choice.
const MAIN: &str = "main";

/// Every function `payload` declares that does not conform to `case`, as findings.
#[must_use]
pub(super) fn Violations_In(payload: &SyntaxPayload, path: &str, case: Case) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, item) in payload.items.iter().enumerate()
    {
        let is_out_of_scope = item.kind != FUNCTION || item.Own_Name() == MAIN || Is_Trait_Method(payload, index);
        if is_out_of_scope
        {
            continue;
        }

        if !Conforms(case, item.Own_Name())
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

    // Read through the typed reader rather than compared against `TRAIT`, for the reason
    // `abbreviations.rs`'s own carry gives: since `OD-CAPABILITY-014` an `impl` block's shape
    // carries its generic type parameters behind that label, and an equality test against the
    // bare constant reads `impl<T> Display for T`'s own methods as ordinary declarations.
    return owner.kind == IMPLEMENTATION && Impl_Serves_A_Trait(&owner.shape) == Some(true);
}

/// A finding for one function whose name does not conform.
fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    return Qualified_Name_Finding(
        super::NAMING_CONVENTION,
        path,
        item,
        GateCategory::Advisory,
        format!(
            "`{}` is not Pascal_Snake_Case: README.md's Conventions section requires \
             function names to be Pascal_Snake_Case, and Cargo.toml disables rustc's own \
             non_snake_case lint specifically because this workspace uses a different \
             convention — nothing else was checking it.",
            item.Own_Name()
        ),
    );
}

/// Whether `name` conforms to `case`.
///
/// A single leading underscore is stripped first — Rust's own convention for "intentionally
/// unused," orthogonal to whichever case a repository configures and not something
/// `README.md`'s Conventions section speaks to.
#[must_use]
fn Conforms(case: Case, name: &str) -> bool
{
    let name = name.strip_prefix('_').unwrap_or(name);

    return case.Conforms(name);
}

#[cfg(test)]
mod tests
{
    use super::*;

    mod casing
    {
        use super::Conforms;
        use nomos_cap_naming_policy::Case;

        /// Exhaustive `Case::UpperSnake` behavior — every segment shape, digits, empty
        /// names — is `nomos-cap-naming-policy`'s own test coverage now; this module keeps
        /// only what is local to this crate's own composition: the leading-underscore
        /// strip layered on top of whichever case is resolved.
        #[test]
        fn Test_A_Single_Leading_Underscore_Should_Be_Stripped_Before_Judging()
        {
            assert!(Conforms(Case::UpperSnake, "_Unused"));
            assert!(!Conforms(Case::UpperSnake, "_unused"));
        }

        #[test]
        fn Test_Conforms_Should_Delegate_To_Whichever_Case_Is_Resolved()
        {
            assert!(Conforms(Case::LowerSnake, "as_str"));
            assert!(!Conforms(Case::UpperSnake, "as_str"));
        }
    }

    mod scanning
    {
        use super::{GateCategory, SyntaxPayload, Violations_In};
        use nomos_cap_naming_policy::Case;

        #[test]
        fn Test_A_Conforming_Function_Should_Produce_No_Finding()
        {
            for name in Conforming_Function_Names()
            {
                let payload = Payload_From_Text(&format!("unexpanded\t0\nitem\t0\tFunction\tPublic\t{name}\t.\t+fn/0\n"));

                let findings = Violations_In(&payload, "src/lib.rs", Case::UpperSnake);

                assert!(findings.is_empty(), "{name}: {findings:?}");
            }
        }

        /// Names conforming to `Pascal_Snake_Case` under every shape `casing`'s own table
        /// exercises: a bare word, two segments, a numeric segment, and a stripped leading
        /// underscore.
        fn Conforming_Function_Names() -> Vec<&'static str>
        {
            return vec!["Good_Name", "New", "As_Str", "Test_CHK_003_Something_Should_Hold", "_Unused"];
        }

        #[test]
        fn Test_Violations_In_Should_Produce_One_Finding_For_A_Non_Conforming_Function()
        {
            let payload = Payload_From_Text(
                "unexpanded\t0\n\
                 item\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs", Case::UpperSnake);

            assert_eq!(findings.len(), 1, "{findings:?}");
            let found = findings.first().expect("asserted len 1 above");
            assert_eq!(found.subject_name, "bad_name");
            assert_eq!(found.gate, GateCategory::Advisory);
        }

        #[test]
        fn Test_Main_Should_Be_Exempt()
        {
            for (visibility, qualified_name) in Main_Qualified_Names()
            {
                let payload = Payload_From_Text(&format!("unexpanded\t0\nitem\t0\tFunction\t{visibility}\t{qualified_name}\t.\t+fn/0\n"));

                let findings = Violations_In(&payload, "src/main.rs", Case::UpperSnake);

                assert!(findings.is_empty(), "{qualified_name}: {findings:?}");
            }
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
        fn Test_A_Trait_Methods_Non_Conforming_Name_Should_Be_Exempt()
        {
            for (trait_name, method) in Foreign_Trait_Methods()
            {
                let payload = Payload_From_Text(&format!(
                    "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\t{trait_name}\t.\t+trait\nitem\t1\tFunction\tPublic\t{trait_name}::{method}\t.\t+fn/1\n"
                ));

                let findings = Violations_In(&payload, "src/lib.rs", Case::UpperSnake);

                assert!(findings.is_empty(), "{trait_name}::{method}: a trait method's fixed name was judged: {findings:?}");
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
        fn Test_An_Inherent_Methods_Non_Conforming_Name_Should_Not_Be_Exempt()
        {
            let payload = Payload_From_Text(
                "unexpanded\t0\n\
                 item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tFunction\tPublic\tTable::bad_name\t.\t+fn/1\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs", Case::UpperSnake);

            assert_eq!(findings.len(), 1, "an inherent method's own name was exempted: {findings:?}");
        }

        #[test]
        fn Test_A_File_Declaring_Nothing_Should_Produce_No_Finding()
        {
            let payload = Payload_From_Text("unexpanded\t0\n");

            assert!(Violations_In(&payload, "src/lib.rs", Case::UpperSnake).is_empty());
        }

        /// `OD-CAPABILITY-014` put a variable-length body behind an `impl` block's own
        /// trait-or-inherent label, so [`Is_Trait_Method`] stopped being an equality test
        /// against [`nomos_cap_syntax::TRAIT`]. `impl<T> Display for T`'s own `fmt` is the
        /// shape that proves it: read the bare constant and a name the trait fixed is
        /// reported as one this repository chose.
        #[test]
        fn Test_A_Generic_Trait_Impls_Own_Method_Should_Stay_Exempt()
        {
            let payload = Payload_From_Text(
                "unexpanded\t0\n\
                 item\t0\tImplementation\tNotApplicable\tT\t.\t+trait\\ngenerics\\nT\n\
                 item\t1\tFunction\tNotApplicable\tT::fmt\t.\t+fn/2\n",
            );

            assert!(
                Violations_In(&payload, "src/lib.rs", Case::UpperSnake).is_empty(),
                "the trait fixed fmt, whether or not the impl is generic"
            );
        }

        fn Payload_From_Text(text: &str) -> SyntaxPayload
        {
            return nomos_cap_syntax::Parse_Payload(text.as_bytes())
                .expect("this fixture payload is well formed");
        }
    }
}
