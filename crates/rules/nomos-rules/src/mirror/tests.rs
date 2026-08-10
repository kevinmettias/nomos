//! Every judgment this rule can reach, exercised against a real source tree.

use super::*;
use nomos_analysis::{
    Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{ProviderOffer, Registry};
use nomos_contracts::{
    Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId, Guarantee,
    IncrementalGranularity, ProviderId, SchemaId, SnapshotId,
};

/// A stand-in for a parser, at exactly the guarantee `nomos-lang-rust` declares.
///
/// Named rather than imported. `nomos-rules` must not depend on a provider even in
/// test code — that is the edge the registry exists to remove, and a rule crate that
/// names one has answered the question the floor is supposed to ask. The end-to-end
/// path with the real provider belongs in the composition root, and `nomos-cli`'s
/// tests are where it is asserted.
const PARSER: &str = "nomos.test.parses";

/// A stand-in for a line scanner, at exactly the guarantee `nomos-lang-rust-scan`
/// declares. Used only to show that this rule's floor refuses it.
const SCANNER: &str = "nomos.test.scans";

fn Parser_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

fn Scanner_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

fn Offer(provider: &str, guarantee: Guarantee) -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(provider),
        capability: nomos_cap_syntax::Capability(),
        version: nomos_cap_syntax::CONTRACT_VERSION,
        guarantee,
    };
}

/// A source file, with the subject a composition root would file its facts under.
fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(
        path,
        SubjectId::From_Digest(Content_Digest(path.as_bytes())),
        text,
    );
}

/// The payload a parser would write for a file declaring these functions.
///
/// Hand-written against the `nomos.syntax.items.v1` encoding rather than produced by
/// calling a provider, for the reason `PARSER` gives. Every name is emitted qualified
/// by a `tests` module, because that is the shape every check in this workspace really
/// has and a reader that only handled bare names would pass here and resolve nothing
/// in the product.
use core::fmt::Write as _;

