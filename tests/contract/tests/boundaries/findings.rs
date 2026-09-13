//! A finding's subject is one of the derivations `OD-ANALYSIS-011` permits.

use nomos_contracts::{Finding, SubjectId};
use nomos_model::{Content_Digest, Subject_Of_Path};
use nomos_rules::SourceFile;

/// A finding's `subject` is its file's identity, the identity of the file a line-suffixed
/// location names, or the digest of its own `subject_name` — and never anything else.
///
/// # Why this is a test and not a sentence
///
/// `Finding::subject` used to document itself as "derived from `subject_name`, never from a
/// path", and `subject_name` as its preimage. Both were false across most of the rule set,
/// and nothing noticed for as long as they stood, because nothing compared them. A whole
/// decision — `OD-GATE-024` — was written on that sentence and had to be retracted when
/// implementing it proved it false. This is what keeps the replacement from going the same
/// way.
///
/// # Why three derivations and not the two the record first named
///
/// `OD-ANALYSIS-011` named two, and a peer session measured that a third of its own census
/// satisfies neither. 22 of the 63 finding constructions under `checks/` pair
/// `subject: source.subject` — the *file*, no line — with a `subject_name` and a location of
/// the form `path:line`. `Normalize_Path` splits on `/` and lowercases; it does not strip a
/// `:line` suffix, so `Subject_Of_Path("a.rs:12")` is not `Subject_Of_Path("a.rs")` and both
/// of the first two arms fail. The derivation that actually holds there is the file part of
/// the location, which is the third arm below.
///
/// # What a failure here means
///
/// A rule wrote a subject from something that is none of the three. That is not
/// automatically wrong — it is a fourth derivation nobody has decided on, and
/// `OD-ANALYSIS-011` is where the decision would have to be amended before the rule lands.
#[test]
fn Test_A_Findings_Subject_Should_Be_One_Of_The_Permitted_Derivations()
{
    let findings = Real_Findings();

    let unexplained: Vec<&Finding> =
        findings.iter().filter(|finding| return !Is_Permitted(finding)).collect();

    assert!(
        unexplained.is_empty(),
        "these findings' subjects are none of the three derivations OD-ANALYSIS-011 permits: {:?}",
        unexplained
            .iter()
            .map(|finding| return (finding.rule.As_Str(), finding.subject_name.as_str()))
            .collect::<Vec<_>>()
    );
}

/// The assertion above is worthless over a fixture that produced none of the shape it exists
/// for, and that is not hypothetical: measured 2026-09-12, a whole-tree `nomos gate run`
/// fires 8 of roughly 70 rules and **not one** of its findings carries a line-suffixed
/// location. A guard written against the committed corpus alone would have passed while
/// saying nothing about the 22 sites that motivated it.
///
/// So this names the shapes the fixture must actually exercise, and fails if it stops
/// producing one of them — a rule renamed or a fixture edited must not quietly turn the
/// assertion above into a tautology.
#[test]
fn Test_The_Fixture_Should_Exercise_Both_Subject_Shapes_The_Census_Found()
{
    let findings = Real_Findings();

    assert!(
        findings.iter().any(|finding| return Names_A_Line(finding)),
        "no finding carries a line-suffixed location, so the third derivation -- the one a \
         third of the census needs -- went unchecked: {:?}",
        findings.iter().map(|finding| return finding.subject_name.as_str()).collect::<Vec<_>>()
    );
    assert!(
        findings.iter().any(|finding| return !Names_A_Line(finding)),
        "every finding carries a line, so the plain file derivation went unchecked"
    );
}

/// Whether `finding` names a location of the `path:line` form.
fn Names_A_Line(finding: &Finding) -> bool
{
    return finding.locations.iter().any(|location| return File_Part(location) != location.as_str());
}

/// Whether `finding`'s subject is one of the three derivations the record permits.
fn Is_Permitted(finding: &Finding) -> bool
{
    if finding.subject == SubjectId::From_Digest(Content_Digest(finding.subject_name.as_bytes()))
    {
        return true;
    }

    return finding.locations.iter().any(|location| {
        return finding.subject == Subject_Of_Path(location)
            || finding.subject == Subject_Of_Path(File_Part(location));
    });
}

/// The path a `path:line` location names, or the location itself when it names no line.
///
/// Split from the right on the last `:`, and only when what follows is all digits — a
/// Windows path carries a drive colon, and a location is repository-relative but nothing
/// here needs to assume that to be safe about it.
fn File_Part(location: &str) -> &str
{
    let Some((path, line)) = location.rsplit_once(':')
    else
    {
        return location;
    };

    if line.is_empty() || !line.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return location;
    }

    return path;
}

/// Findings from two real, fact-free rules over a fixture chosen to produce both shapes: one
/// that attributes to the file, and one that attributes to the file while naming a line.
fn Real_Findings() -> Vec<Finding>
{
    let sources = vec![
        // Declared by nothing, so `no-orphan-modules` attributes to the file itself.
        Source("crates/example/src/lib.rs", "mod thing;\n"),
        Source("crates/example/src/stray.rs", "pub fn Stray() {}\n"),
        // A trailing space, so `no-trailing-whitespace` names a line.
        Source("crates/example/src/thing.rs", "pub fn Thing() {} \n"),
    ];

    let mut findings = nomos_rules::Check_No_Orphan_Modules(&sources);
    findings.extend(nomos_rules::Check_No_Trailing_Whitespace(&sources));

    assert!(!findings.is_empty(), "this fixture must produce real findings");

    return findings;
}

/// A source the way a real walk would hand one over.
fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}
