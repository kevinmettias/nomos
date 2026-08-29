//! `OD-RULES-002`: the shortfall is scoped to the claim it bears on.
//!
//! The defect the record was opened against. One unreadable subject anywhere silenced
//! every phantom in the tree, and over this workspace there is always one:
//! `tests/corpus/analysis/gamma/broken.rs` is a fixture the parser is supposed to
//! refuse. So the guard reported and never judged.
//!
//! Every test here is a pair of a claim and a shortfall that either does or does not bear
//! on it, and the last two are the controls that stop the correction going too far in
//! either direction.

use super::*;

/// How many findings the run produced about one subject.
fn Reported_Times(subject: &str, findings: &[Finding]) -> usize
{
    return findings
        .iter()
        .filter(|finding| return finding.subject_name == subject)
        .count();
}

/// Every claim the run judged, as a name and a gate, in the order it reports them.
fn Claims_Judged(findings: &[Finding]) -> Vec<(&str, GateCategory)>
{
    return findings
        .iter()
        .filter(|finding| return finding.subject_name.starts_with(char::is_uppercase))
        .map(|finding| return (finding.subject_name.as_str(), finding.gate))
        .collect();
}

/// Here the phantom is claimed in a file whose fact was read, and the unreadable file
/// is present in the same run and does not spell the claimed name. Those two facts have
/// nothing to do with each other, and the judgment must be made.
#[test]
fn Test_A_Phantom_In_A_Read_Subject_Should_Block_Though_Another_Subject_Was_Unread()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );
    // The same shape as the real fixture: text no parser accepts, and no fact.
    let broken = Source("broken.rs", "pub const ??? = ;\n");
    let judged = Judged_Beside(&declaring, &broken);
    let claim = Named("T", &judged).expect("the universe is judged");

    assert_eq!(claim.gate, GateCategory::Blocking);
    assert_eq!(claim.applicability, Applicability::Supported);
    assert!(
        claim.Can_Fail_A_Build(),
        "a phantom claimed in a file whose facts were read must stop a build whatever \
         some other subject did: {claim:?}"
    );
    Assert_The_Shortfall_Travels_With_The_Judgment(claim, &judged);
}

/// The caveat travels with the judgment rather than replacing it. A reader who wants to know
/// what the run did not see is owed that on the finding, not instead of it.
///
/// And the unreadable subject is still reported — once now rather than twice. It used to be
/// counted by the text side as well, which was this rule parsing the file itself; there is one
/// report because there is one reading.
fn Assert_The_Shortfall_Travels_With_The_Judgment(claim: &Finding, judged: &[Finding])
{
    assert!(
        claim.summary.contains("short 1 subject(s)")
            && claim.summary.contains("Test_Renamed_Away"),
        "the blocking finding must carry the shortfall it ruled out: {}",
        claim.summary
    );
    assert_eq!(
        Reported_Times("broken.rs", judged),
        1,
        "the unreadable file must still be reported: {judged:?}"
    );
}

/// The second control, and the one that stops the correction going too far.
///
/// A claim sited in a subject whose own fact could not be read, naming a check that
/// subject itself declares. Nothing resolves it and it is still not a phantom: the file
/// that would have resolved it is the file the index is short of. A universe and the
/// test that mirrors it living in one file is the ordinary shape in this workspace, so
/// this is not a corner.
#[test]
fn Test_A_Claim_In_An_Unread_Subject_Should_Not_Be_A_Phantom()
{
    let declaring = Declaring_Its_Own_Mirror();
    let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
    let sources = vec![declaring.clone(), other.clone()];
    // Every fact but the declaring file's own, so the index is real and short of it.
    let world = World_Over(&[(&other, &["Test_Something_Else"])]);
    let findings = Judged_In(&world, &sources);

    // Since `P10-SYNTAX-V2` the claim is not judged at all, because the universe is not
    // discovered: discovery reads the fact and this file has none. The property the
    // test is named for is unchanged and stronger — nothing here is a phantom — and the
    // subject is still reported, which is what keeps the run from rendering clean.
    assert!(
        Named("T", &findings).is_none(),
        "a claim in a file the rule never read must not be judged at all: {findings:?}"
    );
    Assert_The_Unread_Subject_Is_Advisory(&findings);
}

/// A file that declares a universe and holds the test mirroring it, which is the ordinary
/// shape in this workspace.
fn Declaring_Its_Own_Mirror() -> SourceFile
{
    return Source(
        "a.rs",
        "/// Mirrored by `Test_Right_Here`.\n\
         pub const T: &[&str] = &[];\n\
         #[cfg(test)]\n\
         mod tests\n\
         {\n\
         \x20   #[test]\n\
         \x20   fn Test_Right_Here()\n\
         \x20   {\n\
         \x20   }\n\
         }\n",
    );
}

/// The subject whose fact was missing is reported, advisory, and stops nobody — and nothing in
/// the run is a phantom.
fn Assert_The_Unread_Subject_Is_Advisory(findings: &[Finding])
{
    let unread = Named("a.rs", findings).expect("the subject whose fact was missing is reported");

    assert_eq!(unread.gate, GateCategory::Advisory);
    assert_eq!(unread.applicability, Applicability::DependencyUnavailable);
    assert!(
        !unread.Can_Fail_A_Build(),
        "a rule that did not read a subject must not stop anybody over it: {unread:?}"
    );
    assert!(
        !Blocks_Anything(findings),
        "nothing may be reported as a phantom out of a file nobody read: {findings:?}"
    );
}