/// `None` where the provider would have refused the file, so no fact is filed for it —
/// which is what a real run does with a file that does not parse, and why the rule
/// reports it unread rather than clean.
fn Payload(source: &SourceFile, declared: &[&str]) -> Option<Vec<u8>>
{
    let nomos_lang_rust::Reading::Parsed(facts) = nomos_lang_rust::Read_Source(&source.text)
    else
    {
        return None;
    };

    let encoded = nomos_lang_rust::Encode_Payload(&facts);
    let mut text = String::from_utf8(encoded).expect("the encoding is UTF-8");

    for (offset, name) in declared.iter().enumerate()
    {
        let ordinal = facts.items.len().saturating_add(offset);
        let _ = writeln!(text, "item\t{ordinal}\tFunction\tPrivate\ttests::{name}\t.\t+fn/0");
    }

    return Some(text.into_bytes());
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

/// The parts of a composition a rule reads through: a registry and a store.
///
/// Assembled here rather than imported from anywhere, which is the property `D-134`
/// was built on and this record claims is unaffected — a test builds the whole subject
/// out of files that no longer exist, and now out of facts about them too.
struct World
{
    store: MemoryFactStore,
    registry: Registry,
}

impl World
{
    /// A registry holding the syntax contract and whichever offers are named.
    fn Offering(providers: &[(&str, Guarantee)]) -> Self
    {
        let mut registry = Registry::New();
        registry
            .Declare(nomos_cap_syntax::Capability_Contract())
            .expect("the syntax capability is declared once");

        for (provider, guarantee) in providers
        {
            let offer = Offer(provider, *guarantee);
            registry
                .Offer(offer)
                .expect("every offer here is within the capability's ceiling");
        }

        return Self {
            store: MemoryFactStore::New(),
            registry,
        };
    }

    fn Context() -> Context
    {
        return Context {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
            generation: GenerationId::INITIAL,
        };
    }

    /// The key a named offer's answer about one source would be filed under.
    ///
    /// Assembled from the offer rather than from the provider's name alone, because
    /// `Reader::Require` derives the same key from whichever offer the registry chose.
    /// A test that filed a fact under a key the reader does not compute would assert
    /// that the rule reports every subject unread, which is a green test and no
    /// coverage at all.
    fn Key(source: &SourceFile, offer: &ProviderOffer) -> FactKey
    {
        let context = Self::Context();

        return FactKey {
            contract: nomos_cap_syntax::Capability(),
            contract_version: offer.version,
            subject: source.subject,
            semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
            provider: offer.provider.clone(),
            provider_version: offer.version,
            guarantee: GuaranteeDigest::Of(&offer.guarantee),
            variant: context.variant,
            configuration: context.configuration,
        };
    }

    /// Files a fact about one source, exactly as a provider registered here would.
    fn Materializing(mut self, source: &SourceFile, schema: &str, payload: Vec<u8>) -> Self
    {
        let context = Self::Context();
        let offer = Offer(PARSER, Parser_Guarantee());

        self.store
            .Materialize(
                MaterializedFact {
                    identity: Self::Key(source, &offer).At(context.generation),
                    snapshot: context.snapshot,
                    evidence: EvidenceClass::Verified,
                    guarantee: offer.guarantee,
                    payload: FactPayload::New(SchemaId::New(schema), payload),
                },
                &[],
            )
            .expect("nothing here is backdated");

        return self;
    }

    fn Reader(&self) -> Reader<'_, '_>
    {
        return Reader::On(&self.store, &self.registry, Self::Context());
    }
}

/// A parser is registered, and it answered for every source named here.
///
/// The fixture's payload is what the real provider would have written for it, plus
/// whatever check names the test states on top. Both halves are deliberate. Deriving
/// the items means a fixture's universes reach the rule the way they do in the product
/// — this crate no longer parses, so a hand-written payload would be a shape nobody
/// could have produced, and it drifted the moment the schema gained a field. Stating
/// the checks separately keeps the tests that matter honest: a `Test_X` written inside
/// a fixture string is *not* in the parser's output, and a test that wants one resolved
/// has to say so rather than smuggling it through the text.
/// The findings over two sources, only the first of which has a fact.
///
/// The shape every shortfall test needs, because it is the shape of the defect the
/// record was opened against: one file the rule could read and one it could not, and a
/// judgment about the first that must not inherit doubt from the second.
fn Judged_Beside(declaring: &SourceFile, unread: &SourceFile) -> Vec<Finding>
{
    let sources = vec![declaring.clone(), unread.clone()];
    let world = World_Over(&[(declaring, &[])]);
    let mut reader = world.Reader();

    return Check_Completeness_Mirrors(&sources, &mut reader);
}

/// The finding about one named universe, if the run produced one.
fn Named<'a>(subject: &str, findings: &'a [Finding]) -> Option<&'a Finding>
{
    return findings
        .iter()
        .find(|finding| return finding.subject_name == subject);
}

fn World_Over(declaring: &[(&SourceFile, &[&str])]) -> World
{
    let mut world = World::Offering(&[(PARSER, Parser_Guarantee())]);

    for (source, declared) in declaring
    {
        if let Some(payload) = Payload(source, declared)
        {
            world = world.Materializing(source, nomos_cap_syntax::SCHEMA, payload);
        }
    }

    return world;
}

/// Findings over sources whose facts declare no checks at all.
///
/// The common case for a universe test: the tree is read whole, and nothing in it
/// mirrors anything.
fn Findings_Over(sources: &[SourceFile]) -> Vec<Finding>
{
    let declaring: Vec<(&SourceFile, &[&str])> = sources
        .iter()
        .map(|source| return (source, &[] as &[&str]))
        .collect();
    let world = World_Over(&declaring);
    let mut reader = world.Reader();

    return Check_Completeness_Mirrors(sources, &mut reader);
}

fn Only(findings: &[Finding]) -> &Finding
{
    assert_eq!(findings.len(), 1, "expected exactly one finding: {findings:?}");
    return findings.first().expect("just asserted the length is one");
}

/// ---- the three instances `OD-COMPLETENESS-001` analyses ----
///
/// None of the three can be replayed from git; each was repaired at the site. They
/// are reproduced here as they were originally written, which is the whole reason
/// this rule takes its subject as an argument rather than reading the tree. That the
/// subject is now a pair — text and a reader — is the amendment `OD-RULES-001` makes
/// to `D-134`, and these tests are the property it claims is unaffected. If they could
/// not be expressed against a reader, the amendment would be wrong.
///
/// Instance one and two: `Table::All`, a list kept beside the enum, over which
/// `Assert_Complete` and `Assert_Landed` both quantified. A migration adding a table
/// without adding it here left that table outside both guards.
#[test]
fn Test_The_Table_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
{
    let findings = Findings_Over(&[Source(
        "crates/spec/nomos-spec-store/src/store.rs",
        "impl Table\n\
         {\n\
         \x20   /// Every table in the schema.\n\
         \x20   pub const fn All() -> &'static [Self]\n\
         \x20   {\n\
         \x20   }\n\
         }\n",
    )]);

    let finding = Only(&findings);

    assert_eq!(finding.subject_name, "Table::All");
    assert_eq!(finding.rule.As_Str(), COMPLETENESS_MIRROR);
    assert!(finding.summary.contains("declares no mirror"), "{}", finding.summary);
}

