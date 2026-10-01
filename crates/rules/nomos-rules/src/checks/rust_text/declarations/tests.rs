//! What a declaration must prove before it may replace the function it was written as.
//!
//! `OD-RULES-034` names one falsifier and this module is it: a rule re-declared through the
//! form reports **byte-identically** to the function it replaced, measured against this
//! repository's own tree rather than against a fixture chosen to agree. The comparison is of
//! the rendered findings' bytes and nothing weaker, because a count comparison passes on two
//! rules that merely fire the same number of times, and "a form that changed what a rule
//! reports would be a new rule wearing an old identifier" is the thing being ruled out.
//!
//! # Why the tree is walked, and why the walk then says what it found
//!
//! A test that cannot find its corpus passes, and a comparison of two empty vectors is a
//! green that measured nothing. So this reads every `.rs` file under the repository root,
//! asserts a floor on how many it read, and measures separately that the tree really
//! contains construct sites the detector matches -- after the same masking the engine does,
//! so the number is the one the judgment saw rather than a grep's.
//!
//! What this tree does *not* contain is a *finding*: all three of these rules are composed
//! and this workspace's gate is at zero for them, so the real-tree comparison agrees on
//! emptiness by construction. That is why [`Test_A_Declared_Rule_Should_Report_Byte_Identically_On_Material_That_Reports`]
//! exists beside it, carrying material that makes each rule fire and comparing those bytes
//! too. Neither test is the falsifier alone; the pair is.

use super::*;

