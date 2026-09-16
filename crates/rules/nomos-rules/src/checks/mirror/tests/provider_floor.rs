//! What the floor buys, and what still goes wrong above it.
//!
//! Three ways a composition can fail to answer this rule, each reported differently: nothing
//! offers the capability, something offers below the floor, and something admitted answers in
//! a shape that observed nothing. None of them may render as a universe that declares no
//! mirror, which is a gap somebody has already accepted.

use super::*;
use core::fmt::Write as _;

/// A stand-in for a line scanner, at exactly the guarantee `nomos-lang-rust-scan`
/// declares. Used only to show that this rule's floor refuses it.
const SCANNER: &str = "nomos.test.scans";

fn Scanner_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// A payload from a provider that could not observe what a universe is read from.
///
/// The scanner's shape, filed under an admitted guarantee. The rule normally never sees
/// one — its floor excludes the provider that writes it — so this is the case that
/// arrives when some future provider is admitted and still cannot read doc comments.
fn Blind_Payload(names: &[&str]) -> Vec<u8>
{
    let mut text = String::from("unexpanded\t0\n");

    for (ordinal, name) in names.iter().enumerate()
    {
        let _ = writeln!(text, "item\t{ordinal}\tConstant\tPublic\t{name}\t-\t-");
    }

    return text.into_bytes();
}

/// Nothing offers what the rule needs, which is a different sentence from "the store
/// had nothing". The remedy is registering a provider rather than running one, and
/// `Applicability` is where the difference is carried.
#[test]
fn Test_A_Composition_With_No_Provider_Should_Report_A_Missing_Capability()
{
    let source = Source("a.rs", "pub const T: &[&str] = &[];\n".to_owned());

    let world = World::Offering(&[]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&[source], &mut reader);

    assert!(
        findings
            .iter()
            .any(|finding| return finding.applicability == Applicability::MissingCapability),
        "{findings:?}"
    );
}

/// The floor buys something, and this is what.
///
/// A composition that registered only a line scanner cannot serve this rule: the offer
/// is below the floor, the registry refuses it, and the rule reports that it could not
/// run rather than resolving claims against names a scanner found inside comments.
/// Asserted against a guarantee written out here rather than against
/// `nomos-lang-rust-scan`, which this crate must not name.
#[test]
fn Test_The_Scanners_Guarantee_Should_Not_Satisfy_This_Rules_Floor()
{
    Assert_The_Floor_Admits_Only_A_Parser();

    let source = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n".to_owned(),
    );
    let world = World::Offering(&[(SCANNER, Scanner_Guarantee())]);
    let findings = Judged_In(&world, &[source]);

    // The floor keeps the scanner's answer out, so no fact reaches the rule and the
    // universe is not discovered. What the run reports is the subject, as a missing
    // capability — and crucially not as a universe that declares no mirror, which is
    // the downgrade `nomos.syntax.items.v2` and this floor both exist to prevent.
    Assert_The_Run_Reports_A_Missing_Capability(&findings);
}

/// A scanner's approximation is below the floor and a parser's guarantee meets it.
fn Assert_The_Floor_Admits_Only_A_Parser()
{
    let floor = Syntax_Requirement().minimum;

    assert!(
        !Scanner_Guarantee().Satisfies(&floor),
        "an unsound approximation must not be admissible as a source of check names"
    );
    assert!(
        Parser_Guarantee().Satisfies(&floor),
        "a floor no provider can meet is a declared need with nothing behind it"
    );
}

/// With only a scanner admitted the rule could not run, so it reports the subject rather than
/// the universe, and may not block on anything.
fn Assert_The_Run_Reports_A_Missing_Capability(findings: &[Finding])
{
    let unread = Named("a.rs", findings)
        .expect("the subject no admitted provider answered for must be reported");

    assert_eq!(unread.applicability, Applicability::MissingCapability);
    assert!(
        !unread.Can_Fail_A_Build(),
        "with only a scanner admitted the rule could not run, so it may not block"
    );
    assert!(
        !Has_A_Blocking_Finding(findings),
        "an approximate provider must not be able to produce a blocking finding here"
    );
}

/// A universe read through a provider that cannot see doc comments is not an admitted
/// gap.
///
/// The property `P10-SYNTAX-V2` exists for, at the end of the chain. The floor above
/// keeps today's scanner out; this is what happens when some future provider is
/// *admitted* and still cannot read documentation. Its payload spells both fields "not
/// observed", so the rule reports a file it could not judge — never a list that
/// declares no mirror, which would turn a phantom into a gap somebody has already
/// accepted.
#[test]
fn Test_A_Universe_Read_Through_A_Blind_Provider_Should_Not_Be_An_Admitted_Gap()
{
    let source = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n".to_owned(),
    );

    let world = World::Offering(&[(PARSER, Parser_Guarantee())]).Materializing(
        &source,
        nomos_cap_syntax::SCHEMA,
        Blind_Payload(&["T"]),
    );
    let findings = Judged_In(&world, &[source]);

    Assert_Nothing_Was_Observed(&findings);
}

/// A payload that observed nothing about a file is reported as unparseable, naming the field
/// it could not see — and the universe inside it is not judged at all.
fn Assert_Nothing_Was_Observed(findings: &[Finding])
{
    let unobserved = Named("a.rs", findings)
        .expect("a file whose provider saw no documentation must be reported");

    assert_eq!(unobserved.applicability, Applicability::Unparseable);
    assert!(
        unobserved.summary.contains("documentation"),
        "the finding must name the field that was not observed: {}",
        unobserved.summary
    );
    assert!(
        Named("T", findings).is_none(),
        "the universe must not be judged out of a payload that observed nothing about \
         it: {findings:?}"
    );
}

/// A fact stamped with a schema this build does not read is not an empty file.
///
/// The payload's shape is versioned separately from the contract, so a future provider
/// can answer the same question in a shape this reader has never seen. Decoding it
/// anyway would build a check index out of a guess.
#[test]
fn Test_A_Payload_Under_Another_Schema_Should_Not_Be_Decoded()
{
    let source = Source("a.rs", "pub const T: &[&str] = &[];\n".to_owned());

    let world = World::Offering(&[(PARSER, Parser_Guarantee())]).Materializing(
        &source,
        "nomos.syntax.items.v9",
        Payload(&source, &["Test_From_The_Future"]).expect("the fixture parses"),
    );
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&[source], &mut reader);

    let unread = findings
        .iter()
        .find(|finding| return finding.subject_name == "a.rs")
        .expect("the subject must be reported unread");

    assert_eq!(unread.applicability, Applicability::Unparseable);
    assert!(unread.summary.contains("v9"), "{}", unread.summary);
}