/// Instance three: `GOVERNING_RECORD_IDS`, compared against a store seeded from
/// `GOVERNING_RECORD_IDS` — a comparison that cannot fail. Six governing records sat
/// outside it for months.
#[test]
fn Test_The_Governing_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
{
    let findings = Findings_Over(&[Source(
        "crates/spec/nomos-spec-store/src/governing.rs",
        "/// The records this store seeds itself with.\n\
         pub const GOVERNING_RECORD_IDS: &[&str] = &[\n\
         \x20   \"ARC-SPECDB-001\",\n\
         ];\n",
    )]);

    assert_eq!(Only(&findings).subject_name, "GOVERNING_RECORD_IDS");
}

/// And all three together, in one run, because `P10-FIRST-CHECK` asks for the rule
/// to fail on all three rather than on each in isolation.
#[test]
fn Test_All_Three_Historical_Instances_Should_Fail_This_Rule()
{
    let findings = Findings_Over(&[
        Source(
            "crates/spec/nomos-spec-store/src/store.rs",
            "impl Table\n{\n    pub const fn All() -> &'static [Self]\n    {\n    }\n}\n",
        ),
        Source(
            "crates/spec/nomos-spec-store/src/governing.rs",
            "pub const GOVERNING_RECORD_IDS: &[&str] = &[];\n",
        ),
        Source("tests/contract/src/gates.rs", "pub const CORPUS_VARIABLES: &[&str] = &[];\n"),
    ]);

    let judged: Vec<&str> = findings
        .iter()
        .map(|finding| return finding.subject_name.as_str())
        .collect();

    assert_eq!(
        judged,
        vec!["CORPUS_VARIABLES", "GOVERNING_RECORD_IDS", "Table::All"],
        "all three instances OD-COMPLETENESS-001 analyses must be found"
    );
}

/// ---- the rule can say clean ----
///
/// The control that stops the rule being a counter. A check that fires on everything
/// is not a judgment, and a gate that can never be green is one everybody learns to
/// ignore.
#[test]
fn Test_A_Universe_Whose_Mirror_Exists_Should_Produce_No_Finding()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
         pub const TABLES: &[&str] = &[];\n",
    );
    let checking = Source(
        "a_test.rs",
        "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
    );
    let sources = vec![declaring.clone(), checking.clone()];

    let world = World_Over(&[
        (&declaring, &[]),
        (&checking, &["Test_Every_Table_Should_Be_Declared"]),
    ]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert!(findings.is_empty(), "{findings:?}");
}

/// ---- the negative control this rule exists for ----
///
/// A universe naming a check that does not exist reads as covered and checks nothing.
/// It is the only outcome that blocks, and it must block: an admitted gap is honest,
/// a false claim of coverage is not.
#[test]
fn Test_A_Mirror_That_Resolves_To_Nothing_Should_Block()
{
    let findings = Findings_Over(&[Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n",
    )]);

    let finding = Only(&findings);

    assert_eq!(finding.gate, GateCategory::Blocking);
    assert!(
        finding.Can_Fail_A_Build(),
        "a claim of coverage that checks nothing must be able to stop a build"
    );
    assert!(
        finding.summary.contains("Test_Renamed_Away"),
        "the finding must name the check that resolved to nothing: {}",
        finding.summary
    );
}

