//! The rule's judgment over hand-built jumps, and its reading of a file's sites through a real
//! registry, store and reader.
//!
//! Every case of the corpus engine's own tests at code-standards `f0d820729` is ported here:
//! `judgment_test.go` and `noise_label_test.go` against [`Noise_Labels`] and [`Message`] with the
//! jumps those tests build, and `labeled_jump_test.go`'s dispatch -- a language's jumps are the
//! ones read, and a file nobody claims is not judged -- against [`Check_A_Labeled_Jump_Leaves_One_Loop`]
//! over filed facts. None could not be ported. The two judgments `rust_labeled_loop_test.go` asks of
//! `Noise_Labels` are ported too, over the sites `nomos-lang-rust`'s `tests/sites_capability.rs`
//! pins its provider to for the same sources.

use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_syntax::{KindDecline, Render_Sites_Payload};
use nomos_capability::ProviderOffer;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// A jump as the corpus engine's tests build one: the fields they set, the rest as a parse of that
/// jump would leave them.
fn Jump(keyword: &str, label: &str, line: usize, label_line: usize, has_same_target_unlabeled: bool) -> LabeledJump
{
    return LabeledJump {
        line,
        keyword: keyword.to_owned(),
        label: label.to_owned(),
        text: format!("{keyword} {label}"),
        label_line,
        has_same_target_unlabeled,
        is_inside_break_capture: false,
    };
}

fn Noise_Lines(jumps: &[LabeledJump]) -> Vec<usize>
{
    return Noise_Labels(jumps).iter().map(|noise| return noise.line).collect();
}

/// `Test_A_Label_No_Jump_To_It_Needs_Is_Noise`: every jump naming the label lands in the same place
/// without it, so the label is the finding, on the label's line, and the remedy is deletion.
#[test]
fn Test_A_Label_No_Jump_To_It_Needs_Should_Be_Noise()
{
    let jumps = [Jump("break", "Loop", 4, 2, true), Jump("break", "Loop", 8, 2, true)];

    let noise = Noise_Labels(&jumps);

    assert_eq!(noise.iter().map(|label| return label.line).collect::<Vec<_>>(), vec![2]);
    assert!(Message(noise.first().expect("asserted one above")).contains("Delete the label"));
}

/// `Test_A_Jump_That_Leaves_Two_Loops_Is_Permitted`: crossing loop levels is what a label is for.
#[test]
fn Test_A_Jump_That_Leaves_Two_Loops_Should_Be_Permitted()
{
    assert_eq!(Noise_Lines(&[Jump("break", "'outer", 4, 1, false)]), Vec::<usize>::new());
}

/// `Test_A_Label_One_Jump_Needs_Keeps_Its_Name_On_The_Others`: judged jump by jump, half of a name
/// the loop must still carry would be deleted.
#[test]
fn Test_A_Label_One_Jump_Needs_Should_Keep_Its_Name_On_The_Others()
{
    assert_eq!(Noise_Lines(&[Jump("break", "Loop", 4, 2, true), Jump("break", "Loop", 9, 2, false)]), Vec::<usize>::new());
}

/// `Test_A_Jump_Standing_Inside_A_Break_Capture_Is_Permitted`: a `continue` inside a Go `switch`
/// names its loop deliberately, though a bare `continue` would reach it.
#[test]
fn Test_A_Jump_Standing_Inside_A_Break_Capture_Should_Be_Permitted()
{
    let mut inside = Jump("continue", "Loop", 6, 3, true);
    inside.is_inside_break_capture = true;

    assert_eq!(Noise_Lines(&[inside]), Vec::<usize>::new());
}

/// `Test_Two_Labels_With_The_Same_Name_Are_Two_Labels`: grouped by the line the label is written on,
/// so the `L` that needs its name does not excuse the `L` that does not.
#[test]
fn Test_Two_Labels_With_The_Same_Name_Should_Be_Two_Labels()
{
    assert_eq!(Noise_Lines(&[Jump("continue", "L", 12, 10, false), Jump("continue", "L", 42, 40, true)]), vec![40]);
}

