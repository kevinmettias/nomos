use super::*;
use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
use crate::RUST_LANGUAGE;
use nomos_analysis::{InputDigest, MemoryFactStore, Reader};
use nomos_cap_naming_policy::{NamingPolicyPayload, PolicyRow};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId};
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
fn Test_Check_Module_And_Field_Names_Stay_Lower_Snake_Should_Read_And_Judge_A_Real_Fact()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("mod BadModule {}"),
        "unexpanded\t0\nitem\t0\tModule\tPrivate\tBadModule\t.\t.\n",
        Check_Module_And_Field_Names_Stay_Lower_Snake,
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

// The five tests below are the converse of the fact-backed tests above, and they exist because
// those could not tell one default from another: `run_With_Backend` breaks upper snake case and
// lower snake case alike, so a rule judging against the wrong default reported it all the same.
// A name that conforms to the axis's declared default, judged with nothing declared, must be
// accepted -- which fails the moment the default changes, and so proves the rule reads it from
// `rule_descriptor::policy_axis` rather than from anywhere else.

#[test]
fn Test_Check_Module_And_Field_Names_Stay_Lower_Snake_Should_Accept_A_Lower_Snake_Field_With_Nothing_Declared()
{
    let findings = Findings_From(
        Path("src/lib.rs"),
        Text("pub struct Config { worker_count: usize }"),
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t+fields\\nworker_count\\tusize\n",
        Check_Module_And_Field_Names_Stay_Lower_Snake,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Exported_Go_Functions_Use_Upper_Snake_Case_Should_Accept_An_Upper_Snake_Name_With_Nothing_Declared()
{
    let findings = Findings_From(
        Path("main.go"),
        Text("func Run_With_Backend() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPublic\tRun_With_Backend\t.\t+fn/0\n",
        Check_Exported_Go_Functions_Use_Upper_Snake_Case,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_Accept_A_Mixed_Snake_Name_With_Nothing_Declared()
{
    let findings = Findings_From(
        Path("main.go"),
        Text("func row_Breaches() {}"),
        "unexpanded\t0\nitem\t0\tFunction\tPrivate\trow_Breaches\t.\t+fn/0\n",
        Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Go_Type_Names_Use_Camel_Case_Should_Accept_An_Upper_Camel_Exported_Type_With_Nothing_Declared()
{
    let findings = Findings_From(
        Path("types.go"),
        Text("type OrderBook struct{}"),
        "unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n",
        Check_Go_Type_Names_Use_Camel_Case,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn Test_Check_Go_Type_Names_Use_Camel_Case_Should_Accept_A_Lower_Camel_Unexported_Type_With_Nothing_Declared()
{
    let findings = Findings_From(
        Path("types.go"),
        Text("type orderBook struct{}"),
        "unexpanded\t0\nitem\t0\tStruct\tPrivate\torderBook\t.\t.\n",
        Check_Go_Type_Names_Use_Camel_Case,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

// The tests below read a repository's declared naming policy, which none above does: each one
// judges a source against a declaration that differs from the default, so a rule that ignored
// the declaration, or read it through the wrong key or in the wrong order, reports a different
// set of names. `OD-RULES-035` decision 7 is the order they hold the rule to.

/// A pub `as_str` and a private `to_text`, both lower-snake: the two sides of visibility, each
/// spelled so that only a declaration can make it conform.
const PUB_AND_PRIVATE: &str = "unexpanded\t0\n\
     item\t0\tFunction\tPublic\tas_str\t.\t+fn/1\n\
     item\t1\tFunction\tPrivate\tto_text\t.\t+fn/1\n";

/// xvpe's own shape -- a case declared for Rust under `function.exported` and nowhere under
/// `function` -- with a case that differs from the default: a pub function is judged against the
/// declaration, and a private one, whose own refinement nobody declared, against the default.
#[test]
fn Test_Check_Naming_Convention_Should_Judge_A_Pub_Rust_Function_Against_A_Declared_Function_Exported()
{
    let rows = vec![Row(Scope::Language(RUST_LANGUAGE.to_owned()), "function.exported", Case::LowerSnake)];

    let findings = Findings_Under_Policy(Path("src/lib.rs"), PUB_AND_PRIVATE, rows);

    assert_eq!(Judged_Names(&findings), vec!["to_text"], "{findings:?}");
}

/// The other side: `function.unexported` is read for every Rust function that is not exactly
/// `pub`, a `pub(crate)` one included.
#[test]
fn Test_Check_Naming_Convention_Should_Judge_Every_Other_Rust_Function_Against_A_Declared_Function_Unexported()
{
    let rows = vec![Row(Scope::Language(RUST_LANGUAGE.to_owned()), "function.unexported", Case::LowerSnake)];
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPublic\tas_str\t.\t+fn/1\n\
                   item\t1\tFunction\tPrivate\tto_text\t.\t+fn/1\n\
                   item\t2\tFunction\tRestricted(crate)\tfrom_parts\t.\t+fn/2\n";

    let findings = Findings_Under_Policy(Path("src/lib.rs"), payload, rows);

    assert_eq!(Judged_Names(&findings), vec!["as_str"], "{findings:?}");
}

/// A refinement is read before the key it refines, even across scopes: a repository-wide
/// `function.exported` decides a pub function ahead of a `function` declared for Rust, which still
/// decides the private one. code-standards resolves the same declaration the same way.
#[test]
fn Test_Check_Naming_Convention_Should_Read_A_Refinement_Ahead_Of_The_Key_It_Refines()
{
    let rows = vec![
        Row(Scope::Language(RUST_LANGUAGE.to_owned()), "function", Case::LowerSnake),
        Row(Scope::Repository, "function.exported", Case::UpperSnake),
    ];

    let findings = Findings_Under_Policy(Path("src/lib.rs"), PUB_AND_PRIVATE, rows);

    assert_eq!(Judged_Names(&findings), vec!["as_str"], "{findings:?}");
}

/// The plain key, which three of the four repositories that declare a Rust function case write,
/// still decides both sides when no refinement is declared -- for Rust as well as repository-wide.
#[test]
fn Test_Check_Naming_Convention_Should_Judge_Both_Sides_Against_A_Declared_Function_When_No_Refinement_Is_Declared()
{
    for scope in [Scope::Repository, Scope::Language(RUST_LANGUAGE.to_owned())]
    {
        let rows = vec![Row(scope.clone(), "function", Case::LowerSnake)];

        let findings = Findings_Under_Policy(Path("src/lib.rs"), PUB_AND_PRIVATE, rows);

        assert!(findings.is_empty(), "{scope:?}: {findings:?}");
    }
}

// `OD-RULES-035` decision 8 has every language read as decision 7 has Rust read, for the language
// the source is written in, and a Go method read the method keys first. The tests below each
// declare what the rule would not otherwise judge against.

/// xvpe's and code-standards' own shape: a Go function case declared only under
/// `languages.go.naming`, as a refinement. The rule once read `function` repository-wide alone for
/// any language but Rust, and reported this name against upper-snake.
#[test]
fn Test_Check_Naming_Convention_Should_Read_A_Refinement_Declared_For_The_Language_A_Function_Is_Written_In()
{
    let rows = vec![Row(Scope::Language(GO_LANGUAGE.to_owned()), "function.unexported", Case::MixedSnake)];
    let payload = "unexpanded\t0\nitem\t0\tFunction\tPrivate\trun_With_Backend\t.\t+fn/0\n";

    let findings = Findings_Under_Policy(Path("main.go"), payload, rows);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The plain key declared for one language decides that language's functions and no other's: a Go
/// function is judged against the lower-snake declared for Go, a Rust function against the
/// upper-snake nothing declared for it.
#[test]
fn Test_Check_Naming_Convention_Should_Read_Function_Declared_For_A_Language_Only_For_That_Language()
{
    let rows = vec![Row(Scope::Language(GO_LANGUAGE.to_owned()), "function", Case::LowerSnake)];
    let payload = "unexpanded\t0\nitem\t0\tFunction\tPrivate\trun_with_backend\t.\t+fn/0\n";

    let go = Findings_Under_Policy(Path("main.go"), payload, rows.clone());
    let rust = Findings_Under_Policy(Path("src/lib.rs"), payload, rows);

    assert!(go.is_empty(), "{go:?}");
    assert_eq!(Judged_Names(&rust), vec!["run_with_backend"], "{rust:?}");
}

/// A Go method is told apart from a free function, and reads `method.unexported` and then `method`
/// ahead of every function key; the free function beside it reads the function keys alone.
#[test]
fn Test_Check_Naming_Convention_Should_Read_The_Method_Keys_Ahead_Of_The_Function_Keys_For_A_Go_Method()
{
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPrivate\trowCount\t.\t+fn/0\n\
                   item\t1\tFunction\tPrivate\tTable::rowCount\t.\t+fn/1\n";
    for method_key in ["method.unexported", "method"]
    {
        let rows = vec![
            Row(Scope::Repository, "function.unexported", Case::LowerSnake),
            Row(Scope::Language(GO_LANGUAGE.to_owned()), method_key, Case::LowerCamel),
        ];

        let findings = Findings_Under_Policy(Path("table.go"), payload, rows);

        assert_eq!(Judged_Names(&findings), vec!["rowCount"], "{method_key}: {findings:?}");
    }
}

/// A Rust function inside an `impl` block reads no method key: the syntax fact cannot say whether
/// it takes a receiver, which is what code-standards means by a Rust method, so the function keys
/// still decide it.
#[test]
fn Test_Check_Naming_Convention_Should_Not_Read_A_Method_Key_For_A_Rust_Function()
{
    let rows = vec![Row(Scope::Repository, "method", Case::LowerSnake), Row(Scope::Language(RUST_LANGUAGE.to_owned()), "method", Case::LowerSnake)];
    let payload = "unexpanded\t0\n\
                   item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                   item\t1\tFunction\tPublic\tTable::as_str\t.\t+fn/1\n";

    let findings = Findings_Under_Policy(Path("src/lib.rs"), payload, rows);

    assert_eq!(Judged_Names(&findings), vec!["Table::as_str"], "{findings:?}");
}

/// The exported Go rule falls back to `function`, as code-standards does: a declared upper-camel
/// reports the upper-snake name the rule's own default would accept.
#[test]
fn Test_Check_Exported_Go_Functions_Use_Upper_Snake_Case_Should_Read_Function_When_No_Refinement_Is_Declared()
{
    let rows = vec![Row(Scope::Repository, "function", Case::UpperCamel)];
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPublic\tRun_With_Backend\t.\t+fn/0\n\
                   item\t1\tFunction\tPublic\tRunWithBackend\t.\t+fn/0\n";

    let findings = Judged_Under_Policy(Check_Exported_Go_Functions_Use_Upper_Snake_Case, Path("main.go"), payload, rows);

    assert_eq!(Summaries(&findings), vec!["exported Go function `Run_With_Backend` is not upper-camel case"], "{findings:?}");
}

/// This repository's own shape -- `function` upper-snake and no Go key -- read by the unexported
/// Go rule: a mixed-snake name its default accepts is reported, and `main` is not judged at all.
#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_Read_Function_When_No_Refinement_Is_Declared()
{
    let rows = vec![Row(Scope::Repository, "function", Case::UpperSnake)];
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPrivate\tmain\t.\t+fn/0\n\
                   item\t1\tFunction\tPrivate\trow_Breaches\t.\t+fn/0\n";

    let findings = Judged_Under_Policy(Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter, Path("main.go"), payload, rows);

    assert_eq!(Summaries(&findings), vec!["unexported Go function `row_Breaches` is not upper-snake case"], "{findings:?}");
}

/// The Go rules read in decision 7's order: `function` declared for Go ahead of `function`
/// repository-wide, and a refinement ahead of either, wherever each is declared.
#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_Read_The_Most_Specific_Key_First()
{
    let payload = "unexpanded\t0\nitem\t0\tFunction\tPrivate\trow_breaches\t.\t+fn/0\n";
    let go_function = vec![Row(Scope::Repository, "function", Case::UpperSnake), Row(Scope::Language(GO_LANGUAGE.to_owned()), "function", Case::LowerSnake)];
    let refinement = vec![Row(Scope::Language(GO_LANGUAGE.to_owned()), "function", Case::UpperSnake), Row(Scope::Repository, "function.unexported", Case::LowerSnake)];

    for rows in [go_function, refinement]
    {
        let findings = Judged_Under_Policy(Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter, Path("main.go"), payload, rows.clone());

        assert!(findings.is_empty(), "{rows:?}: {findings:?}");
    }
}

/// Each Go rule judges a method against the method keys ahead of the function keys: a declared
/// `method.unexported` or `method` decides `Table::rowCount`, and the free `rowCount` beside it is
/// still judged against the rule's mixed-snake default.
#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_Read_The_Method_Keys_For_A_Method()
{
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPrivate\trowCount\t.\t+fn/0\n\
                   item\t1\tFunction\tPrivate\tTable::rowCount\t.\t+fn/1\n";
    for method_key in ["method.unexported", "method"]
    {
        let rows = vec![Row(Scope::Language(GO_LANGUAGE.to_owned()), method_key, Case::LowerCamel)];

        let findings = Judged_Under_Policy(Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter, Path("table.go"), payload, rows);

        assert_eq!(Summaries(&findings), vec!["unexported Go function `rowCount` is not mixed-snake case"], "{method_key}: {findings:?}");
    }
}

/// The exported side of the same read.
#[test]
fn Test_Check_Exported_Go_Functions_Use_Upper_Snake_Case_Should_Read_The_Method_Keys_For_A_Method()
{
    let payload = "unexpanded\t0\n\
                   item\t0\tFunction\tPublic\tRowCount\t.\t+fn/0\n\
                   item\t1\tFunction\tPublic\tTable::Row_Count\t.\t+fn/1\n";
    let rows = vec![Row(Scope::Repository, "method.exported", Case::UpperCamel)];

    let findings = Judged_Under_Policy(Check_Exported_Go_Functions_Use_Upper_Snake_Case, Path("table.go"), payload, rows);

    assert_eq!(Summaries(&findings), vec!["exported Go function `Row_Count` is not upper-camel case"], "{findings:?}");
}

// The tests below hold each casing rule's finding to `OD-RULES-011` version 3 decision 4: it states
// the case it was judged against, and nothing about where that case came from. Each judges one
// subject three times -- in a repository that declares nothing, in one that declares exactly the
// case the rule would otherwise judge against, and in one that declares another -- and requires the
// first two to report identical findings, so identical identities, and the third to name its own
// case. The exact text is asserted, so a summary that names a source again fails here.

#[test]
fn Test_Check_Naming_Convention_Should_State_The_Case_It_Judged_Against_And_Not_Where_It_Came_From()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Naming_Convention,
        path: Path("src/lib.rs"),
        payload: "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
        symbol: "function",
        undeclared: (Case::UpperSnake, "function `bad_name` is not upper-snake case"),
        declared: (Case::ScreamingSnake, "function `bad_name` is not screaming-snake case"),
    });
}

/// The same judgment under the code-standards id carries the same text.
#[test]
fn Test_Check_Project_Owned_Function_Names_Use_Upper_Snake_Case_Should_State_The_Case_It_Judged_Against()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
        path: Path("src/lib.rs"),
        payload: "unexpanded\t0\nitem\t0\tFunction\tPrivate\tbad_name\t.\t+fn/0\n",
        symbol: "function",
        undeclared: (Case::UpperSnake, "function `bad_name` is not upper-snake case"),
        declared: (Case::UpperCamel, "function `bad_name` is not upper-camel case"),
    });
}

#[test]
fn Test_Check_Module_And_Field_Names_Stay_Lower_Snake_Should_State_The_Case_It_Judged_Against()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Module_And_Field_Names_Stay_Lower_Snake,
        path: Path("src/lib.rs"),
        payload: "unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t+fields\\nBadField\\tString\n",
        symbol: "field",
        undeclared: (Case::LowerSnake, "`BadField` is a data name that is not lower-snake case"),
        declared: (Case::ScreamingSnake, "`BadField` is a data name that is not screaming-snake case"),
    });
}

#[test]
fn Test_Check_Exported_Go_Functions_Use_Upper_Snake_Case_Should_State_The_Case_It_Judged_Against()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Exported_Go_Functions_Use_Upper_Snake_Case,
        path: Path("main.go"),
        payload: "unexpanded\t0\nitem\t0\tFunction\tPublic\trun_With_Backend\t.\t+fn/0\n",
        symbol: "function.exported",
        undeclared: (Case::UpperSnake, "exported Go function `run_With_Backend` is not upper-snake case"),
        declared: (Case::LowerSnake, "exported Go function `run_With_Backend` is not lower-snake case"),
    });
}