/// The severity ordering, asserted directly, because it is the judgment call this
/// rule makes and a later edit could quietly invert it.
#[test]
fn Test_A_False_Claim_Should_Outrank_An_Admitted_Gap()
{
    let admitted = Findings_Over(&[Source("a.rs", "pub const TABLES: &[&str] = &[];\n")]);
    let false_claim = Findings_Over(&[Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLES: &[&str] = &[];\n",
    )]);

    assert_eq!(Only(&admitted).gate, GateCategory::Advisory);
    assert_eq!(Only(&false_claim).gate, GateCategory::Blocking);
    assert!(
        Only(&false_claim).gate > Only(&admitted).gate,
        "a phantom mirror must outrank an admitted gap"
    );
    assert!(
        !Only(&admitted).Can_Fail_A_Build(),
        "an admitted gap is honest, and twelve of them exist; blocking on those \
         makes a gate that can never be green"
    );
}

/// A finding must carry its own provenance rather than leaving the reader to assume
/// it. `Derived` is the honest class: computed from source by a deterministic rule.
#[test]
fn Test_A_Finding_Should_Report_How_It_Was_Come_By()
{
    let findings = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
    let finding = Only(&findings);

    assert_eq!(finding.evidence, EvidenceClass::Derived);
    assert!(finding.Is_Mechanical());
    assert_eq!(finding.applicability, Applicability::Supported);
}

/// Identity is the name, not the path. A universe that moves file is the same
/// universe, and a finding keyed on its location would close and reopen for free.
#[test]
fn Test_The_Same_Universe_In_Two_Places_Should_Keep_One_Identity()
{
    let here = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
    let moved = Findings_Over(&[Source("b/c.rs", "pub const T: &[&str] = &[];\n")]);

    assert_eq!(Only(&here).subject, Only(&moved).subject);
    assert_ne!(Only(&here).locations, Only(&moved).locations);
}

/// An empty tree yields nothing, and that is exactly why the caller has to check for
/// vacuity. Asserted here so the property is written down where the rule is, rather
/// than being an unstated assumption the composition root happens to cover.
#[test]
fn Test_No_Sources_Should_Produce_No_Findings()
{
    let world = World_Over(&[]);
    let mut reader = world.Reader();

    assert!(Check_Completeness_Mirrors(&[], &mut reader).is_empty());
}

/// The rule's own back door, under the new source of names. A check name written
/// inside a fixture string is not a check, and resolving it would let a claim pass
/// while nothing checks it.
///
/// What keeps that true is no longer a parse performed here. It is the payload: a
/// parser reports the items a file declares and a string literal is not one, so the
/// name is not in the index. The fixture file's fact declares the function holding the
/// fixture and nothing else, which is what `nomos-lang-rust` really writes. That the
/// *provider* behaves this way and a line scanner does not is what
/// [`crate::Syntax_Requirement`]'s floor is for, and it is asserted below.
#[test]
fn Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Only_In_A_Fixture`.\npub const T: &[&str] = &[];",
    );
    let fixture = Source(
        "b.rs",
        "fn Fixture() { let source = \"fn Test_Only_In_A_Fixture() {}\"; }",
    );
    let sources = vec![declaring.clone(), fixture.clone()];

    let world = World_Over(&[(&declaring, &[]), (&fixture, &["Fixture"])]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert_eq!(Only(&findings).gate, GateCategory::Blocking);
}

/// A file that could not be read is reported, not skipped. A run that silently drops
/// what it could not parse and reports clean is the shape this workspace keeps
/// finding — and `Applicability` is the field that says so.
///
/// Which side reports it moved with `P10-SYNTAX-V2`. The rule used to parse the text
/// itself and refuse it here; now the provider refuses it, files no fact, and the
/// subject arrives unread. The property is the same and the path is shorter — there is
/// one parser in the workspace and it is the one that says a file does not parse.
#[test]
fn Test_An_Unparseable_File_Should_Be_Reported_And_Not_Fail_The_Build()
{
    let findings = Findings_Over(&[Source("broken.rs", "pub const ??? = ;")]);

    let unparseable = findings
        .iter()
        .find(|finding| return finding.summary.contains("no syntax fact could be read"))
        .expect("the file nobody could produce a fact for must be reported");

    assert_eq!(unparseable.applicability, Applicability::DependencyUnavailable);
    assert!(
        !unparseable.Can_Fail_A_Build(),
        "a rule that could not read its subject must not stop anybody"
    );
}

/// A mention is not a definition. Resolving against prose would let a comment naming
/// a deleted test keep the claim alive, which is the defect one level up.
#[test]
fn Test_A_Test_Named_Only_In_Prose_Should_Not_Resolve()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Deleted`.\npub const T: &[&str] = &[];\n",
    );
    let prose = Source("b.rs", "// see Test_Deleted for the comparison\n");
    let sources = vec![declaring.clone(), prose.clone()];

    let world = World_Over(&[(&declaring, &[]), (&prose, &[])]);
    let mut reader = world.Reader();

    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert_eq!(Only(&findings).gate, GateCategory::Blocking);
}

