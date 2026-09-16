use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
use nomos_model::Content_Digest;

const PARSER: &str = "nomos.test.naming.parses";

#[test]
fn Test_Payload_Of_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("fn bad_name() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
        Check_Naming_Convention,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "bad_name");
}

/// `path` and `text` are both `&str`; without a distinct type per position a call site
/// like `Source_File("src/lib.rs", "fn bad_name() {}")` reads as two interchangeable
/// strings and a swap compiles silently. These wrappers give each position a type the
/// other cannot satisfy.
#[derive(Clone, Copy)]
struct Path<'a>(&'a str);

#[derive(Clone, Copy)]
struct Text<'a>(&'a str);

#[test]
fn Test_Check_Naming_Convention_Should_Report_A_Subject_With_No_Fact_Rather_Than_Silently_Clean()
{
    let source = Source_File(Path("src/lib.rs"), Text("fn bad_name() {}"));
    let TestOffering { store, registry, .. } = Offering();

    let mut reader = Reader::On(&store, &registry, Test_Context());
    let findings = Check_Naming_Convention(&[source], &mut reader);

    assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
    assert_eq!(
        findings.first().expect("asserted len 1 above").subject_name,
        "src/lib.rs"
    );
}

#[test]
fn Test_Check_Project_Owned_Function_Names_Use_Upper_Snake_Case_Should_Report_Under_The_Code_Standards_Id()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("fn bad_name() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
        Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE));
    assert_eq!(found.gate, nomos_contracts::GateCategory::Blocking);
}

#[test]
fn Test_Check_Test_Names_Describe_Behavior_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("#[test]\nfn Test_Insert_Works() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPrivate\tTest_Insert_Works\t.\t+fn/0\n",
        Check_Test_Names_Describe_Behavior,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Test_Insert_Works");
}

#[test]
fn Test_Check_Data_Names_Stay_Lower_Snake_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("mod BadModule {}"),
        "unexpanded\t0\nitem\t0\tModule\tPrivate\tBadModule\t.\t.\n",
        Check_Data_Names_Stay_Lower_Snake,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadModule");
}

#[test]
fn Test_Check_File_Name_Matches_Declared_Type_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/orders.rs"),
        Text("pub struct OrderBook;"),
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n",
        Check_File_Name_Matches_Declared_Type,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "OrderBook");
}

#[test]
fn Test_Check_One_Public_Type_Per_File_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/order.rs"),
        Text("pub struct Order; pub enum OrderKind {}"),
        "unexpanded\t0\n\
         item\t0\tStruct\tPublic\tOrder\t.\t.\n\
         item\t1\tEnum\tPublic\tOrderKind\t.\t.\n",
        Check_One_Public_Type_Per_File,
    );

    const EXPECTED_PUBLIC_TYPE_COUNT: usize = 2;
    assert_eq!(findings.len(), EXPECTED_PUBLIC_TYPE_COUNT, "{findings:?}");
    assert!(findings.iter().all(|finding| return finding.rule == nomos_contracts::RuleId::New(ONE_PUBLIC_TYPE_PER_FILE)));
}

#[test]
fn Test_Check_Single_Letter_Names_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/point.rs"),
        Text("pub struct Point { x: f64 }"),
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tPoint\t.\t+fields\\nx\\tf64\n",
        Check_Single_Letter_Names,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "x");
}

#[test]
fn Test_Check_Boolean_Predicates_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/flag.rs"),
        Text("pub struct Flag { ready: bool }"),
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tFlag\t.\t+fields\\nready\\tbool\n",
        Check_Boolean_Predicates,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "ready");
}

#[test]
fn Test_Check_Go_Type_Names_Use_Camel_Case_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("types.go"),
        Text("type order_book struct{}"),
        "unexpanded\t0\nitem\t0\tStruct\tPrivate\torder_book\t.\t.\n",
        Check_Go_Type_Names_Use_Camel_Case,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE));
    assert_eq!(found.subject_name, "order_book");
}

#[test]
fn Test_Check_Exported_Go_Functions_Use_Upper_Snake_Case_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("main.go"),
        Text("func run_With_Backend() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPublic\trun_With_Backend\t.\t+fn/0\n",
        Check_Exported_Go_Functions_Use_Upper_Snake_Case,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE));
    assert_eq!(found.subject_name, "run_With_Backend");
}

#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("main.go"),
        Text("func rowBreaches() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPrivate\trowBreaches\t.\t+fn/0\n",
        Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER));
    assert_eq!(found.subject_name, "rowBreaches");
}

#[test]
fn Test_Check_Go_Constants_Split_By_Export_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("kinds.go"),
        Text("const KindRule = \"rule\""),
        "unexpanded\t0\nitem\t0\tConstant\tPublic\tKindRule\t.\t.\n",
        Check_Go_Constants_Split_By_Export,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(CONSTANTS_SPLIT_BY_EXPORT));
    assert_eq!(found.subject_name, "KindRule");
}

#[test]
fn Test_Check_Go_Variables_Use_Lower_Snake_Case_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("state.go"),
        Text("var entityID int"),
        "unexpanded\t0\nitem\t0\tVariable\tPrivate\tentityID\t.\t.\n",
        Check_Go_Variables_Use_Lower_Snake_Case,
    );

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.rule, nomos_contracts::RuleId::New(GO_VARIABLES_USE_LOWER_SNAKE_CASE));
    assert_eq!(found.subject_name, "entityID");
}

/// Builds `path`/`text` into a source, materializes `payload` as its syntax fact, and
/// returns what `check` finds — the shared shape every fact-backed test in this module
/// repeats up to its own assertions.
fn Findings_From(
    path: Path<'_>,
    text: Text<'_>,
    payload: &str,
    check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>,
) -> Vec<Finding>
{
    let source = Source_File(path, text);
    let TestOffering { mut store, registry, offer } = Offering();
    Materialize_Syntax_Fact(&mut store, &source, &offer, payload);

    let mut reader = Reader::On(&store, &registry, Test_Context());
    return check(&[source], &mut reader);
}

fn Materialize_Syntax_Fact(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &str)
{
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);
    test_support::Materialize_Fact(store, FactToFile { subject: source.subject, offer, semantic_inputs: inputs, schema: nomos_cap_syntax::Payload_Schema(), bytes: payload.as_bytes().to_vec() }).expect("the fixture's store holds no fact under this key at a newer generation");
}

fn Guarantee_At_Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

fn Source_File(path: Path<'_>, text: Text<'_>) -> SourceFile
{
    let mut source = SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
    source.language = crate::Recognized_Language_In_Tests(path.0);
    return source;
}

fn Offering() -> TestOffering
{
    return test_support::Offered_Registry(
        OfferedProvider {
            contract: nomos_cap_syntax::Capability_Contract(),
            capability: nomos_cap_syntax::Capability(),
            version: nomos_cap_syntax::CONTRACT_VERSION,
            provider: PARSER,
            guarantee: Guarantee_At_Floor(),
        },
    ).expect("a fresh Registry holds neither this contract nor this provider");
}