use crate::checks::code_prefix::{Code_Prefix, Code_With_String_Bodies_Masked};
use crate::checks::test_support::Test_Context;
use crate::checks::{Check_A_Disabled_Test_States_Why, Check_Every_Allow_Carries_A_Justification, Check_Inline_Always_Justification};
use crate::rule_descriptor::{RequiredFact, SubjectKind};
use crate::SourceFile;
use nomos_analysis::{FactReader, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{Finding, SubjectId};
use nomos_model::Content_Digest;
use std::path::Path;

/// The fewest Rust sources this repository has, as a floor rather than a count.
///
/// A floor and not the real number on purpose: the real number moves with every file
/// anybody adds, and a test that pinned it would fail for a reason that has nothing to do
/// with what it measures. What it rules out is the failure it was written for -- a walk
/// that found nothing, or found one directory, and then compared two empty vectors.
const FEWEST_RUST_SOURCES_THIS_TREE_HAS: usize = 1000;

/// The directories the walk does not descend into.
///
/// `target` is build output, which is not this repository's source and would multiply the
/// walk by every vendored crate cargo happened to unpack. Everything else skipped is a
/// tool's own directory, recognized by the leading dot rather than named one at a time.
const BUILD_OUTPUT_DIRECTORY: &str = "target";

/// Every declaration this crate states, enumerated where a test can iterate them.
///
/// A test-local list and deliberately not a `const` beside the declarations themselves:
/// `DESCRIPTORS` is the one table, and a second registration path is the accretion
/// `OD-RULES-034` refuses by name. A list that exists only inside a test binary registers
/// nothing.
fn Every_Declaration() -> [DeclaredTextRule; 3]
{
    return [EVERY_ALLOW_DECLARATION, INLINE_ALWAYS_DECLARATION, A_DISABLED_TEST_DECLARATION];
}

/// Each declaration beside the function it replaced, which is the pairing every comparison
/// below is over.
///
/// The linked side of two of the three takes only `&[SourceFile]`; the widening closure is
/// the same one `DESCRIPTORS` spelled for them before they were declared, and it decides
/// nothing.
fn Every_Declaration_Beside_Its_Function() -> [(DeclaredTextRule, LinkedRule); 3]
{
    return [
        (EVERY_ALLOW_DECLARATION, LinkedRule(Check_Every_Allow_Carries_A_Justification)),
        (INLINE_ALWAYS_DECLARATION, LinkedRule(|sources, _facts| return Check_Inline_Always_Justification(sources))),
        (A_DISABLED_TEST_DECLARATION, LinkedRule(|sources, _facts| return Check_A_Disabled_Test_States_Why(sources))),
    ];
}

/// The function a declaration replaced, named so a call site cannot transpose it with the
/// declaration it is being compared against.
struct LinkedRule(fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>);

/// `OD-RULES-034`'s falsifier, against the tree the record names.
///
/// Every `.rs` file in this repository is handed to the function and to the declaration, and
/// the two renderings must be the same bytes. The floor assertion is what keeps a walk that
/// found nothing from reporting as agreement.
#[test]
fn Test_A_Declared_Rule_Should_Report_Byte_Identically_Against_This_Tree()
{
    let sources = This_Repositorys_Rust_Sources();

    assert!(
        sources.len() >= FEWEST_RUST_SOURCES_THIS_TREE_HAS,
        "the walk read {} Rust sources, which is below the floor -- this comparison ran against a corpus it did not find",
        sources.len()
    );

    for (declaration, linked) in Every_Declaration_Beside_Its_Function()
    {
        Assert_Byte_Identical(&sources, &declaration, linked);
    }
}

/// What the tree comparison above was actually able to judge.
///
/// The real-tree comparison agrees on emptiness, because all three rules are composed and
/// this workspace satisfies them. That agreement is evidence only if the judgment reached
/// real construct sites and the justification look-back is what made them pass, so this
/// measures that it did -- through the same masking the engine applies, which is the
/// difference between the number the judgment saw and the number a grep would report.
///
/// It asserts for one rule rather than three, and the measurement is why. Over the 2012
/// Rust sources of this repository at the commit this landed on: thirty-three sites of
/// `#[allow(...)]`, none of `#[inline(always)]`, none of a bare `#[ignore]`. Every
/// occurrence of the latter two in this tree is inside a documentation comment or a string
/// literal -- this crate's own prose about the rules -- and the engine masks both before
/// judging. So the real-tree half of the falsifier is substantive for the rule
/// `OD-RULES-034` names and structurally empty for the two siblings converted with it,
/// which is exactly what [`Test_A_Declared_Rule_Should_Report_Byte_Identically_On_Material_That_Reports`]
/// carries material for. Asserting a zero for those two would fail the day somebody writes
/// one, which is not a defect in anything.
#[test]
fn Test_This_Tree_Should_Carry_Real_Construct_Sites_For_The_Rule_The_Falsifier_Names()
{
    let sources = This_Repositorys_Rust_Sources();

    let sites = Construct_Sites_In(&sources, EVERY_ALLOW_DECLARATION);

    assert!(
        sites > 0,
        "this tree carries no site of the construct {} judges, so the comparison against it compared two empty vectors",
        EVERY_ALLOW_DECLARATION.id
    );
}

/// The same comparison, over material that makes each rule report.
///
/// Byte-identical on an empty result is the weaker half of the falsifier; this is the half
/// that fails when a declaration reports a different line, a different message or a
/// different order than the function did. Each rule must report at least once here, which is
/// what stops this from silently becoming a second comparison of two empty vectors.
#[test]
fn Test_A_Declared_Rule_Should_Report_Byte_Identically_On_Material_That_Reports()
{
    let sources = Material_Every_Declared_Rule_Reports_On();

    for (declaration, linked) in Every_Declaration_Beside_Its_Function()
    {
        let reported = Judged_By(|facts| return declaration.Judges(&sources, facts));
        assert!(!reported.is_empty(), "{}: this material reports nothing, so the comparison below is vacuous", declaration.id);

        Assert_Byte_Identical(&sources, &declaration, linked);
    }
}

/// `OD-RULES-034`'s relation refusal, as the behaviour it forbids rather than as a note.
///
/// A form that could relate subjects would judge a file against something another file said.
/// Judging the whole corpus therefore has to report exactly what judging each file alone
/// reports, and the record's mirror-rule warning is the case this rules out: a mirror
/// declared through this form would render as a per-file loop that finds nothing, so no such
/// declaration can be written.
#[test]
fn Test_A_Declared_Rule_Should_Judge_Each_Source_Independently()
{
    let sources = Material_Every_Declared_Rule_Reports_On();

    for declaration in Every_Declaration()
    {
        let together = Rendered(&Judged_By(|facts| return declaration.Judges(&sources, facts)));
        let separately = Rendered(&Each_Source_Judged_Alone(&declaration, &sources));

        assert!(
            together == separately,
            "{}: judging the corpus whole reported something judging each file alone did not.\ntogether:\n{}\nseparately:\n{}",
            declaration.id,
            String::from_utf8_lossy(&together),
            String::from_utf8_lossy(&separately)
        );
    }
}

/// `OD-RULES-034`'s payload refusal, measured over the whole range of the derivation rather
/// than over the three declarations that exist.
///
/// A declaration does not state what it requires: [`DeclaredTextRule::Requires`] derives it
/// from the one thing a declaration says about facts. So the complete set of families any
/// declaration can ever reach is the image of that derivation, which is checked here to be
/// the test-material policy and nothing else -- no declaration can ask for a materialized
/// payload, so none can judge one.
#[test]
fn Test_No_Declaration_Should_Be_Able_To_Require_A_Materialized_Payload()
{
    for sensitivity in [TestMaterialSensitivity::Judged, TestMaterialSensitivity::Excluded]
    {
        let declaration = DeclaredTextRule { test_material: sensitivity, ..EVERY_ALLOW_DECLARATION };

        assert!(
            declaration.Requires().iter().all(|required| return matches!(*required, RequiredFact::TestMaterialPolicy)),
            "{sensitivity:?} reaches a family beyond the one that parameterizes the gate"
        );
    }
}

/// The subject a declaration derives and the families it derives are one answer or the
/// table's two tests disagree about the same row.
#[test]
fn Test_A_Declarations_Subject_Should_Agree_With_What_It_Requires()
{
    for sensitivity in [TestMaterialSensitivity::Judged, TestMaterialSensitivity::Excluded]
    {
        let declaration = DeclaredTextRule { test_material: sensitivity, ..EVERY_ALLOW_DECLARATION };
        let reads_a_fact = !declaration.Requires().is_empty();

        assert_eq!(reads_a_fact, declaration.Subject() == SubjectKind::SourceFacts, "{sensitivity:?}");
    }
}

/// The one refusal that is not impossible by construction, asserted over every declaration
/// this crate states rather than over the one the table would have caught.
///
/// `Descriptor_For_Declaration` asserts the same thing in a `const` context, so a
/// malformed declaration in `DESCRIPTORS` does not compile at all. This says it again where
/// a declaration written but not yet composed can still be caught, and names the property so
/// a reader of the failure knows what was refused.
#[test]
fn Test_Every_Declaration_Should_Be_Well_Formed()
{
    for declaration in Every_Declaration()
    {
        assert!(declaration.Is_Well_Formed(), "{}: names a parameter its detector does not take", declaration.id);
    }
}

/// Every declaration states a message, which is the one field with no default and no
/// derivation: a declaration with an empty one would report a finding saying nothing.
#[test]
fn Test_Every_Declaration_Should_Carry_A_Message()
{
    for declaration in Every_Declaration()
    {
        assert!(!declaration.message.is_empty(), "{}", declaration.id);
    }
}

/// A declaration's identity is the rule's, not a second one: the message a declaration
/// carries is the message the function it replaced carried, and the id is the same string
/// the table always named.
#[test]
fn Test_A_Declared_Rule_Should_Keep_The_Identity_Of_The_Rule_It_Declares()
{
    let sources = Material_Every_Declared_Rule_Reports_On();

    for declaration in Every_Declaration()
    {
        let reported = Judged_By(|facts| return declaration.Judges(&sources, facts));

        assert!(reported.iter().all(|finding| return finding.rule.As_Str() == declaration.id), "{}", declaration.id);
    }
}

/// Runs one comparison: the function's bytes against the declaration's, over `sources`.
fn Assert_Byte_Identical(sources: &[SourceFile], declaration: &DeclaredTextRule, linked: LinkedRule)
{
    let from_function = Rendered(&Judged_By(|facts| return (linked.0)(sources, facts)));
    let from_declaration = Rendered(&Judged_By(|facts| return declaration.Judges(sources, facts)));

    assert!(
        from_function == from_declaration,
        "{}: the declaration does not report what the function reported.\nfunction:\n{}\ndeclaration:\n{}",
        declaration.id,
        String::from_utf8_lossy(&from_function),
        String::from_utf8_lossy(&from_declaration)
    );
}

/// What a comparison compares: the findings' own rendering, as bytes.
///
/// A derived `Debug` prints every field a [`Finding`] carries, so two renderings that match
/// are two findings that match in rule, subject, line, message, category, evidence and
/// applicability -- which is the whole of what a consumer of a finding reads. A count would
/// have matched on any two of them.
fn Rendered(findings: &[Finding]) -> Vec<u8>
{
    return format!("{findings:#?}").into_bytes();
}

/// Runs `judge` against a fresh, empty reader.
///
/// Empty on purpose: this workspace declares no `nomos.cap.test.material.policy` fact in a
/// unit test, so the read misses and the test-material criterion resolves to its own fixed
/// clauses -- identically for the function and for the declaration, which is the only thing
/// the comparison needs of it.
fn Judged_By(judge: impl FnOnce(&mut dyn FactReader) -> Vec<Finding>) -> Vec<Finding>
{
    let store = MemoryFactStore::New();
    let registry = Registry::New();
    let mut facts = Reader::On(&store, &registry, Test_Context());

    return judge(&mut facts);
}

/// Each source judged on its own, gathered and ordered the way the interpreter orders a
/// whole run's findings.
fn Each_Source_Judged_Alone(declaration: &DeclaredTextRule, sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let alone = std::slice::from_ref(source);
        findings.extend(Judged_By(|facts| return declaration.Judges(alone, facts)));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// How many lines of `sources` this declaration's detector matches, read the way the engine
/// reads them.
fn Construct_Sites_In(sources: &[SourceFile], declaration: DeclaredTextRule) -> usize
{
    let detects = declaration.detector.As_Predicate();

    return sources
        .iter()
        .flat_map(|source| return source.text.lines())
        .filter(|line| return detects(&Code_With_String_Bodies_Masked(&Code_Prefix(line))))
        .count();
}

/// Material each of the three rules reports on, plus the cases each one must not report on.
///
/// Written out rather than walked, because the point of it is the cases this tree does not
/// happen to contain: a justified construct, a test source, this family's own implementation
/// module, and a path that merely ends like that module. Each is a place a declaration and
/// the function it replaced could diverge without either being obviously wrong.
///
/// The order is load-bearing and is the reason the `vendored/` entry is first. Every rule
/// here ends by ordering its findings on `subject_name`, so a corpus already in that order
/// would compare equal whether or not the interpreter ordered anything -- and the ordering
/// is one of the four things a declaration does *not* state and therefore has to inherit.
/// Handed in this order, each of the three rules reports a `vendored/...` finding before a
/// finding that sorts ahead of it.
fn Material_Every_Declared_Rule_Reports_On() -> Vec<SourceFile>
{
    return [
        ("vendored/crates/rules/nomos-rules/src/checks/rust_text/declarations.rs", "#[allow(clippy::redundant_clone)]\n#[inline(always)]\n#[ignore]\n"),
        ("crates/rules/nomos-rules/src/checks/rust_text/declarations.rs", "#[allow(clippy::redundant_clone)]\n#[inline(always)]\n#[ignore]\n"),
        ("src/unexplained_allow.rs", "#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n"),
        ("src/explained_allow.rs", "// the clone is required by the trait this implements\n#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n"),
        ("src/hot_path.rs", "#[inline(always)]\npub fn Sample_Texel() -> u8 { return 0; }\n"),
        ("tests/fixture.rs", "#[allow(clippy::redundant_clone)]\nlet processed = input.clone();\n"),
        ("tests/disabled.rs", "#[ignore]\nfn Test_Something() {}\n"),
        ("tests/reasoned.rs", "#[ignore = \"requires a live network\"]\nfn Test_Something() {}\n"),
    ]
    .into_iter()
    .map(|(path, text)| return Source_Of(path, text))
    .collect();
}

/// This repository's own Rust sources, as a run would hand them to a rule.
fn This_Repositorys_Rust_Sources() -> Vec<SourceFile>
{
    let root = Repository_Root();
    let mut sources = Vec::new();

    Collect_Rust_Sources_Under(&root, &root, &mut sources);
    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

/// This repository's root, from this crate's own manifest directory.
///
/// Read from `CARGO_MANIFEST_DIR` rather than from the working directory, because a test
/// binary's working directory is the package's and a shared runner's is not promised to be
/// anything -- and a walk rooted at the wrong place is the corpus-that-is-not-there failure
/// the floor assertion exists to catch, arriving by a different route.
fn Repository_Root() -> std::path::PathBuf
{
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .ancestors()
        .nth(3)
        .expect("this crate sits three directories below the repository root")
        .to_path_buf();
}

fn Collect_Rust_Sources_Under(root: &Path, directory: &Path, sources: &mut Vec<SourceFile>)
{
    let Ok(entries) = std::fs::read_dir(directory)
    else
    {
        return;
    };

    for entry in entries.flatten()
    {
        Collect_Rust_Sources_At(root, &entry.path(), sources);
    }
}

fn Collect_Rust_Sources_At(root: &Path, path: &Path, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        if !Is_Skipped_Directory(path)
        {
            Collect_Rust_Sources_Under(root, path, sources);
        }
        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs")
    {
        sources.extend(Rust_Source_At(root, path));
    }
}

fn Is_Skipped_Directory(path: &Path) -> bool
{
    let Some(name) = path.file_name().and_then(|name| return name.to_str())
    else
    {
        return true;
    };

    return name == BUILD_OUTPUT_DIRECTORY || name.starts_with('.');
}

fn Rust_Source_At(root: &Path, path: &Path) -> Option<SourceFile>
{
    let text = std::fs::read_to_string(path).ok()?;
    let relative = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");

    return Some(Source_Of(&relative, &text));
}

/// One source in the shape the composition root hands a rule: a repo-relative path, the
/// subject that path is filed under, and the language the walk recognized.
fn Source_Of(path: &str, text: &str) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);

    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}