/// ---- the judgment reads a fact ----
///
/// The load-bearing test for `OD-RULES-001`, and the one that would catch a fact path
/// nothing consults. One tree, two runs, one difference: whether the store holds the
/// fact for the file that defines the check. Withheld, the claim does not resolve;
/// present, the rule reports nothing at all. The verdict turns on the fact and on
/// nothing else — the text handed to the rule is byte-identical in both runs.
#[test]
fn Test_A_Mirror_Should_Resolve_Only_Through_A_Fact()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Present`.\npub const T: &[&str] = &[];\n",
    );
    let checking = Source("b.rs", "#[test]\nfn Test_Present()\n{\n}\n");
    let sources = vec![declaring.clone(), checking.clone()];

    let with_fact = World_Over(&[(&declaring, &[]), (&checking, &["Test_Present"])]);
    let mut reader = with_fact.Reader();
    let resolved = Check_Completeness_Mirrors(&sources, &mut reader);

    assert!(
        resolved.is_empty(),
        "the fact for the defining file is present, so the claim resolves: {resolved:?}"
    );

    // The same two files, and the store is told about only one of them.
    let without_fact = World_Over(&[(&declaring, &[])]);
    let mut reader = without_fact.Reader();
    let unresolved = Check_Completeness_Mirrors(&sources, &mut reader);

    assert!(
        unresolved
            .iter()
            .any(|finding| return finding.subject_name == "b.rs"),
        "the subject whose fact was withheld must be named: {unresolved:?}"
    );
    assert!(
        unresolved
            .iter()
            .any(|finding| return finding.subject_name == "T"),
        "the claim must not resolve out of a text the rule can still see: {unresolved:?}"
    );
}

/// An empty store is not a clean tree.
///
/// Every subject unread, so the rule must say it could not run rather than say
/// nothing. Three properties together are what "could not run" means: the result is
/// not empty, every finding names an unavailability rather than a phantom, and nothing
/// in it can stop a build.
///
/// The count is two rather than three since `P10-SYNTAX-V2`, and the difference is
/// worth stating because it is a guarantee that changed rather than a fixture that
/// moved. A universe declared in a file whose fact was never read is no longer
/// discovered at all: discovery reads the fact, and there is no fact. The rule used to
/// find it in the text and report the claim as withheld, which was the same bypass
/// `OD-RULES-001` closed for check names — a rule reaching around its own fact layer.
/// What is kept is the property this test is named for: a run that read nothing renders
/// as one finding per unread file and never as a clean tree. What is given up is
/// per-claim detail inside a file nobody read, which is a sentence about a subject the
/// rule never saw. `OD-SYNTAX-002` records the trade.
#[test]
fn Test_An_Empty_Store_Should_Not_Report_A_Clean_Tree()
{
    let sources = vec![
        Source("a.rs", "/// Mirrored by `Test_Somewhere`.\npub const T: &[&str] = &[];\n"),
        Source("b.rs", "#[test]\nfn Test_Somewhere()\n{\n}\n"),
    ];

    let world = World::Offering(&[(PARSER, Parser_Guarantee())]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    assert!(
        !findings.is_empty(),
        "a run that read no fact at all must not render as a clean tree"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| {
                return finding.applicability == Applicability::DependencyUnavailable;
            })
            .count(),
        2,
        "one finding per unread subject: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| return !finding.Can_Fail_A_Build()),
        "a rule that read nothing must not stop anybody: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| return finding.gate != GateCategory::Blocking),
        "nothing may be reported as a phantom out of an index that was never built"
    );
}

/// A short index must not manufacture the one finding that blocks.
///
/// One file's fact is missing and the universe claims a mirror defined in that file.
/// The claim fails to resolve, and the rule may not call that a false claim of
/// coverage: it cannot tell one from a name it did not get to look for.
///
/// This is also the control that stops `OD-RULES-002` over-correcting into "every
/// unresolved claim blocks". The scoping added there is a narrowing of *which* claims
/// inherit the doubt and not a removal of the doubt, and this is the case it must
/// still cover: the missing subject is exactly the one that would have resolved the
/// name.
#[test]
fn Test_An_Incomplete_Index_Should_Not_Manufacture_A_Phantom()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_In_The_Unread_File`.\npub const T: &[&str] = &[];\n",
    );
    let unread = Source("b.rs", "#[test]\nfn Test_In_The_Unread_File()\n{\n}\n");
    let judged = Judged_Beside(&declaring, &unread);
    let claim = Named("T", &judged).expect("the universe is still judged");

    assert_eq!(claim.gate, GateCategory::Advisory);
    assert_eq!(claim.applicability, Applicability::DependencyUnavailable);
    assert!(
        !claim.Can_Fail_A_Build(),
        "a claim the rule could not check must not stop a build: {claim:?}"
    );
    assert!(
        claim.summary.contains("the check index is short b.rs"),
        "the finding must name the subject that could have resolved it: {}",
        claim.summary
    );
}

