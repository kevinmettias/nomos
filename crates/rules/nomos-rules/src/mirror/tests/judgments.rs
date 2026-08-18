//! The verdicts this rule can reach over a tree it read whole, and the controls that stop
//! each of them from being the only one it can reach.
//!
//! A check that fires on everything is not a judgment and a check that never fires is not a
//! gate, so both directions are asserted here.

use super::*;

/// The control that stops the rule being a counter. A check that fires on everything
/// is not a judgment, and a gate that can never be green is one everybody learns to
/// ignore.
#[test]
fn Test_A_Universe_Whose_Mirror_Exists_Should_Produce_No_Finding()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
         pub const TABLES: &[&str] = &[];\n",
    );
    let checking = Source(
        "a_test.rs",
        "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
    );
    let sources = vec![declaring.clone(), checking.clone()];

    let world = World_Over(&[
        (&declaring, &[]),
        (&checking, &["Test_Every_Table_Should_Be_Declared"]),
    ]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The negative control this rule exists for.
///
/// A universe naming a check that does not exist reads as covered and checks nothing.
/// It is the only outcome that blocks, and it must block: an admitted gap is honest,
/// a false claim of coverage is not.
#[test]
fn Test_A_Mirror_That_Resolves_To_Nothing_Should_Block()
{
    let findings = Findings_Over(&[Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n",
    )]);

    let finding = Only(&findings);

    assert_eq!(finding.gate, GateCategory::Blocking);
    assert!(
        finding.Can_Fail_A_Build(),
        "a claim of coverage that checks nothing must be able to stop a build"
    );
    assert!(
        finding.summary.contains("Test_Renamed_Away"),
        "the finding must name the check that resolved to nothing: {}",
        finding.summary
    );
}

/// The severity ordering, asserted directly, because it is the judgment call this
/// rule makes and a later edit could quietly invert it.
#[test]
fn Test_A_False_Claim_Should_Outrank_An_Admitted_Gap()
{
    let admitted = Findings_Over(&[Source("a.rs", "pub const TABLES: &[&str] = &[];\n")]);
    let false_claim = Findings_Over(&[Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLES: &[&str] = &[];\n",
    )]);

    assert_eq!(Only(&admitted).gate, GateCategory::Advisory);
    assert_eq!(Only(&false_claim).gate, GateCategory::Blocking);
    assert!(
        Only(&false_claim).gate > Only(&admitted).gate,
        "a phantom mirror must outrank an admitted gap"
    );
    assert!(
        !Only(&admitted).Can_Fail_A_Build(),
        "an admitted gap is honest, and twelve of them exist; blocking on those \
         makes a gate that can never be green"
    );
}

/// A finding must carry its own provenance rather than leaving the reader to assume
/// it. `Derived` is the honest class: computed from source by a deterministic rule.
#[test]
fn Test_A_Finding_Should_Report_How_It_Was_Come_By()
{
    let findings = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
    let finding = Only(&findings);

    assert_eq!(finding.evidence, EvidenceClass::Derived);
    assert!(finding.Is_Mechanical());
    assert_eq!(finding.applicability, Applicability::Supported);
}

/// Identity is the name, not the path. A universe that moves file is the same
/// universe, and a finding keyed on its location would close and reopen for free.
#[test]
fn Test_The_Same_Universe_In_Two_Places_Should_Keep_One_Identity()
{
    let here = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
    let moved = Findings_Over(&[Source("b/c.rs", "pub const T: &[&str] = &[];\n")]);

    assert_eq!(Only(&here).subject, Only(&moved).subject);
    assert_ne!(Only(&here).locations, Only(&moved).locations);
}

/// An empty tree yields nothing, and that is exactly why the caller has to check for
/// vacuity. Asserted here so the property is written down where the rule is, rather
/// than being an unstated assumption the composition root happens to cover.
#[test]
fn Test_No_Sources_Should_Produce_No_Findings()
{
    let world = World_Over(&[]);
    let mut reader = world.Reader();

    assert!(Check_Completeness_Mirrors(&[], &mut reader).is_empty());
}