/// `Test_The_Message_Counts_One_Jump_In_The_Singular`.
#[test]
fn Test_The_Message_Should_Count_One_Jump_In_The_Singular()
{
    let jumps = [Jump("break", "Loop", 4, 2, true)];
    let noise = Noise_Labels(&jumps);

    assert!(Message(noise.first().expect("one noise label")).contains("the one jump that names it would land"));
}

/// The plural the singular case is the exception to.
#[test]
fn Test_The_Message_Should_Count_Several_Jumps_In_The_Plural()
{
    let jumps = [Jump("break", "Loop", 4, 2, true), Jump("continue", "Loop", 6, 2, true)];
    let noise = Noise_Labels(&jumps);

    assert!(Message(noise.first().expect("one noise label")).contains("the 2 jumps that name it would each land"));
}

/// `Test_Noise_Labels_Flags_The_Label_No_Jump_Needs`: the finding lands on the label line and
/// carries its jumps; a label a jump genuinely needs is not noise.
#[test]
fn Test_Noise_Labels_Should_Flag_The_Label_No_Jump_Needs()
{
    let jumps = [Jump("break", "Loop", 4, 2, true), Jump("break", "Loop", 8, 2, true)];

    assert_eq!(Noise_Labels(&jumps), vec![NoiseLabel { line: 2, label: "Loop", jumps: 2 }]);
    assert_eq!(Noise_Lines(&[Jump("break", "Loop", 4, 2, false)]), Vec::<usize>::new());
}

/// `Test_Message_Says_Delete_The_Label`: it names the label and says to delete it.
#[test]
fn Test_Message_Should_Name_The_Label_And_Say_To_Delete_It()
{
    let jumps = [Jump("break", "Loop", 4, 2, true)];
    let noise = Noise_Labels(&jumps);
    let message = Message(noise.first().expect("one noise label"));

    assert!(message.contains("Loop") && message.contains("Delete the label") && message.contains("one jump"), "{message}");
}

/// `rust_labeled_loop_test.go`'s two judgments: the doubly-nested search keeps its label, and a
/// label on the loop a `match` arm jumps out of is noise.
#[test]
fn Test_The_Rust_Kernels_Sites_Should_Be_Judged_As_The_Corpus_Judges_Them()
{
    assert_eq!(Noise_Lines(&[Jump("break", "'outer", 6, 3, false)]), Vec::<usize>::new());
    assert_eq!(Noise_Lines(&[Jump("break", "'loop_kinds", 5, 3, true)]), vec![3]);
}

fn Source(path: &str) -> SourceFile
{
    let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), "fn f() {}\n".to_owned());
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}

fn Sites_Offering() -> TestOffering
{
    return test_support::Offered_Registry(OfferedProvider {
        contract: nomos_cap_syntax::Sites_Capability_Contract(),
        capability: Sites_Capability(),
        version: SITES_CONTRACT_VERSION,
        provider: "nomos.test.sites",
        guarantee: Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
    })
    .expect("a fresh registry holds neither this contract nor this provider");
}

fn File_Sites(store: &mut MemoryFactStore, offer: &ProviderOffer, source: &SourceFile, bytes: Vec<u8>)
{
    test_support::Materialize_Fact(store, FactToFile {
        subject: source.subject,
        offer,
        semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
        schema: Sites_Payload_Schema(),
        bytes,
    })
    .expect("the fixture's store holds no fact under this key at a newer generation");
}

/// The rule's findings over one source whose sites are `bytes`.
fn Findings_Over(source: &SourceFile, bytes: Vec<u8>) -> Vec<Finding>
{
    let TestOffering { mut store, registry, offer } = Sites_Offering();
    File_Sites(&mut store, &offer, source, bytes);
    let mut reader = Reader::On(&store, &registry, test_support::Test_Context());

    return Check_A_Labeled_Jump_Leaves_One_Loop(core::slice::from_ref(source), &mut reader);
}