/// ---- OD-RULES-002: the shortfall is scoped to the claim it bears on ----
///
/// The defect the record was opened against. One unreadable subject anywhere silenced
/// every phantom in the tree, and over this workspace there is always one:
/// `tests/corpus/analysis/gamma/broken.rs` is a fixture the parser is supposed to
/// refuse. So the guard reported and never judged.
///
/// Here the phantom is claimed in a file whose fact was read, and the unreadable file
/// is present in the same run and does not spell the claimed name. Those two facts have
/// nothing to do with each other, and the judgment must be made.
#[test]
fn Test_A_Phantom_In_A_Read_Subject_Should_Block_Though_Another_Subject_Was_Unread()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );
    // The same shape as the real fixture: text no parser accepts, and no fact.
    let broken = Source("broken.rs", "pub const ??? = ;\n");
    let judged = Judged_Beside(&declaring, &broken);
    let claim = Named("T", &judged).expect("the universe is judged");

    assert_eq!(claim.gate, GateCategory::Blocking);
    assert_eq!(claim.applicability, Applicability::Supported);
    assert!(
        claim.Can_Fail_A_Build(),
        "a phantom claimed in a file whose facts were read must stop a build whatever \
         some other subject did: {claim:?}"
    );

    // The caveat travels with the judgment rather than replacing it. A reader who wants
    // to know what the run did not see is owed that on the finding, not instead of it.
    assert!(
        claim.summary.contains("short 1 subject(s)")
            && claim.summary.contains("Test_Renamed_Away"),
        "the blocking finding must carry the shortfall it ruled out: {}",
        claim.summary
    );

    // And the unreadable subject is still reported — once now rather than twice. It
    // used to be counted by the text side as well, which was this rule parsing the file
    // itself; there is one report because there is one reading.
    assert_eq!(
        judged
            .iter()
            .filter(|finding| return finding.subject_name == "broken.rs")
            .count(),
        1,
        "the unreadable file must still be reported: {judged:?}"
    );
}