/// The unsound half of the scoping, in the direction it is allowed to be unsound.
///
/// `Can_Have_Declared` is a substring test over a text no parser read, so a name
/// spelled only in a comment counts as "could have declared it". That is a false
/// positive and it *withholds* a block, which is the safe direction and the reason an
/// unsound signal is admissible here at all: `Check_Index_Of` may not use text as a
/// source of names because an unsound positive there resolves a claim and silences the
/// rule, and this one can only ever add doubt.
#[test]
fn Test_A_Name_Spelled_In_A_Comment_Should_Still_Withhold_The_Block()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Only_Mentioned`.\npub const T: &[&str] = &[];\n",
    );
    let broken = Source(
        "broken.rs",
        "// Test_Only_Mentioned used to live here.\npub const ??? = ;\n",
    );
    let judged = Judged_Beside(&declaring, &broken);
    let claim = Named("T", &judged).expect("the universe is judged");

    assert_eq!(
        claim.gate,
        GateCategory::Advisory,
        "an unread file that spells the name is doubt, however it spells it: {claim:?}"
    );
    assert!(!claim.Can_Fail_A_Build());
}

/// Two claims, one run, one unreadable subject — and they are judged differently.
///
/// The sharpest form of what `OD-RULES-002` changed. Under one flag over the whole run
/// these two claims were indistinguishable and both were downgraded. The shortfall is a
/// property of the subjects each judgment rests on, so the run now says two different
/// things in the same breath.
#[test]
fn Test_Two_Claims_Under_One_Shortfall_Should_Be_Judged_Separately()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_In_The_Broken_File`.\n\
         pub const WITHHELD: &[&str] = &[];\n\
         /// Mirrored by `Test_Nowhere_At_All`.\n\
         pub const PHANTOM: &[&str] = &[];\n",
    );
    let broken = Source("broken.rs", "fn Test_In_The_Broken_File( ??? = ;\n");
    let sources = vec![declaring.clone(), broken.clone()];
    let world = World_Over(&[(&declaring, &[])]);
    let findings = Judged_In(&world, &sources);
    let judged = Claims_Judged(&findings);

    assert_eq!(
        judged,
        vec![
            ("PHANTOM", GateCategory::Blocking),
            ("WITHHELD", GateCategory::Advisory),
        ],
        "one shortfall bears on one of these two claims and not the other: {findings:?}"
    );
}

/// And the shortfall is still whole-run when there is no index at all.
///
/// `MissingCapability` is not scoped by what a subject spells, and the reason is not a
/// carve-out: with no admitted provider there is no index, so no name can be shown
/// absent from it. Without this arm a run against a composition that offers the rule
/// nothing would block on every claim in the tree — a verdict reached by a rule that
/// never got an answer from anybody, which is what `OD-RULES-001` forbids.
#[test]
fn Test_With_No_Provider_Admitted_No_Claim_Should_Be_A_Phantom()
{
    let source = Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere_In_This_Tree`.\npub const T: &[&str] = &[];\n",
    );
    let world = World::Offering(&[]);
    let findings = Judged_In(&world, &[source]);

    // With nothing offering, no fact exists and no universe is discovered — so the
    // claim is not judged rather than judged leniently. `MissingCapability` is still
    // what the run reports, on the subject instead of on the claim, and the finding
    // that must never appear is a phantom.
    let unread = Named("a.rs", &findings)
        .expect("a run with no provider must report the subject it could not read");

    assert_eq!(unread.applicability, Applicability::MissingCapability);
    assert_eq!(unread.gate, GateCategory::Advisory);
    assert!(!unread.Can_Fail_A_Build());
    assert!(
        !Blocks_Anything(&findings),
        "with no provider admitted there is no index for a name to be absent from, so \
         nothing may be called a phantom: {findings:?}"
    );
}

/// The control that stops the tests above from being satisfied by a rule that never
/// blocks. Every subject read, one claim that resolves to nothing, and it blocks.
#[test]
fn Test_A_Phantom_Should_Still_Block_When_Every_Subject_Was_Read()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );
    let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
    let sources = vec![declaring.clone(), other.clone()];

    let world = World_Over(&[(&declaring, &[]), (&other, &["Test_Something_Else"])]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    let claim = Only(&findings);

    assert_eq!(claim.gate, GateCategory::Blocking);
    assert_eq!(claim.applicability, Applicability::Supported);
    assert!(claim.Can_Fail_A_Build());
}

/// An admitted gap stays advisory and stays *evaluated* even when the index is short.
///
/// `D-134`'s decision 4 is that a false claim outranks an admitted gap, and the
/// amendment must not weaken the second half of it by letting an incomplete index
/// downgrade a universe that never claimed anything. Nothing was resolved for it, so
/// nothing could have been missed.
#[test]
fn Test_An_Admitted_Gap_Should_Not_Inherit_The_Indexs_Doubt()
{
    let declaring = Source("a.rs", "pub const T: &[&str] = &[];\n");
    let unread = Source("b.rs", "#[test]\nfn Test_Whatever()\n{\n}\n");
    let judged = Judged_Beside(&declaring, &unread);
    let gap = Named("T", &judged).expect("the universe is judged");

    assert_eq!(gap.gate, GateCategory::Advisory);
    assert_eq!(gap.applicability, Applicability::Supported);
    assert!(gap.summary.contains("declares no mirror"), "{}", gap.summary);
}
