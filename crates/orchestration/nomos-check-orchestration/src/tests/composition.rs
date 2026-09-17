//! The direct [`crate::Run`] seam: what the shipped composition answers over sources a test
//! wrote by hand, and the run table a caller selects through.

use nomos_analysis::{MemoryFactStore, Reader};
use nomos_contracts::{Applicability, Finding, GateCategory, RuleId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};

use crate::{CheckOutcome, Claim, Composed_Rules, Run, RunContext};

use super::{Repository_Root, Source_File, SourceText, Test_Variant};

/// How many architecture rules [`Architectural_Rules`] names.
const ARCHITECTURAL_RULE_COUNT: usize = 5;

/// Selected explicitly rather than left to `&[]`'s "everything" default: this repository
/// keeps its own architecture at zero findings (`COMPLETENESS_MIRROR`, `NAMING_CONVENTION`,
/// `DEPENDENCY_DIRECTION`, `DEPENDENCY_COMPLETENESS`, `UNREAD_REACHES_FINDING`), which is
/// what "a clean tree" means here, but does not keep `LINT_DIAGNOSTICS` at zero --
/// `cargo clippy`'s own pedantic-level warnings are tolerated debt this workspace's own gate
/// explicitly does not block on (`.github/workflows/gate.yml` carries no `-D warnings`), and
/// they fluctuate as concurrent sessions touch the tree. Selecting `&[]` here would make
/// this test's own pass/fail depend on the ambient lint-cleanliness of the whole real
/// repository at the moment it runs, which is not the property this test exists to prove.
fn Architectural_Rules() -> [RuleId; ARCHITECTURAL_RULE_COUNT]
{
    return [
        RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        RuleId::New(nomos_rules::NAMING_CONVENTION),
        RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
        RuleId::New(nomos_rules::DEPENDENCY_COMPLETENESS),
        RuleId::New(nomos_rules::UNREAD_REACHES_FINDING),
    ];
}

#[test]
fn Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings()
{
    let sources = vec![Source_File("a.rs", SourceText("pub fn Ok() {}\n"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &Architectural_Rules());

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::Examined { files: 1, facts: 1 });
    assert_eq!(claim, Claim::Complete);
}

/// `OD-CAPABILITY-009`'s corrected fix, proven end to end: a clean `.go` source is judged
/// under `nomos_lang_go`'s own identity, the same way the `.rs` test just above is judged
/// under `nomos_lang_rust`'s -- and both stay green together, which is what proves the fix
/// does not reintroduce the regression this record's own body describes (`.rs` facts
/// resolving against `nomos_lang_go`'s stronger, unpreferenced-ranked offer instead of the
/// provider that actually wrote them).
///
/// Selects only `COMPLETENESS_MIRROR` and `NAMING_CONVENTION` -- the two rules
/// `nomos.cap.syntax.items` actually drives, and the whole of what this item fixes.
/// `UNREAD_REACHES_FINDING` has no Go provider registered for `nomos.cap.controlflow.
/// reachability` at all (`Declare_Controlflow_Capability` offers only `nomos_lang_rust`'s),
/// so selecting it here would report an honest `DependencyUnavailable` for a capability
/// this item was never asked to extend to Go, not a regression in the one it was.
#[test]
fn Test_A_Clean_Go_Source_Should_Be_Judged_Through_Its_Own_Real_Provider()
{
    let sources = vec![Source_File("main.go", SourceText("package main\n\nfunc One() {}\n"))];
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR), RuleId::New(nomos_rules::NAMING_CONVENTION)];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &selected);

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::Examined { files: 1, facts: 1 });
    assert_eq!(claim, Claim::Complete);
}

/// A Rust half and a Go half declared to correspond, judged together -- the whole subject of
/// the two tests below.
struct Correspondence
{
    rust: SourceFile,
    go: SourceFile,
}

/// The findings `Run` answers for the pair `Correspondence` names, judged under the
/// cross-language correspondence rule.
fn Corresponded_Findings(correspondence: Correspondence) -> Vec<Finding>
{
    let sources = vec![correspondence.rust, correspondence.go];
    let selected = [RuleId::New(nomos_rules::CROSS_LANGUAGE_CORRESPONDENCE)];
    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &selected);

    let CheckOutcome::Judged { findings, .. } = outcome else { panic!("a tree the provider can read must be judged") };

    return findings;
}

/// `OD-CAPABILITY-010`'s own end-to-end proof: a real Rust source and a real Go source,
/// judged through `Run` together, over a declared correspondence whose two sides
/// genuinely disagree on fields -- not a synthetic fact built by hand, the real syntax
/// providers reading real source text.
#[test]
fn Test_A_Genuine_Cross_Language_Field_Mismatch_Should_Be_Reported()
{
    let findings = Corresponded_Findings(Correspondence {
        rust: Source_File("wide.rs", SourceText("/// Corresponds to `Wide`.\npub struct Wide { pub A: u32, pub B: u32 }\n")),
        go: Source_File("wide.go", SourceText("package main\n\ntype Wide struct {\n\tA int\n}\n")),
    });

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.applicability, Applicability::Supported);
    assert!(found.summary.contains('B'), "{}", found.summary);
}