#[test]
fn Test_Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter_Should_State_The_Case_It_Judged_Against()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
        path: Path("main.go"),
        payload: "unexpanded\t0\nitem\t0\tFunction\tPrivate\trowBreaches\t.\t+fn/0\n",
        symbol: "function.unexported",
        undeclared: (Case::MixedSnake, "unexported Go function `rowBreaches` is not mixed-snake case"),
        declared: (Case::LowerSnake, "unexported Go function `rowBreaches` is not lower-snake case"),
    });
}

/// The exported side. The old text named upper camel case by visibility alone, so a declared lower
/// snake case would have been reported as upper camel case, which is false of `OrderBook` and of
/// every other name upper camel case accepts.
#[test]
fn Test_Check_Go_Type_Names_Use_Camel_Case_Should_State_The_Case_It_Judged_Against()
{
    Assert_States_The_Case_It_Judged_Against(&CaseStatement {
        check: Check_Go_Type_Names_Use_Camel_Case,
        path: Path("types.go"),
        payload: "unexpanded\t0\nitem\t0\tStruct\tPublic\tOrder_Book\t.\t.\n",
        symbol: "type.exported",
        undeclared: (Case::UpperCamel, "Go type `Order_Book` is not upper-camel case"),
        declared: (Case::LowerSnake, "Go type `Order_Book` is not lower-snake case"),
    });
}

