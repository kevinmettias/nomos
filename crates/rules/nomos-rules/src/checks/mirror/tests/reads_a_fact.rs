//! The judgment reads a fact, and an empty store is not a clean tree.
//!
//! These are the tests that would catch a fact path nothing consults: the verdict has to
//! turn on what the store holds and on nothing the rule could have read out of the text
//! itself.

use super::*;

/// How many findings report a subject the run could not read.
fn Unavailable(findings: &[Finding]) -> usize
{
    return findings
        .iter()
        .filter(|finding| return finding.applicability == Applicability::DependencyUnavailable)
        .count();
}

/// The load-bearing test for `OD-RULES-001`, and the one that would catch a fact path
/// nothing consults. One tree, two runs, one difference: whether the store holds the
/// fact for the file that defines the check. Withheld, the claim does not resolve;
/// present, the rule reports nothing at all. The verdict turns on the fact and on
/// nothing else — the text handed to the rule is byte-identical in both runs.
#[test]
fn Test_A_Mirror_Should_Resolve_Only_Through_A_Fact()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Present`.\npub const T: &[&str] = &[];\n",
    );
    let checking = Source("b.rs", "#[test]\nfn Test_Present()\n{\n}\n");
    let sources = vec![declaring.clone(), checking.clone()];
    let with_fact = World_Over(&[(&declaring, &[]), (&checking, &["Test_Present"])]);
    // The same two files, and the store is told about only one of them.
    let without_fact = World_Over(&[(&declaring, &[])]);

    let resolved = Judged_In(&with_fact, &sources);
    let unresolved = Judged_In(&without_fact, &sources);

    assert!(
        resolved.is_empty(),
        "the fact for the defining file is present, so the claim resolves: {resolved:?}"
    );
    Assert_Withholding_The_Fact_Leaves_Both_Unresolved(&unresolved);
}

/// With the defining file's fact withheld, the subject is named and the claim does not resolve
/// out of a text the rule can still see.
fn Assert_Withholding_The_Fact_Leaves_Both_Unresolved(unresolved: &[Finding])
{
    assert!(
        Named("b.rs", unresolved).is_some(),
        "the subject whose fact was withheld must be named: {unresolved:?}"
    );
    assert!(
        Named("T", unresolved).is_some(),
        "the claim must not resolve out of a text the rule can still see: {unresolved:?}"
    );
}

/// An empty store is not a clean tree.
///
/// Every subject unread, so the rule must say it could not run rather than say
/// nothing. Three properties together are what "could not run" means: the result is
/// not empty, every finding names an unavailability rather than a phantom, and nothing
/// in it can stop a build.
///
/// The count is two rather than three since `P10-SYNTAX-V2`, and the difference is
/// worth stating because it is a guarantee that changed rather than a fixture that
/// moved. A universe declared in a file whose fact was never read is no longer
/// discovered at all: discovery reads the fact, and there is no fact. The rule used to
/// find it in the text and report the claim as withheld, which was the same bypass
/// `OD-RULES-001` closed for check names — a rule reaching around its own fact layer.
/// What is kept is the property this test is named for: a run that read nothing renders
/// as one finding per unread file and never as a clean tree. What is given up is
/// per-claim detail inside a file nobody read, which is a sentence about a subject the
/// rule never saw. `OD-SYNTAX-002` records the trade.
#[test]
fn Test_An_Empty_Store_Should_Not_Report_A_Clean_Tree()
{
    let sources = vec![
        Source("a.rs", "/// Mirrored by `Test_Somewhere`.\npub const T: &[&str] = &[];\n"),
        Source("b.rs", "#[test]\nfn Test_Somewhere()\n{\n}\n"),
    ];
    let world = World::Offering(&[(PARSER, Parser_Guarantee())]);
    let findings = Judged_In(&world, &sources);

    assert!(
        !findings.is_empty(),
        "a run that read no fact at all must not render as a clean tree"
    );
    assert_eq!(Unavailable(&findings), 2, "one finding per unread subject: {findings:?}");
    assert!(
        findings
            .iter()
            .all(|finding| return !finding.Can_Fail_A_Build()),
        "a rule that read nothing must not stop anybody: {findings:?}"
    );
    assert!(
        !Blocks_Anything(&findings),
        "nothing may be reported as a phantom out of an index that was never built"
    );
}

/// A short index must not manufacture the one finding that blocks.
///
/// One file's fact is missing and the universe claims a mirror defined in that file.
/// The claim fails to resolve, and the rule may not call that a false claim of
/// coverage: it cannot tell one from a name it did not get to look for.
///
/// This is also the control that stops `OD-RULES-002` over-correcting into "every
/// unresolved claim blocks". The scoping added there is a narrowing of *which* claims
/// inherit the doubt and not a removal of the doubt, and this is the case it must
/// still cover: the missing subject is exactly the one that would have resolved the
/// name.
#[test]
fn Test_An_Incomplete_Index_Should_Not_Manufacture_A_Phantom()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_In_The_Unread_File`.\npub const T: &[&str] = &[];\n",
    );
    let unread = Source("b.rs", "#[test]\nfn Test_In_The_Unread_File()\n{\n}\n");
    let judged = Judged_Beside(&declaring, &unread);
    let claim = Named("T", &judged).expect("the universe is still judged");

    assert_eq!(claim.gate, GateCategory::Advisory);
    assert_eq!(claim.applicability, Applicability::DependencyUnavailable);
    assert!(
        !claim.Can_Fail_A_Build(),
        "a claim the rule could not check must not stop a build: {claim:?}"
    );
    assert!(
        claim.summary.contains("the check index is short b.rs"),
        "the finding must name the subject that could have resolved it: {}",
        claim.summary
    );
}