/// The second control, and the one that stops the correction going too far.
///
/// A claim sited in a subject whose own fact could not be read, naming a check that
/// subject itself declares. Nothing resolves it and it is still not a phantom: the file
/// that would have resolved it is the file the index is short of. A universe and the
/// test that mirrors it living in one file is the ordinary shape in this workspace, so
/// this is not a corner.
#[test]
fn Test_A_Claim_In_An_Unread_Subject_Should_Not_Be_A_Phantom()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Right_Here`.\n\
         pub const T: &[&str] = &[];\n\
         #[cfg(test)]\n\
         mod tests\n\
         {\n\
         \x20   #[test]\n\
         \x20   fn Test_Right_Here()\n\
         \x20   {\n\
         \x20   }\n\
         }\n",
    );
    let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
    let sources = vec![declaring.clone(), other.clone()];

    // Every fact but the declaring file's own, so the index is real and short of it.
    let world = World_Over(&[(&other, &["Test_Something_Else"])]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    // Since `P10-SYNTAX-V2` the claim is not judged at all, because the universe is not
    // discovered: discovery reads the fact and this file has none. The property the
    // test is named for is unchanged and stronger — nothing here is a phantom — and the
    // subject is still reported, which is what keeps the run from rendering clean.
    assert!(
        !findings.iter().any(|finding| return finding.subject_name == "T"),
        "a claim in a file the rule never read must not be judged at all: {findings:?}"
    );

    let unread = findings
        .iter()
        .find(|finding| return finding.subject_name == "a.rs")
        .expect("the subject whose fact was missing must be reported");

    assert_eq!(unread.gate, GateCategory::Advisory);
    assert_eq!(unread.applicability, Applicability::DependencyUnavailable);
    assert!(
        !unread.Can_Fail_A_Build(),
        "a rule that did not read a subject must not stop anybody over it: {unread:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| return finding.gate != GateCategory::Blocking),
        "nothing may be reported as a phantom out of a file nobody read: {findings:?}"
    );
}

/// The unsound half of the scoping, in the direction it is allowed to be unsound.
///
/// `Could_Have_Declared` is a substring test over a text no parser read, so a name
/// spelled only in a comment counts as "could have declared it". That is a false
/// positive and it *withholds* a block, which is the safe direction and the reason an
/// unsound signal is admissible here at all: `Check_Index_Of` may not use text as a
/// source of names because an unsound positive there resolves a claim and silences the
/// rule, and this one can only ever add doubt.
#[test]
fn Test_A_Name_Spelled_In_A_Comment_Should_Still_Withhold_The_Block()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Only_Mentioned`.\npub const T: &[&str] = &[];\n",
    );
    let broken = Source(
        "broken.rs",
        "// Test_Only_Mentioned used to live here.\npub const ??? = ;\n",
    );
    let judged = Judged_Beside(&declaring, &broken);
    let claim = Named("T", &judged).expect("the universe is judged");

    assert_eq!(
        claim.gate,
        GateCategory::Advisory,
        "an unread file that spells the name is doubt, however it spells it: {claim:?}"
    );
    assert!(!claim.Can_Fail_A_Build());
}

/// Two claims, one run, one unreadable subject — and they are judged differently.
///
/// The sharpest form of what `OD-RULES-002` changed. Under one flag over the whole run
/// these two claims were indistinguishable and both were downgraded. The shortfall is a
/// property of the subjects each judgment rests on, so the run now says two different
/// things in the same breath.
#[test]
fn Test_Two_Claims_Under_One_Shortfall_Should_Be_Judged_Separately()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_In_The_Broken_File`.\n\
         pub const WITHHELD: &[&str] = &[];\n\
         /// Mirrored by `Test_Nowhere_At_All`.\n\
         pub const PHANTOM: &[&str] = &[];\n",
    );
    let broken = Source("broken.rs", "fn Test_In_The_Broken_File( ??? = ;\n");
    let sources = vec![declaring.clone(), broken.clone()];

    let world = World_Over(&[(&declaring, &[])]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    let judged: Vec<(&str, GateCategory)> = findings
        .iter()
        .filter(|finding| return finding.subject_name.starts_with(char::is_uppercase))
        .map(|finding| return (finding.subject_name.as_str(), finding.gate))
        .collect();

    assert_eq!(
        judged,
        vec![
            ("PHANTOM", GateCategory::Blocking),
            ("WITHHELD", GateCategory::Advisory),
        ],
        "one shortfall bears on one of these two claims and not the other: {findings:?}"
    );
}

/// And the shortfall is still whole-run when there is no index at all.
///
/// `MissingCapability` is not scoped by what a subject spells, and the reason is not a
/// carve-out: with no admitted provider there is no index, so no name can be shown
/// absent from it. Without this arm a run against a composition that offers the rule
/// nothing would block on every claim in the tree — a verdict reached by a rule that
/// never got an answer from anybody, which is what `OD-RULES-001` forbids.
#[test]
fn Test_With_No_Provider_Admitted_No_Claim_Should_Be_A_Phantom()
{
    let source = Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere_In_This_Tree`.\npub const T: &[&str] = &[];\n",
    );

    let world = World::Offering(&[]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&[source], &mut reader);

    // With nothing offering, no fact exists and no universe is discovered — so the
    // claim is not judged rather than judged leniently. `MissingCapability` is still
    // what the run reports, on the subject instead of on the claim, and the finding
    // that must never appear is a phantom.
    let unread = findings
        .iter()
        .find(|finding| return finding.subject_name == "a.rs")
        .expect("a run with no provider must report the subject it could not read");

    assert_eq!(unread.applicability, Applicability::MissingCapability);
    assert_eq!(unread.gate, GateCategory::Advisory);
    assert!(!unread.Can_Fail_A_Build());
    assert!(
        findings
            .iter()
            .all(|finding| return finding.gate != GateCategory::Blocking),
        "with no provider admitted there is no index for a name to be absent from, so \
         nothing may be called a phantom: {findings:?}"
    );
}