/// The rule's own back door, under the new source of names. A check name written
/// inside a fixture string is not a check, and resolving it would let a claim pass
/// while nothing checks it.
///
/// What keeps that true is no longer a parse performed here. It is the payload: a
/// parser reports the items a file declares and a string literal is not one, so the
/// name is not in the index. The fixture file's fact declares the function holding the
/// fixture and nothing else, which is what `nomos-lang-rust` really writes. That the
/// *provider* behaves this way and a line scanner does not is what
/// [`crate::Syntax_Requirement`]'s floor is for, and it is asserted in
/// [`super::provider_floor`].
#[test]
fn Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Only_In_A_Fixture`.\npub const T: &[&str] = &[];",
    );
    let fixture = Source(
        "b.rs",
        "fn Fixture() { let source = \"fn Test_Only_In_A_Fixture() {}\"; }",
    );
    let sources = vec![declaring.clone(), fixture.clone()];

    let world = World_Over(&[(&declaring, &[]), (&fixture, &["Fixture"])]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert_eq!(Only(&findings).gate, GateCategory::Blocking);
}

/// A file that could not be read is reported, not skipped. A run that silently drops
/// what it could not parse and reports clean is the shape this workspace keeps
/// finding — and `Applicability` is the field that says so.
///
/// Which side reports it moved with `P10-SYNTAX-V2`. The rule used to parse the text
/// itself and refuse it here; now the provider refuses it, files no fact, and the
/// subject arrives unread. The property is the same and the path is shorter — there is
/// one parser in the workspace and it is the one that says a file does not parse.
#[test]
fn Test_An_Unparseable_File_Should_Be_Reported_And_Not_Fail_The_Build()
{
    let findings = Findings_Over(&[Source("broken.rs", "pub const ??? = ;")]);

    let unparseable = findings
        .iter()
        .find(|finding| return finding.summary.contains("no syntax fact could be read"))
        .expect("the file nobody could produce a fact for must be reported");

    assert_eq!(unparseable.applicability, Applicability::DependencyUnavailable);
    assert!(
        !unparseable.Can_Fail_A_Build(),
        "a rule that could not read its subject must not stop anybody"
    );
}

/// A mention is not a definition. Resolving against prose would let a comment naming
/// a deleted test keep the claim alive, which is the defect one level up.
#[test]
fn Test_A_Test_Named_Only_In_Prose_Should_Not_Resolve()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Deleted`.\npub const T: &[&str] = &[];\n",
    );
    let prose = Source("b.rs", "// see Test_Deleted for the comparison\n");
    let sources = vec![declaring.clone(), prose.clone()];

    let world = World_Over(&[(&declaring, &[]), (&prose, &[])]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert_eq!(Only(&findings).gate, GateCategory::Blocking);
}

/// `D-134`'s qualified-name fix, proven end to end. `verdict.rs` used to hash
/// `universe.name` alone into the finding's subject, so two universes sharing a bare
/// name in different crates — same constant name, different `crates/.../src/...` roots
/// — would collide onto one `SubjectId` and become indistinguishable in every finding,
/// suppression, or history keyed on it. Real crate paths, not the synthetic single-
/// segment paths the rest of this suite uses, because the qualifier is derived from the
/// `/src/` split and a path without one stays deliberately unqualified.
#[test]
fn Test_Two_Crates_Sharing_A_Bare_Universe_Name_Should_Not_Share_A_Subject()
{
    let store = Source(
        "crates/spec/nomos-spec-store/src/table.rs",
        "pub const TABLES: &[&str] = &[];\n",
    );
    let project = Source(
        "crates/spec/nomos-spec-project/src/table.rs",
        "pub const TABLES: &[&str] = &[];\n",
    );

    let findings = Findings_Over(&[store, project]);

    let [store_finding, project_finding] = findings.as_slice()
    else
    {
        panic!("expected exactly two findings: {findings:?}");
    };

    assert_eq!(store_finding.subject_name, project_finding.subject_name, "{findings:?}");
    assert_ne!(
        store_finding.subject, project_finding.subject,
        "two universes named `TABLES` in different crates must not collide onto one \
         subject: {findings:?}"
    );
}
