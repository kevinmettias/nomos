use super::*;

#[test]
fn Test_Split_Identifier_Words_Should_Split_On_Case_Boundaries()
{
    assert_eq!(Split_Identifier_Words("computeTotal"), vec!["compute", "total"]);
    assert_eq!(Split_Identifier_Words("ComputeTotal"), vec!["compute", "total"]);
}

#[test]
fn Test_Split_Identifier_Words_Should_Split_On_Underscores_And_Hyphens()
{
    assert_eq!(Split_Identifier_Words("compute_total"), vec!["compute", "total"]);
    assert_eq!(Split_Identifier_Words("compute-total"), vec!["compute", "total"]);
}

#[test]
fn Test_Split_Identifier_Words_Should_Drop_A_Digit_Only_Run()
{
    // Not `Vector2` — despite the upstream doc comment's claim, `insert_Case_Boundaries`
    // never separates a letter run from a digit run that follows it directly (only an
    // upper-case letter following a lower/digit gets a boundary), so `Vector2` stays one
    // field, `vector2`, all the way through. A digit-only run is only its own field, and
    // so only dropped, when something else already separates it: an explicit `_`/`-`, or
    // a case boundary on its OTHER side.
    assert_eq!(Split_Identifier_Words("Vector2"), vec!["vector2"]);
    assert_eq!(Split_Identifier_Words("item_2"), vec!["item"]);
}

#[test]
fn Test_Split_Identifier_Words_Should_Keep_An_Acronym_Run_As_One_Segment()
{
    assert_eq!(Split_Identifier_Words("HTTP"), vec!["http"]);
}

#[test]
fn Test_Abbreviation_Reason_Should_Flag_A_Vowelless_Word()
{
    assert_eq!(Abbreviation_Reason("ctx", &[]), Some("no vowels — an abbreviation; spell it out"));
}

#[test]
fn Test_Abbreviation_Reason_Should_Flag_A_Banned_Word()
{
    assert_eq!(Abbreviation_Reason("msg", &[]), Some("known abbreviation; spell it out"));
}

#[test]
fn Test_Abbreviation_Reason_Should_Accept_A_Default_Approved_Word()
{
    assert_eq!(Abbreviation_Reason("json", &[]), None);
}

#[test]
fn Test_Abbreviation_Reason_Should_Accept_A_Word_Approved_By_A_Repository_Addition()
{
    assert_eq!(Abbreviation_Reason("kwb", &["kwb".to_owned()]), None);
}

#[test]
fn Test_Abbreviation_Reason_Should_Accept_A_Digit_Bearing_Word_Whose_Letter_Core_Has_A_Vowel()
{
    assert_eq!(Abbreviation_Reason("utf8", &[]), None, "utf is approved, so utf8 passes");
    assert_eq!(Abbreviation_Reason("sha256", &[]), None, "sha has a vowel");
}

#[test]
fn Test_Abbreviation_Reason_Should_Flag_A_Digit_Bearing_Word_Whose_Letter_Core_Is_Vowelless()
{
    assert_eq!(
        Abbreviation_Reason("gp400", &[]),
        Some("no vowels — an abbreviation; spell it out"),
        "gp has no vowel even with the digits stripped"
    );
}

/// `md` is itself in the default approved list (`.md`, the Markdown extension), so
/// `md5` is exempt by the same "approved core" reasoning that clears `utf8`/`sha256` —
/// not by the vowel rule, which would flag `md` (no vowel) on its own. Upstream's own
/// doc comment on `abbreviation_Reason` names `md5` as an example the vowel rule
/// catches, which no longer holds once `md` was added to `default_Approved_Words` — a
/// stale-doc drift the same shape as this session's other doc-vs-implementation finds,
/// caught here by tracing the real data rather than the comment.
#[test]
fn Test_Abbreviation_Reason_Should_Accept_Md5_Because_Its_Core_Is_Approved()
{
    assert_eq!(Abbreviation_Reason("md5", &[]), None);
}

#[test]
fn Test_Abbreviation_Reason_Should_Accept_An_Ordinary_Voweled_Word()
{
    assert_eq!(Abbreviation_Reason("total", &[]), None);
}

#[test]
fn Test_Abbreviation_Reason_Should_Ignore_A_Single_Character_Word()
{
    assert_eq!(Abbreviation_Reason("x", &[]), None);
}

#[test]
fn Test_Violations_In_Should_Report_A_Vowelless_Item_Name()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tCtx\t.\t+fn/0\n");

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Ctx");
}

#[test]
fn Test_Violations_In_Should_Report_A_Banned_Struct_Field()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t+fields\\nmsg\\tString\\nbody\\tString\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "msg");
}

/// The carve-out code-standards' own `check-generic-parameter-name` states and this rule
/// was missing: a declaration whose name was not the author's is not the author's to
/// answer for. `fmt` is `core::fmt::Display`'s required method name, and it was 104 of
/// the 148 findings this rule reported against this workspace before this exemption.
#[test]
fn Test_Violations_In_Should_Not_Judge_A_Name_A_Trait_Fixed()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tTable\t.\t+trait\n\
         item\t1\tFunction\tPrivate\tTable::fmt\t.\t+fn/2\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert!(findings.is_empty(), "a trait fixes what its implementers may call things: {findings:?}");
}