/// The control that stops the tests above from being satisfied by a rule that never
/// blocks. Every subject read, one claim that resolves to nothing, and it blocks.
#[test]
fn Test_A_Phantom_Should_Still_Block_When_Every_Subject_Was_Read()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );
    let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
    let sources = vec![declaring.clone(), other.clone()];

    let world = World_Over(&[(&declaring, &[]), (&other, &["Test_Something_Else"])]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    let claim = Only(&findings);

    assert_eq!(claim.gate, GateCategory::Blocking);
    assert_eq!(claim.applicability, Applicability::Supported);
    assert!(claim.Can_Fail_A_Build());
}

/// An admitted gap stays advisory and stays *evaluated* even when the index is short.
///
/// `D-134`'s decision 4 is that a false claim outranks an admitted gap, and the
/// amendment must not weaken the second half of it by letting an incomplete index
/// downgrade a universe that never claimed anything. Nothing was resolved for it, so
/// nothing could have been missed.
#[test]
fn Test_An_Admitted_Gap_Should_Not_Inherit_The_Indexs_Doubt()
{
    let declaring = Source("a.rs", "pub const T: &[&str] = &[];\n");
    let unread = Source("b.rs", "#[test]\nfn Test_Whatever()\n{\n}\n");
    let judged = Judged_Beside(&declaring, &unread);
    let gap = Named("T", &judged).expect("the universe is judged");

    assert_eq!(gap.gate, GateCategory::Advisory);
    assert_eq!(gap.applicability, Applicability::Supported);
    assert!(gap.summary.contains("declares no mirror"), "{}", gap.summary);
}

/// Nothing offers what the rule needs, which is a different sentence from "the store
/// had nothing". The remedy is registering a provider rather than running one, and
/// `Applicability` is where the difference is carried.
#[test]
fn Test_A_Composition_With_No_Provider_Should_Report_A_Missing_Capability()
{
    let source = Source("a.rs", "pub const T: &[&str] = &[];\n");

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
    let floor = Syntax_Requirement().minimum;

    assert!(
        !Scanner_Guarantee().Satisfies(&floor),
        "an unsound approximation must not be admissible as a source of check names"
    );
    assert!(
        Parser_Guarantee().Satisfies(&floor),
        "a floor no provider can meet is a declared need with nothing behind it"
    );

    let source = Source(
        "a.rs",
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );
    let world = World::Offering(&[(SCANNER, Scanner_Guarantee())]);
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&[source], &mut reader);

    // The floor keeps the scanner's answer out, so no fact reaches the rule and the
    // universe is not discovered. What the run reports is the subject, as a missing
    // capability — and crucially not as a universe that declares no mirror, which is
    // the downgrade `nomos.syntax.items.v2` and this floor both exist to prevent.
    let unread = findings
        .iter()
        .find(|finding| return finding.subject_name == "a.rs")
        .expect("the subject no admitted provider answered for must be reported");

    assert_eq!(unread.applicability, Applicability::MissingCapability);
    assert!(
        !unread.Can_Fail_A_Build(),
        "with only a scanner admitted the rule could not run, so it may not block"
    );
    assert!(
        findings
            .iter()
            .all(|finding| return finding.gate != GateCategory::Blocking),
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
        "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
    );

    let world = World::Offering(&[(PARSER, Parser_Guarantee())]).Materializing(
        &source,
        nomos_cap_syntax::SCHEMA,
        Blind_Payload(&["T"]),
    );
    let mut reader = world.Reader();
    let findings = Check_Completeness_Mirrors(&[source], &mut reader);

    let unobserved = findings
        .iter()
        .find(|finding| return finding.subject_name == "a.rs")
        .expect("a file whose provider saw no documentation must be reported");

    assert_eq!(unobserved.applicability, Applicability::Unparseable);
    assert!(
        unobserved.summary.contains("documentation"),
        "the finding must name the field that was not observed: {}",
        unobserved.summary
    );
    assert!(
        !findings.iter().any(|finding| return finding.subject_name == "T"),
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
    let source = Source("a.rs", "pub const T: &[&str] = &[];\n");

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
