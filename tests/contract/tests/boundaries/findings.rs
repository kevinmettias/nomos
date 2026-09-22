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
    assert!(
        findings.iter().any(Names_Its_Address),
        "no finding's subject is the digest of a declared address, so the fourth derivation \
         went unchecked -- which is the state this fixture was in before P110-B, and the \
         reason a guard over two fact-free rules passed over the family it exists for: {:?}",
        findings.iter().map(|finding| return (finding.rule.As_Str(), finding.address.as_deref())).collect::<Vec<_>>()
    );
    assert!(
        findings.iter().any(|finding| return finding.address.is_none()),
        "every finding declares an address, so the three derivations that do not use one went \
         unchecked"
    );
}

/// Whether `finding` names a location of the `path:line` form.
fn Names_A_Line(finding: &Finding) -> bool
{
    return finding.locations.iter().any(|location| return File_Part(location) != location.as_str());
}

/// Whether `finding`'s subject is one of the four derivations the record permits.
fn Is_Permitted(finding: &Finding) -> bool
{
    if Names_Its_Address(finding)
    {
        return true;
    }

    if finding.subject == SubjectId::From_Digest(Content_Digest(finding.subject_name.as_bytes()))
    {
        return true;
    }

    return finding.locations.iter().any(|location| {
        return finding.subject == Subject_Of_Path(location)
            || finding.subject == Subject_Of_Path(File_Part(location));
    });
}

/// Whether `finding`'s subject is the digest of the address its rule declared.
///
/// The fourth derivation, added by `P110-B` and carried by `OD-ANALYSIS-011`. A rule that
/// subjects its findings to something *inside* a file assembles a qualified name and digests
/// it; before this the composite was private, so the finding could be produced and not named,
/// and it satisfied none of the three arms below.
///
/// A finding declaring no address is not permitted by this arm rather than exempted from it.
/// Returning `true` for `None` would turn the whole guard into a tautology the moment a rule
/// stopped declaring one, which is the failure mode the fixture test beside this exists for.
fn Names_Its_Address(finding: &Finding) -> bool
{
    let Some(address) = finding.address.as_ref()
    else
    {
        return false;
    };

    return finding.subject == SubjectId::From_Digest(Content_Digest(address.as_bytes()));
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

/// Findings from two real, fact-free rules over a fixture chosen to produce both shapes --
/// one that attributes to the file, one that attributes to the file while naming a line --
/// and from one real fact-reading rule, for the derivation neither of those can reach.
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
    findings.extend(Addressed_Findings());

    assert!(!findings.is_empty(), "this fixture must produce real findings");

    return findings;
}

/// Findings from a rule that declares an address, which the two above cannot be.
///
/// Every rule that declares one reads syntax facts, so this cannot be a direct call into
/// `nomos_rules` the way they are: it goes through the real composition, which materializes
/// the fact and runs the rule the way a check does. A stub `FactReader` was refused here for
/// the reason this whole file exists -- a boundary asserted against a second composition says
/// nothing about the one that ships.
///
/// `completeness-mirror` alone, rather than the empty selection that means every rule, so the
/// call stays at one rule's worth of materialization.
///
/// The universe is declared under a `src/` directory on purpose. A mirror finding's address
/// is `{crate}::{universe}`, and `Qualifier_Of` leaves it unqualified for a path that names
/// no crate -- which would make it equal to `subject_name` and permitted by the first arm, so
/// the fixture would pass while exercising nothing.
fn Addressed_Findings() -> Vec<Finding>
{
    use nomos_check_orchestration::{CheckOutcome, Run, RunContext};

    let sources = vec![Source(
        "crates/example/src/lib.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let selected = [nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];

    let mut workspace = None;
    let mut store = nomos_analysis::MemoryFactStore::New();
    let judged = Run(
        &sources,
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &nomos_platform_std::StdProgramLauncher,
            filesystem: &nomos_platform_std::StdFileSystem,
            environment: &nomos_platform_std::StdEnvironment,
            workspace: &mut workspace,
            store: &mut store,
        },
        &selected,
    );

    let CheckOutcome::Judged { findings, .. } = judged
    else
    {
        panic!("the composition must judge a readable fixture, or this guard exercises nothing");
    };

    return findings;
}

/// The build variant the fixture run is judged under. Its values are never read by the rule
/// under test; a run needs one.
fn Test_Variant() -> nomos_workspace::BuildVariant
{
    return nomos_workspace::BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own root, which the run's dependency step reads regardless of the
/// sources handed in -- a workspace-wide fact is not a fact about any one of them.
fn Repository_Root() -> std::path::PathBuf
{
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::PathBuf::from)
        .expect("this suite sits two levels below the workspace root");
}

/// A source the way a real walk would hand one over.
fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}