/// The other half, and the reason the exemption reads the block's shape rather than
/// merely noticing that a name is nested: an inherent `impl` chose its own method names,
/// so the identical qualified name is still judged there.
#[test]
fn Test_Violations_In_Should_Judge_An_Inherent_Impls_Own_Member()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
         item\t1\tFunction\tPrivate\tTable::fmt\t.\t+fn/2\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "an inherent impl picked this name itself: {findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "fmt");
}

/// Two blocks for one type are indistinguishable by qualified name, so the exemption has
/// to end at the next `impl` rather than at the next differently-named item.
#[test]
fn Test_Violations_In_Should_Stop_Exempting_At_The_Next_Impl_Block()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tTable\t.\t+trait\n\
         item\t1\tFunction\tPrivate\tTable::fmt\t.\t+fn/2\n\
         item\t2\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
         item\t3\tFunction\tPrivate\tTable::ctx\t.\t+fn/1\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "only the inherent block's own member is judged: {findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "ctx");
}

/// A later sibling at the same level does not carry the block's prefix, so it falls out
/// of the exemption with nothing having to pop it.
#[test]
fn Test_Violations_In_Should_Still_Judge_A_Sibling_After_A_Trait_Impl()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tTable\t.\t+trait\n\
         item\t1\tFunction\tPrivate\tTable::fmt\t.\t+fn/2\n\
         item\t2\tFunction\tPrivate\tCtx\t.\t+fn/0\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Ctx");
}

/// The second name-nobody-chose exemption, found by composing the rule into a real run
/// and measuring: 144 of 327 findings against this workspace were `PathBuf`, reported
/// once per file importing it. `std::path::PathBuf` is not this repository's to rename.
#[test]
fn Test_Violations_In_Should_Not_Judge_A_Use_Binding()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tUse\tPrivate\tPathBuf\t.\t.\n");

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert!(findings.is_empty(), "an import names something declared elsewhere: {findings:?}");
}

/// `P68-ABBREVIATIONS-DOES-NOT-EXEMPT-EXTERN-CRATE`: the third name-nobody-chose
/// exemption, found by `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON`'s own
/// third-party fixture. `extern crate alloc;` names the crate being linked, not
/// something authored at this site, the identical reason a `Use` binding is exempt.
#[test]
fn Test_Violations_In_Should_Not_Judge_An_Extern_Crate_Declaration()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tExternCrate\tPrivate\talloc\t.\t.\n");

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert!(findings.is_empty(), "the real crate being linked chose this name, not this site: {findings:?}");
}

/// And the half that keeps the exemption honest: the declaration itself is still judged,
/// so a name this repository really did choose is reported where it was chosen rather
/// than nowhere.
#[test]
fn Test_Violations_In_Should_Still_Judge_A_Real_Declaration_Of_The_Same_Name()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\nitem\t0\tUse\tPrivate\tPathBuf\t.\t.\n\
         item\t1\tStruct\tPublic\tPathBuf\t.\t+fields\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "the declaration answers for the name, the import does not: {findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "PathBuf");
}

/// The `impl` block itself still answers for its own name -- the type it names was the
/// author's choice even when the trait's member names were not.
#[test]
fn Test_Violations_In_Should_Still_Judge_The_Trait_Impl_Blocks_Own_Name()
{
    let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tCtx\t.\t+trait\n");

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Ctx");
}

#[test]
fn Test_Violations_In_Should_Accept_Approved_And_Ordinary_Names()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\n\
         item\t0\tFunction\tPublic\tParseJson\t.\t+fn/0\n\
         item\t1\tStruct\tPublic\tConfig\t.\t+fields\\ntotal_count\\tusize\n",
    );

    let findings = Violations_In(&payload, "src/lib.rs", &[]);

    assert!(findings.is_empty(), "{findings:?}");
}

fn Payload_From_Text(text: &str) -> SyntaxPayload
{
    return nomos_cap_syntax::Parse_Payload(text.as_bytes()).expect("this fixture payload is well formed");
}

/// `OD-CAPABILITY-014` put a variable-length body behind an `impl` block's own
/// trait-or-inherent label, so this carry stopped being an equality test against
/// [`nomos_cap_syntax::TRAIT`]. `impl<T: Display> Display for T` is the shape that
/// proves it: read the bare constant and its members lose an exemption the trait fixed.
#[test]
fn Test_Enclosing_Trait_Impl_Should_Carry_A_Generic_Trait_Impl()
{
    let payload = Payload_From_Text(
        "unexpanded\t0\n\
         item\t0\tImplementation\tNotApplicable\tT\t.\t+trait\\ngenerics\\nT\n\
         item\t1\tImplementation\tNotApplicable\tTable\t.\t+inherent\\ngenerics\\nT\n",
    );

    let generic_trait_impl = payload.items.first().expect("the payload fixture declares two items");
    let generic_inherent_impl = payload.items.get(1).expect("the payload fixture declares two items");

    assert_eq!(Enclosing_Trait_Impl(generic_trait_impl, None), Some("T".to_owned()));
    assert_eq!(Enclosing_Trait_Impl(generic_inherent_impl, None), None);
}