/// The identical pair, this time agreeing on fields -- proving a real match is silent
/// rather than merely proving a real mismatch is loud.
#[test]
fn Test_A_Genuine_Cross_Language_Field_Match_Should_Report_Nothing()
{
    let findings = Corresponded_Findings(Correspondence {
        rust: Source_File("clean.rs", SourceText("/// Corresponds to `Clean`.\npub struct Clean { pub A: u32 }\n")),
        go: Source_File("clean.go", SourceText("package main\n\ntype Clean struct {\n\tA int\n}\n")),
    });

    assert!(findings.is_empty(), "{findings:?}");
}

/// A finding that can fail a build and a finding the run could not resolve a judgment
/// about are different axes -- `OD-COMPLETENESS-004`'s whole point, restated here because
/// this crate is now where both axes are actually decided. A phantom mirror is
/// `Applicability::Supported` (the rule read it and judged it false) and `GateCategory::
/// Blocking`, so it must move `nomos-cli`'s exit code and must NOT flip [`Claim`] --
/// [`Claim`] answers "did the run reach a verdict", not "did the verdict pass".
#[test]
fn Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete()
{
    let sources = vec![Source_File(
        "a.rs",
        SourceText("/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n"),
    )];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    let CheckOutcome::Judged { findings, claim, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(!findings.is_empty(), "the phantom mirror must be reported");
    assert!(
        findings.iter().any(|finding| return finding.gate == GateCategory::Blocking),
        "{findings:?}"
    );
    assert_eq!(claim, Claim::Complete, "a blocking verdict is still a reached verdict");
}

/// A run that materializes no fact for anything it read must not be reported as judged --
/// an empty findings list here would render identically to a clean tree, which is exactly
/// the lie `nomos-cli::check`'s own vacuity guard exists to catch, one layer in.
#[test]
fn Test_A_Run_That_Materializes_No_Facts_Should_Report_NoFacts()
{
    let sources = vec![Source_File("broken.rs", SourceText("pub const ??? = ;"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    assert!(
        matches!(outcome, CheckOutcome::NoFacts { files: 1 }),
        "a provider refusing every file must not be judged"
    );
}

/// Two sources that normalize to the same workspace member cannot both be ingested. Not a
/// case a real walk produces -- `nomos-cli::check::sources::Read_Sources` reads a real
/// directory, which cannot hand back two entries for one path -- but this crate accepts
/// `sources` from any caller, and a second adapter's own walk is not this crate's to trust.
#[test]
fn Test_Conflicting_Paths_Should_Be_Unreadable()
{
    let sources = vec![Source_File("a.rs", SourceText("pub fn one() {}\n")), Source_File("a.rs", SourceText("pub fn two() {}\n"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    assert!(matches!(outcome, CheckOutcome::Unreadable), "duplicate paths must not be ingested");
}

/// The floor is the rule's, and the composition this crate assembles meets it -- the same
/// control `nomos-cli::check`'s own suite used to keep, restated here because this crate is
/// now where the composition actually lives.
#[test]
fn Test_The_Registered_Provider_Should_Satisfy_The_Rules_Floor()
{
    let sources = vec![Source_File("a.rs", SourceText("pub const TABLE: &[&str] = &[];\n"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[]);

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(
        findings
            .iter()
            .all(|finding| return finding.applicability == Applicability::Supported),
        "the registered parser must serve nomos_rules::Syntax_Requirement: {findings:?}"
    );
    assert!(
        findings.iter().all(|finding| return finding.gate == GateCategory::Advisory),
        "{findings:?}"
    );
}

/// ---- the shipped composition consults a fact ----
///
/// The assertion `P10-FACT-BYPASS` turns on, and the reason this crate's own test suite
/// reaches past [`Run`] into its private [`crate::composition`] and [`crate::facts`]
/// pieces: the guarantee is that the rule's verdict depends on what the *store* holds, not
/// merely on which sources the rule was handed, and demonstrating that needs a store built
/// from one source list judged against a different, larger one. `Run` deliberately does not
/// expose that split -- a second adapter has no reason to ingest less than it judges -- so
/// this is where the split-composition guarantee is proven instead: internally, once, by
/// the crate that owns it.
#[test]
fn Test_The_Composed_Crate_Should_Resolve_A_Mirror_Through_A_Real_Fact()
{
    let mirror = Mirror_Fixture();
    let whole = vec![mirror.declaring.clone(), mirror.checking];

    let resolved = Findings_Over(&whole, &whole);
    // The store is told about the declaring file only; the rule is handed both.
    let short = Findings_Over(&[mirror.declaring], &whole);

    assert!(
        resolved.is_empty(),
        "the registered parser must find the check in b.rs: {resolved:?}"
    );
    assert!(
        short.iter().any(|finding| return finding.subject_name == "T"),
        "with b.rs's fact withheld the claim must not resolve: {short:?}"
    );
}

/// The declaring and checking half of the phantom mirror [`Test_The_Composed_Crate_Should_
/// Resolve_A_Mirror_Through_A_Real_Fact`] withholds one fact from.
struct Mirror
{
    declaring: SourceFile,
    checking: SourceFile,
}

fn Mirror_Fixture() -> Mirror
{
    let declaring = Source_File("a.rs", SourceText("/// Mirrored by `Test_The_Real_Provider_Found_This`.\npub const T: &[&str] = &[];\n"));
    let checking = Source_File("b.rs", SourceText("#[cfg(test)]\nmod tests\n{\n    #[test]\n    fn Test_The_Real_Provider_Found_This()\n    {\n    }\n}\n"));

    return Mirror { declaring, checking };
}

/// Ingests `ingested` into a real fact store and judges `judged` over it -- the split
/// [`crate::run_context::Run`] does not offer, assembled here from the crate's own private pieces.
///
/// `judged` is run through [`crate::run_context::Recognized_Sources`] before the rule ever sees it, the
/// identical enrichment `Run` gives every real caller: `Check_Completeness_Mirrors` narrows
/// its own `Require` call by `SourceFile::preferred_syntax_provider`, and a fixture built by
/// hand needs that field populated the same way a real walk's sources would be, or `.rs`
/// resolves against `nomos_lang_go`'s stronger, unpreferenced-ranked offer instead of the
/// provider that actually wrote the fact.
fn Findings_Over(ingested: &[SourceFile], judged: &[SourceFile]) -> Vec<Finding>
{
    let registry = crate::composition::Registered().expect("the fixture composition is this crate's own");
    let context = crate::facts::Ingested_Workspace(ingested, &registry, Test_Variant(), &mut None).expect("the fixture is a valid tree");
    let mut store = MemoryFactStore::New();
    let _written = crate::facts::Materialize_Syntax(ingested, &context, &mut store);

    let judged = crate::run_context::Recognized_Sources(judged);
    let mut reader = Reader::On(&store, &registry, context);
    return Check_Completeness_Mirrors(&judged, &mut reader);
}

/// Every rule [`Composed_Rules`] names must be named once.
///
/// A rule composed twice into the run table is invisible from inside this crate -- the
/// second entry simply runs the same check again -- but
/// `nomos-gate-orchestration::composition::Registered` derives its offers from this export
/// and refuses a repeated [`RuleId`] with `RuleRegistryError::AlreadyOffered`, so a
/// duplicate here turns a harmless double-run into a whole-registry refusal one crate away.
/// This is the assertion that catches it at the source rather than at the consumer.
#[test]
fn Test_Composed_Rules_Should_Name_Each_Rule_Once()
{
    let composed = Composed_Rules();

    let mut seen = std::collections::HashSet::new();
    let repeated: Vec<&RuleId> = composed.iter().filter(|rule| return !seen.insert(*rule)).collect();

    assert!(repeated.is_empty(), "the run table composes these rules more than once: {repeated:?}");
}

/// The exported identifiers are the run table's own selection keys, proved by selecting
/// through one.
///
/// This is the assertion a length or uniqueness check cannot make. A second, hand-typed
/// list would satisfy both while carrying a lookalike string -- `no_trailing_whitespace`
/// against `no-trailing-whitespace`, or an identifier left behind by a rename -- and every
/// caller that selected through the export would then silently run nothing where it asked
/// for a rule. So the rule below is selected by the string [`Composed_Rules`] handed back,
/// never by the constant, and the finding that comes back has to carry it.
#[test]
fn Test_A_Rule_Selected_Through_Its_Exported_Identifier_Should_Run()
{
    let exported = Composed_Rules()
        .into_iter()
        .find(|rule| return rule.As_Str() == nomos_rules::NO_TRAILING_WHITESPACE)
        .expect("the export must name a rule the run table composes");

    let sources = vec![Source_File("a.rs", SourceText("pub fn one() {} \n"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[exported.clone()]);

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(
        !findings.is_empty(),
        "selecting through the exported identifier must run the rule it names"
    );
    assert!(
        findings.iter().all(|finding| return finding.rule == exported),
        "only the selected rule may report: {findings:?}"
    );
}

/// The negative control for the test above.
///
/// Without it, a selection seam that ignored `selected` entirely and ran everything would
/// satisfy that assertion by accident -- the trailing-whitespace finding would still arrive
/// -- and the export could name anything at all.
#[test]
fn Test_An_Identifier_The_Export_Does_Not_Name_Should_Select_Nothing()
{
    let composed = Composed_Rules();
    let absent = RuleId::New("no-rule-by-this-name");
    assert!(!composed.contains(&absent), "this control needs an identifier the table does not carry");

    let sources = vec![Source_File("a.rs", SourceText("pub fn one() {} \n"))];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, &[absent]);

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(
        findings.is_empty(),
        "an unselected run table must report nothing: {findings:?}"
    );
}
