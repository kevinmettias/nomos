//! Every judgment this rule can reach, exercised against a real source tree.
//!
//! The fixture is here and the claims are in the five modules below, because the fixture is
//! the one thing all of them share: a registry, a store, and the payload a real parser would
//! have written for a source. A suite whose fixtures are restated per file drifts into
//! several compositions that agree only by coincidence.

use super::*;
use nomos_analysis::{
    Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{ProviderOffer, Registry};
use nomos_contracts::{
    Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SchemaId,
};

mod historical;
mod judgments;
mod provider_floor;
mod reads_a_fact;
mod scoping;

/// A stand-in for a parser, at exactly the guarantee `nomos-lang-rust` declares.
///
/// Named rather than imported. `nomos-rules` must not depend on a provider even in
/// test code — that is the edge the registry exists to remove, and a rule crate that
/// names one has answered the question the floor is supposed to ask. The end-to-end
/// path with the real provider belongs in the composition root, and `nomos-cli`'s
/// tests are where it is asserted.
const PARSER: &str = "nomos.test.parses";

fn Parser_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
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
fn Source(path: &str, text: String) -> SourceFile
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
        return crate::checks::test_support::Test_Context();
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

/// The findings one world produces over a list of sources.
fn Judged_In(world: &World, sources: &[SourceFile]) -> Vec<Finding>
{
    let mut reader = world.Reader();

    return Check_Completeness_Mirrors(sources, &mut reader);
}

/// Whether anything in a run was called a phantom, which is the only verdict that blocks.
fn Has_A_Blocking_Finding(findings: &[Finding]) -> bool
{
    return findings
        .iter()
        .any(|finding| return finding.gate == GateCategory::Blocking);
}
