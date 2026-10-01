//! Public type declarations should live in the file named for that type.
//!
//! code-standards' `file-name-matches-declared-type` rule is cross-language and includes
//! generated/partial companion-file behavior. The current syntax payload exposes declared
//! items and visibility, so this rule judges public type-like declarations whose file stem
//! is meaningful to compare, while skipping structural Rust entrypoints such as `lib.rs`
//! and `main.rs`.
//!
//! `one-public-type-per-file` is the sibling rule for the same home: the public-surface
//! form counts top-level public type-like declarations. The C# internal/file-scoped split
//! is outside this crate's current syntax inputs, but the Rust/Go/public-surface rule is
//! exact over the payload.
//!
//! Both rules skip a test or example source, through the same
//! [`crate::checks::Is_Test_Or_Example_Source`] every other file-path exemption in this
//! crate reads. A fixture's filename is part of what the fixture fixes: `tests/corpus/
//! analysis/alpha/one.rs` declares `Anchor` and is named `one.rs` because the analysis
//! tests that read it care about ordering, not about naming, and renaming it to
//! `anchor.rs` would change what those tests measure in order to satisfy a rule they are
//! not subject to. `Comparable_Stem`'s `lib`/`main`/`mod` skip is the same kind of
//! judgment one level down — a name that carries no claim about a type — and this is the
//! same answer for a whole file whose name carries no such claim either.

use crate::SourceFile;
use crate::checks::finding_shape::{Finding_Shape, Own_Name_Finding};
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, PayloadItem, SyntaxPayload};
use nomos_contracts::{Finding, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const FILE_NAME_MATCHES_DECLARED_TYPE: &str = "file-name-matches-declared-type";
/// This rule's own identifier, matching the code-standards rule id.
pub const ONE_PUBLIC_TYPE_PER_FILE: &str = "one-public-type-per-file";

const ENUM: &str = "Enum";
const INTERFACE: &str = "Interface";
const STRUCT: &str = "Struct";
const TRAIT: &str = "Trait";
const TRAIT_ALIAS: &str = "TraitAlias";
const TYPE_ALIAS: &str = "TypeAlias";
const TYPE_DEFINITION: &str = "TypeDefinition";
const UNION: &str = "Union";

/// Reports public type-like declarations whose file stem does not match the declared type.
#[must_use]
pub fn Check_File_Name_Matches_Declared_Type(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Judged_Sources(sources, facts, Violations_In, Unread_As_This_Rule);
}

/// Reports files with more than one top-level public type-like declaration.
#[must_use]
pub fn Check_One_Public_Type_Per_File(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return Judged_Sources(sources, facts, One_Public_Type_Violations_In, Unread_As_One_Public_Type_Rule);
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let Some(stem) = Comparable_Stem(path)
    else
    {
        return Vec::new();
    };

    if Is_Declaring_A_Public_Operation(payload)
    {
        return Vec::new();
    }

    let public_types: Vec<&PayloadItem> =
        payload.items.iter().filter(|item| return item.Is_Public() && Is_Type_Like(item)).collect();

    if public_types.iter().any(|item| return To_Snake_Case(item.Own_Name()) == stem)
    {
        return Vec::new();
    }

    return public_types.into_iter().map(|item| return Violation_Finding(path, item, &stem)).collect();
}

fn Comparable_Stem(path: &str) -> Option<String>
{
    let normalized = path.replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or(&normalized);
    let stem = file_name.split('.').next().unwrap_or(file_name);

    if matches!(stem, "lib" | "main" | "mod")
    {
        return None;
    }

    return Some(stem.to_owned());
}

// A file naming one of the types it declares satisfies this rule for all of them, which
// `OD-RULES-016` decides and this is the whole of. `budget_estimate.rs` declares
// `BudgetEstimate` and the `CheckOrFixStage` enum that is one of its fields; the file is
// named for a declared type, which is what the rule id says, and reporting the companion
// would mean splitting a cohesive module in two. Doing that everywhere is
// `one-public-type-per-file` -- the sibling rule below, deliberately composed into nothing.
// A workspace that declined to require one public type per file did not mean this rule to
// require it transitively.

/// Whether a module's public surface includes a free function, and so is named for
/// something this rule has no claim about.
///
/// A module holding types alone is named for a type, and `file-name-matches-declared-type`
/// says which one. A module that also exports a free function is named for what it does,
/// and its types are subordinate to that: `nomos-repo-policy/src/goals/reading.rs` exports
/// `Discover_Workspace` and the error `Discover_Workspace` returns, so renaming it
/// `goals_policy_error.rs` would name it after the least important thing in it. `OD-RULES-015`
/// records the decision and the measurement behind it -- across the 46 findings this rule
/// raised on this workspace, the 13 files declaring a free public function were every false
/// positive and the 23 declaring none were every true one, with no file on the wrong side.
///
/// This is the judgment `Comparable_Stem` already makes one level up when it skips `lib`,
/// `main` and `mod`: a name that carries no claim about a type is not a name this rule can
/// check. A method does not count, because a method is named inside the type it belongs to
/// and says nothing about what the module is for -- which is why this asks for a *free*
/// function, using the same `Is_Top_Level` the sibling rule below reads.
fn Is_Declaring_A_Public_Operation(payload: &SyntaxPayload) -> bool
{
    return payload
        .items
        .iter()
        .any(|item| return item.Is_Public() && item.kind == FUNCTION && Is_Top_Level(item));
}

/// One public type whose file stem is not its own snake-case name.
fn Violation_Finding(path: &str, item: &PayloadItem, stem: &str) -> Finding
{
    let type_name = item.Own_Name();
    let expected = To_Snake_Case(type_name);

    return Own_Name_Finding(
        Finding_Shape { rule: FILE_NAME_MATCHES_DECLARED_TYPE, path, summary: format!("`{type_name}` is public but {path} has stem `{stem}` instead of `{expected}`") },
        item,
    );
}

fn One_Public_Type_Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let public_file_home_types: Vec<&PayloadItem> = payload
        .items
        .iter()
        .filter(|item| return item.Is_Public() && Is_Type_Like(item) && Is_Top_Level(item))
        .collect();

    if public_file_home_types.len() <= 1
    {
        return Vec::new();
    }

    return public_file_home_types
        .into_iter()
        .map(|item| return One_Public_Type_Finding(path, item))
        .collect();
}

