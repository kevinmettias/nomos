use super::*;
use nomos_cap_syntax::Parse_Payload;

#[test]
fn Test_Universes_In_Should_Find_A_Constant_Slice()
{
    let found = Universes_In(
        "a.rs",
        &Payload_From_Records("item\t0\tConstant\tPublic\tGOVERNING_RECORD_IDS\t.\t+slice\n"),
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
        &Payload_From_Records(
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
    for trait_name in Trait_Impl_Names()
    {
        let found = Universes_In(
            "a.rs",
            &Payload_From_Records(&format!(
                "item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\nitem\t1\tImplementation\tNotApplicable\t{trait_name}\t.\t+trait\nitem\t2\tFunction\tNotApplicable\t{trait_name}::All\t.\t+fn/0\n"
            )),
        );

        assert!(found.is_empty(), "{trait_name}: {found:?}");
    }
}

/// Trait names an `All()` sits behind, for [`Test_A_Trait_Impl_Should_Not_Claim_The_Type`] —
/// whichever foreign trait it is, a trait implementation never owns the variant list.
fn Trait_Impl_Names() -> Vec<&'static str>
{
    return vec!["Other", "Display", "Iterator", "SomeCustomTrait"];
}

/// An accessor on an instance is not the type's list of itself.
#[test]
fn Test_An_All_That_Takes_A_Receiver_Should_Not_Be_A_Universe()
{
    for arity in Nonzero_All_Arities()
    {
        let found = Universes_In(
            "a.rs",
            &Payload_From_Records(&format!(
                "item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\nitem\t1\tFunction\tPublic\tTable::All\t.\t+fn/{arity}\n"
            )),
        );

        assert!(found.is_empty(), "arity {arity}: {found:?}");
    }
}

/// The highest arity [`Nonzero_All_Arities`] exercises: a receiver and two arguments. The
/// value means "one more than the two arguments this fixture needs", not merely itself.
const HIGHEST_EXERCISED_ARITY: u32 = 3;

/// Nonzero arities for `Table::All`, for
/// [`Test_An_All_That_Takes_A_Receiver_Should_Not_Be_A_Universe`] — any receiver or
/// argument at all makes it an accessor rather than the type's own list.
fn Nonzero_All_Arities() -> Vec<u32>
{
    return (1..=HIGHEST_EXERCISED_ARITY).collect();
}

/// A scalar constant is not a universe. Matching it would bury the real ones.
#[test]
fn Test_A_Scalar_Constant_Should_Not_Be_A_Universe()
{
    assert!(Universes_In(
        "a.rs",
        &Payload_From_Records("item\t0\tConstant\tPublic\tLIMIT\t.\t+value\n")
    )
    .is_empty());
}

/// A list the rest of the workspace cannot see is out of this rule's declared scope.
#[test]
fn Test_A_Private_List_Should_Not_Be_A_Universe()
{
    assert!(Universes_In(
        "a.rs",
        &Payload_From_Records("item\t0\tConstant\tPrivate\tTABLES\t.\t+slice\n")
    )
    .is_empty());
}

#[test]
fn Test_A_Declared_Mirror_Should_Be_Read_Off_The_Documentation_Comment()
{
    let found = Universes_In(
        "a.rs",
        &Payload_From_Records(
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
        &Payload_From_Records("item\t0\tConstant\tPublic\tTABLES\t.\t+slice\n"),
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
    let blind = Payload_From_Records("item\t0\tConstant\tPublic\tTABLES\t-\t-\n");

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
fn Test_Read_Universes_Should_Observe_A_File_That_Declares_Nothing_As_Empty()
{
    assert_eq!(Read_Universes("a.rs", &Payload_From_Records("")), Reading::Observed(Vec::new()));
}

/// `OD-CAPABILITY-014` put a variable-length body behind an `impl` block's own
/// trait-or-inherent label, so this attribution stopped being an equality test against
/// [`nomos_cap_syntax::INHERENT`]. `impl<T> Holder<T>` is the shape that proves it: read
/// the bare constant and a generic type's own declared universe stops being seen at all,
/// which is an absence becoming a clean result rather than a finding.
#[test]
fn Test_A_Generic_Inherent_Impl_Should_Still_Own_Its_Types_Variant_List()
{
    let found = Universes_In(
        "a.rs",
        &Payload_From_Records(
            "item\t0\tImplementation\tNotApplicable\tHolder\t.\t+inherent\\ngenerics\\nT\n\
             item\t1\tFunction\tPublic\tHolder::All\t.\t+fn/0\n",
        ),
    );

    assert_eq!(
        found.first().map(|universe| universe.name.clone()),
        Some("Holder::All".to_owned())
    );
}

/// A payload built from item records, so a fixture reads as the bytes a provider wrote.
fn Payload_From_Records(records: &str) -> SyntaxPayload
{
    return Parse_Payload(format!("unexpanded\t0\n{records}").as_bytes())
        .expect("the fixture is written in the schema");
}
