//! The rule's judgment over hand-built values, and its reading through a real registry, store and
//! reader -- with a file's fact, with none, and with an offer below its floor.

use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_cap_go_types::{DiscardedValuesPayload, Encode_Payload};
use nomos_capability::ProviderOffer;
use nomos_model::Subject_Of_Path;

const FILE: &str = "cmd/tool/main.go";

/// A file discarding five values: an unexplained error, an explained one on the same line and one
/// explained on the line above, a string that is not an error, and two errors on one line.
const TEXT: &str = "package main

func main() {
\t_ = os.Remove(path)
\t_ = os.Remove(lock) // best-effort cleanup, nothing to do if it fails
\t// a missing directory is already the state we want
\t_ = os.Remove(dir)
\t_ = fmt.Sprintf(\"%d\", n)
\t_, _ = pair()
}
";

fn Value(line: u32, column: u32, is_error: bool, type_name: &str) -> DiscardedValue
{
    return DiscardedValue { line, column, is_error, type_name: type_name.to_owned() };
}

fn Values() -> Vec<DiscardedValue>
{
    return vec![
        Value(4, 2, true, "error"),
        Value(5, 2, true, "error"),
        Value(7, 2, true, "error"),
        Value(8, 2, false, "string"),
        Value(9, 2, true, "error"),
        Value(9, 5, true, "*os.PathError"),
    ];
}

fn Source() -> SourceFile
{
    return SourceFile::New(FILE, Subject_Of_Path(FILE), TEXT.to_owned());
}

fn Subject_Names(findings: &[Finding]) -> Vec<&str>
{
    return findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
}

/// The unexplained error is reported, both explained ones are not, the string is not an error at
/// all, and a line discarding two errors is reported once.
#[test]
fn Test_Only_An_Unexplained_Error_Should_Be_Reported_Once_Per_Line()
{
    let findings = Unexplained_Errors(&Source(), &Values());

    assert_eq!(Subject_Names(&findings), [format!("{FILE}:4"), format!("{FILE}:9")], "{findings:?}");
    let first = findings.first().expect("asserted two above");
    assert_eq!(first.rule, RuleId::New(A_DISCARDED_ERROR_IS_EXPLAINED));
    assert_eq!(first.gate, GateCategory::Blocking);
    assert!(first.summary.contains("discards an error (`error`)"), "{}", first.summary);
}

/// Through a real registry, store and reader, the file's fact under its own subject with the empty
/// inputs the provider files -- the reading a run makes.
#[test]
fn Test_Check_Should_Judge_A_Files_Fact_Through_The_Reader()
{
    let Offering { mut store, registry, offer } = Offering(FactVariant::SemanticallyResolved);
    let source = File_Answer(&mut store, &offer, Values());
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_A_Discarded_Error_Is_Explained(&[source], &mut reader);

    assert_eq!(Subject_Names(&findings), [format!("{FILE}:4"), format!("{FILE}:9")], "{findings:?}");
}

/// A file with no fact is reported as unread and never judged from its text, which would call
/// line 8's string an error.
#[test]
fn Test_A_File_With_No_Fact_Should_Be_Reported_Unread_And_Not_Judged()
{
    let Offering { store, registry, .. } = Offering(FactVariant::SemanticallyResolved);
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_A_Discarded_Error_Is_Explained(&[Source()], &mut reader);

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted one above");
    assert!(found.applicability.Is_Coverage_Debt(), "{found:?}");
    assert_eq!(found.subject_name, FILE);
}

/// An offer that reads the file on its face -- `Syntactic` -- is below this rule's floor, so its
/// answer is not believed however it is filed.
#[test]
fn Test_Check_Should_Not_Believe_An_Answer_Below_Its_Floor()
{
    let Offering { mut store, registry, offer } = Offering(FactVariant::Syntactic);
    let source = File_Answer(&mut store, &offer, Values());
    let mut reader = Reader::On(&store, &registry, Test_Context());

    let findings = Check_A_Discarded_Error_Is_Explained(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "an unbelieved answer is reported, never passed: {findings:?}");
    assert!(findings.iter().all(|finding| return finding.applicability.Is_Coverage_Debt()), "{findings:?}");
}

struct Offering
{
    store: MemoryFactStore,
    registry: nomos_capability::Registry,
    offer: ProviderOffer,
}

fn Offering(variant: FactVariant) -> Offering
{
    let TestOffering { store, registry, offer } = test_support::Offered_Registry(OfferedProvider {
        contract: nomos_cap_go_types::Capability_Contract(),
        capability: nomos_cap_go_types::Capability(),
        version: nomos_cap_go_types::CONTRACT_VERSION,
        provider: "nomos.test.go.types",
        guarantee: Guarantee::New(variant, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File),
    })
    .expect("a fresh Registry holds neither this contract nor this provider");

    return Offering { store, registry, offer };
}

/// Files `values` under the file's subject and returns the source a run would hand the rule.
fn File_Answer(store: &mut MemoryFactStore, offer: &ProviderOffer, values: Vec<DiscardedValue>) -> SourceFile
{
    let source = Source();
    test_support::Materialize_Fact(store, FactToFile {
        subject: source.subject,
        offer,
        semantic_inputs: InputDigest::Of(&[]),
        schema: nomos_cap_go_types::Payload_Schema(),
        bytes: Encode_Payload(&DiscardedValuesPayload { values }),
    })
    .expect("the fixture's store holds no fact under this key at a newer generation");

    return source;
}