/// One casing rule and one subject it reports, judged in three repositories by
/// [`Assert_States_The_Case_It_Judged_Against`].
struct CaseStatement<'a>
{
    check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>,
    path: Path<'a>,
    payload: &'a str,
    /// The key the two declarations are written under, repository-wide.
    symbol: &'a str,
    /// The case the rule judges against when nothing is declared, and the one summary it reports.
    undeclared: (Case, &'a str),
    /// A case that differs from it, declared, and the one summary that reports.
    declared: (Case, &'a str),
}

/// Declaring nothing and declaring the undeclared case report identical findings, which therefore
/// hash to identical identities, with exactly the expected text; declaring another case reports
/// that case.
fn Assert_States_The_Case_It_Judged_Against(statement: &CaseStatement<'_>)
{
    let judged_under = |rows: Vec<PolicyRow>| return Judged_Under_Policy(statement.check, statement.path, statement.payload, rows);
    let (undeclared_case, undeclared_text) = statement.undeclared;
    let (declared_case, declared_text) = statement.declared;

    let undeclared = judged_under(Vec::new());
    let restated = judged_under(vec![Row(Scope::Repository, statement.symbol, undeclared_case)]);
    let declared = judged_under(vec![Row(Scope::Repository, statement.symbol, declared_case)]);

    assert_eq!(Summaries(&undeclared), vec![undeclared_text], "{undeclared:?}");
    assert_eq!(restated, undeclared, "declaring the case the rule already judged against must not change its finding");
    assert_eq!(Summaries(&declared), vec![declared_text], "{declared:?}");
}

fn Summaries(findings: &[Finding]) -> Vec<&str>
{
    return findings.iter().map(|finding| return finding.summary.as_str()).collect();
}

fn Row(scope: Scope, symbol: &str, case: Case) -> PolicyRow
{
    return PolicyRow { scope, symbol: symbol.to_owned(), case };
}

fn Judged_Names(findings: &[Finding]) -> Vec<&str>
{
    return findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
}

/// What [`Check_Naming_Convention`] finds in one source whose syntax fact is `payload`, in a
/// repository whose `standards.json` declares `rows`.
fn Findings_Under_Policy(path: Path<'_>, payload: &str, rows: Vec<PolicyRow>) -> Vec<Finding>
{
    return Judged_Under_Policy(Check_Naming_Convention, path, payload, rows);
}

/// What `check` finds in one source whose syntax fact is `payload`, in a repository whose
/// `standards.json` declares `rows` -- the naming policy fact filed beside the syntax fact, under
/// its own provider, the way a real run materializes both.
fn Judged_Under_Policy(check: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>, path: Path<'_>, payload: &str, rows: Vec<PolicyRow>) -> Vec<Finding>
{
    let source = Source_File(path, Text("// the payload is the fixture"));
    let TestOffering { mut store, mut registry, offer } = Offering();
    Materialize_Syntax_Fact(&mut store, &source, &offer, payload);
    let policy = ProviderOffer {
        provider: ProviderId::New("nomos.test.naming.policy"),
        capability: nomos_cap_naming_policy::Capability(),
        version: nomos_cap_naming_policy::CONTRACT_VERSION,
        guarantee: nomos_cap_naming_policy::Ceiling(),
    };
    registry
        .Declare_And_Offer(nomos_cap_naming_policy::Capability_Contract(), policy.clone())
        .expect("the registry holds only the syntax contract and its provider");
    let bytes = nomos_cap_naming_policy::Encode_Payload(&NamingPolicyPayload { rows });
    let fact = FactToFile { subject: nomos_model::Subject_Of_Path(""), offer: &policy, semantic_inputs: InputDigest::Of(&[]), schema: nomos_cap_naming_policy::Payload_Schema(), bytes };
    test_support::Materialize_Fact(&mut store, fact).expect("the fixture's store holds no fact under this key at a newer generation");

    let mut reader = Reader::On(&store, &registry, Test_Context());
    return check(&[source], &mut reader);
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