fn Offering_Jumps(jumps: &[LabeledJump]) -> Vec<u8>
{
    return Render_Sites_Payload(&SitesPayload {
        offered: vec![LABELED_JUMP.name.to_owned()],
        declined: Vec::new(),
        records: jumps.iter().map(LabeledJump::Record).collect(),
    });
}

/// `Test_Labeled_Jumps_For_Returns_The_Languages_Jumps`, first half: the jumps judged are the ones
/// the file's provider filed, and a noise label among them is a blocking finding on its own line.
#[test]
fn Test_Check_Should_Report_A_Noise_Label_In_The_Sites_Its_Provider_Filed()
{
    let source = Source("a.rs");

    let findings = Findings_Over(&source, Offering_Jumps(&[Jump("break", "'each", 4, 2, true)]));

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.subject_name, "a.rs:2");
    assert_eq!(found.gate, GateCategory::Blocking);
    assert_eq!(found.applicability, Applicability::PartiallySupported);
    assert!(found.summary.contains("`'each`"), "{}", found.summary);
}

/// An offered kind with no records is a clean file: no finding at all.
#[test]
fn Test_Check_Should_Pass_A_File_Whose_Provider_Offers_The_Kind_And_Found_None()
{
    assert_eq!(Findings_Over(&Source("a.rs"), Offering_Jumps(&[])), Vec::new());
}

/// A declined kind is `NotApplicable`, with the provider's reason, and never a pass.
#[test]
fn Test_Check_Should_Report_A_Declined_Kind_As_Not_Applicable_With_Its_Reason()
{
    let declining = Render_Sites_Payload(&SitesPayload {
        offered: Vec::new(),
        declined: vec![KindDecline { kind: LABELED_JUMP.name.to_owned(), reason: "this language cannot name a loop".to_owned() }],
        records: Vec::new(),
    });

    let findings = Findings_Over(&Source("a.go"), declining);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert_eq!(found.applicability, Applicability::NotApplicable);
    assert!(found.summary.contains("this language cannot name a loop"), "{}", found.summary);
}

/// A payload neither offering nor declining the kind is a gap: `MissingCapability`, never a pass.
#[test]
fn Test_Check_Should_Report_An_Unanswered_Kind_As_Missing_Capability()
{
    let findings = Findings_Over(&Source("a.go"), Vec::new());

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted one above").applicability, Applicability::MissingCapability);
}

/// Sites that are not this schema are a source not judged, not a clean one.
#[test]
fn Test_Check_Should_Report_Unreadable_Sites_Rather_Than_Pass_Them()
{
    let findings = Findings_Over(&Source("a.rs"), b"offers\tlabelled-jump\n".to_vec());

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted one above").applicability, Applicability::Unparseable);
}

/// A source with no sites fact at all is coverage debt, never a clean source.
#[test]
fn Test_Check_Should_Report_A_Source_With_No_Sites_Rather_Than_Pass_It()
{
    let TestOffering { store, registry, .. } = Sites_Offering();
    let mut reader = Reader::On(&store, &registry, test_support::Test_Context());

    let findings = Check_A_Labeled_Jump_Leaves_One_Loop(&[Source("a.rs")], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings.first().expect("asserted one above").applicability.Is_Coverage_Debt(), "{findings:?}");
}

/// `Test_Labeled_Jumps_For_Returns_The_Languages_Jumps`, second half: a file no language claims is
/// not judged -- the corpus's `OUTCOME_UNCLAIMED` -- so it reports nothing rather than a gap.
#[test]
fn Test_Check_Should_Not_Judge_A_File_No_Syntax_Provider_Recognizes()
{
    let TestOffering { store, registry, .. } = Sites_Offering();
    let mut reader = Reader::On(&store, &registry, test_support::Test_Context());

    assert_eq!(Check_A_Labeled_Jump_Leaves_One_Loop(&[Source("notes.md")], &mut reader), Vec::new());
}