/// The second and later public types in one file.
fn One_Public_Type_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let type_name = item.Own_Name();

    return Own_Name_Finding(
        Finding_Shape { rule: ONE_PUBLIC_TYPE_PER_FILE, path, summary: format!("{path} declares more than one top-level public type; `{type_name}` needs its own file") },
        item,
    );
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(FILE_NAME_MATCHES_DECLARED_TYPE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's declared public types could not be judged");
    return finding;
}

fn Unread_As_One_Public_Type_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(ONE_PUBLIC_TYPE_PER_FILE);
    finding.summary = finding.summary.replace(
        "this file's naming could not be judged",
        "this file's public type count could not be judged",
    );
    return finding;
}

/// The shape both checks above share: skip a test or example source, read each remaining
/// source's own syntax fact, and fold either a real reading failure or `judge`'s own
/// findings into one sorted list. `judge` and `unread` are each rule's own way of turning a
/// decoded payload, or an unread source, into that rule's `Finding`s.
fn Judged_Sources(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    judge: fn(&SyntaxPayload, &str) -> Vec<Finding>,
    unread: fn(Finding) -> Finding,
) -> Vec<Finding>
{
    let declared = crate::checks::Resolve_Declared_Fixture_Locations(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        if crate::checks::Is_Test_Or_Example_Source(source, &declared)
        {
            continue;
        }

        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = judge(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(unread(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Is_Type_Like(item: &PayloadItem) -> bool
{
    return matches!(
        item.kind.as_str(),
        ENUM | INTERFACE | STRUCT | TRAIT | TRAIT_ALIAS | TYPE_ALIAS | TYPE_DEFINITION | UNION
    );
}

fn Is_Top_Level(item: &PayloadItem) -> bool
{
    return !item.qualified_name.contains("::");
}

fn To_Snake_Case(name: &str) -> String
{
    let mut snake = String::new();
    let mut previous_was_lower_or_digit = false;

    for character in name.chars()
    {
        if character.is_ascii_uppercase()
        {
            if previous_was_lower_or_digit
            {
                snake.push('_');
            }
            snake.push(character.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
            continue;
        }

        previous_was_lower_or_digit = character.is_ascii_lowercase() || character.is_ascii_digit();
        snake.push(character);
    }

    return snake;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::{self, FactToFile, OfferedProvider, Test_Context, TestOffering};
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    /// A fixture whose filename is part of what it fixes is not judged by either rule.
    ///
    /// Reached through the public entry point rather than through `Violations_In`, because
    /// the exemption is a source filter and `Violations_In` never sees the path it skipped:
    /// a test calling the inner function would pass with the filter deleted, which is the
    /// one thing this must not do. `tests/corpus/analysis/alpha/one.rs` is the real path
    /// this was written for — it declares `Anchor` and the analysis suites that read it
    /// depend on the name `one`.
    #[test]
    fn Test_Check_File_Name_Matches_Declared_Type_Should_Not_Judge_A_Test_Or_Example_Source()
    {
        let path = "tests/corpus/analysis/alpha/one.rs";
        let source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), "pub struct Anchor;\n");
        let TestOffering { store, registry, .. } = Offering_Over_A_Corpus_Fixture(&source);

        let mut reader = nomos_analysis::Reader::On(&store, &registry, Test_Context());
        let findings = Check_File_Name_Matches_Declared_Type(&[source], &mut reader);

        assert!(findings.is_empty(), "a corpus fixture must not be judged on its stem: {findings:?}");
    }

    /// A declared-and-offered syntax capability with the `Anchor` item `source` declares
    /// materialized against it — the reader the corpus-fixture test above is built over, so
    /// that fixture really would be judged if the exemption were deleted.
    fn Offering_Over_A_Corpus_Fixture(source: &SourceFile) -> TestOffering
    {
        use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

        const PARSER: &str = "nomos.test.file.names.parses";
        const ITEMS: &str = "unexpanded\t0\nitem\t0\tStruct\tPublic\tAnchor\t.\t.\n";
        let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
        let TestOffering { mut store, registry, offer } = test_support::Offered_Registry(
            OfferedProvider {
                contract: nomos_cap_syntax::Capability_Contract(),
                capability: nomos_cap_syntax::Capability(),
                version: nomos_cap_syntax::CONTRACT_VERSION,
                provider: PARSER,
                guarantee,
            },
        ).expect("a fresh Registry holds neither this contract nor this provider");
        test_support::Materialize_Fact(
            &mut store,
            FactToFile {
                subject: source.subject,
                offer: &offer,
                semantic_inputs: nomos_analysis::InputDigest::Of(&[source.text.as_bytes()]),
                schema: nomos_cap_syntax::Payload_Schema(),
                bytes: ITEMS.as_bytes().to_vec(),
            },
        ).expect("the fixture's store holds no fact under this key at a newer generation");
        return TestOffering { store, registry, offer };
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Public_Type_Whose_File_Stem_Does_Not_Match()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/orders.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_NAME_MATCHES_DECLARED_TYPE));
        assert_eq!(found.subject_name, "OrderBook");
    }

    /// The family layout: a module exporting an operation and the error that operation
    /// returns. `OD-RULES-015` decides this is not this rule's to judge, because the
    /// module is named for `Discover_Workspace` and the error is subordinate to it.
    /// A module naming one of the types it declares is satisfied for all of them.
    /// `budget_estimate.rs` declares `BudgetEstimate` and the `CheckOrFixStage` that is one
    /// of its fields, and `OD-RULES-016` decides the companion is not this rule's to report.
    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Companion_Type_Beside_A_Matching_One()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tBudgetEstimate\t.\t.\nitem\t1\tEnum\tPublic\tCheckOrFixStage\t.\t.\n");

        let findings = Violations_In(&payload, "src/budget_estimate.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The other side: a stem naming none of the types it declares is still reported, and
    /// reported for every one of them, because there is no way to tell which the file meant.
    #[test]
    fn Test_Violations_In_Should_Judge_Every_Type_When_The_Stem_Names_None_Of_Them()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tEffortMappingRecord\t.\t.\nitem\t1\tEnum\tPublic\tMappingQuality\t.\t.\n");

        let findings = Violations_In(&payload, "src/effort_mapping.rs");

        const EXPECTED_UNMATCHED_TYPE_COUNT: usize = 2;
        assert_eq!(findings.len(), EXPECTED_UNMATCHED_TYPE_COUNT, "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Module_That_Exports_A_Free_Function()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tGoalsPolicyError\t.\t.\nitem\t1\tFunction\tPublic\tDiscover_Workspace\t.\t.\n");

        let findings = Violations_In(&payload, "src/reading.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The other side of the same decision, and the one that keeps it narrow: a method is
    /// named inside its own type and says nothing about what the module is for, so a
    /// module whose only public functions are methods is still a module of types.
    #[test]
    fn Test_Violations_In_Should_Still_Judge_A_Module_Whose_Only_Functions_Are_Methods()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\nitem\t1\tFunction\tPublic\tOrderBook::New\t.\t.\n");

        let findings = Violations_In(&payload, "src/orders.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "OrderBook");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_A_Public_Type_Whose_File_Stem_Matches()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/order_book.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Private_Types()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPrivate\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/orders.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Compare_Companion_Files_By_The_First_Stem()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/order_book.generated.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Structural_Entrypoints()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tOrderBook\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Report_Each_Public_Type_When_Two_Are_Top_Level()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tEnum\tPublic\tOrderKind\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        const EXPECTED_TOP_LEVEL_PUBLIC_TYPE_COUNT: usize = 2;
        assert_eq!(findings.len(), EXPECTED_TOP_LEVEL_PUBLIC_TYPE_COUNT, "{findings:?}");
        assert!(findings.iter().all(|finding| return finding.rule == RuleId::New(ONE_PUBLIC_TYPE_PER_FILE)));
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Ignore_A_Private_Second_Type()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tEnum\tPrivate\tOrderKind\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_One_Public_Type_Violations_In_Should_Ignore_Nested_Public_Types()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tOrder\t.\t.\n\
             item\t1\tStruct\tPublic\tOrder::Builder\t.\t.\n",
        );

        let findings = One_Public_Type_Violations_In(&payload, "src/order.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
