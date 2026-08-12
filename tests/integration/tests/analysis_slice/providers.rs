//! Two providers of one capability.
//!
//! What the registry hands back when two offers answer the same contract: which one wins at
//! a given floor, what a preference does when it cannot be served, and that two providers'
//! answers about one file are never filed under one key. The scale half of this — whether
//! the weaker provider's coverage is worth what it costs — is in [`crate::scale`], because
//! it needs a corpus.

use crate::common::{Alpha_One, Loose, Precision_Corpus};
use nomos_analysis::{FactStore, MaterializedFact};
use nomos_cap_syntax as syntax;
use nomos_contracts::{Applicability, CapabilityId};
use nomos_integration_tests::{
    Approximate_Floor, Decode_Surface, Registered, Resolved, RunReport, Slice, SourceFile
};
use nomos_lang_rust as rust;
use nomos_lang_rust_scan as scan;

/// One provider's answer about one file, read back out of its own store.
fn Answer_About(slice: &Slice, file: &SourceFile) -> MaterializedFact
{
    let key = slice.Syntax_Key(file).At(slice.Generation());

    return slice
        .Store()
        .Current(&key, slice.Generation())
        .expect("this provider answered for alpha/one.rs");
}

/// A floor only one offer clears resolves to that one.
#[test]
fn Test_A_Requirement_Only_One_Provider_Satisfies_Should_Resolve_To_That_One()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection: parsed,
        applicability: how,
    } = Slice::Over(&corpus).Resolved();

    assert_eq!(parsed.chosen.provider.As_Str(), rust::PROVIDER);
    assert_eq!(how, Applicability::Supported);
    assert!(
        parsed.alternatives.is_empty(),
        "only one offer clears this floor, so there is nothing to have been chosen over: {:?}",
        parsed.alternatives
    );
    // The scanner is registered and cannot serve this floor. Without that, the assertion
    // above passes over a registry that still has only one offer in it.
    assert_eq!(
        Loose(&corpus).Resolved().selection.chosen.provider.As_Str(),
        scan::PROVIDER,
        "the scanner is in the registry and can be reached"
    );
}

/// A preference that cannot be served is a fallback, and the caller is told.
///
/// This is the branch that was unreachable with one provider: naming a preference always
/// got it, so `SupportedWithFallback` had never been produced. The answer still stands —
/// the floor was met — and its provenance is not what was asked for, which is a thing the
/// caller has to be able to record.
#[test]
fn Test_A_Preference_That_Cannot_Be_Served_Should_Report_A_Fallback()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection,
        applicability: how,
    } = Slice::Over(&corpus).Preferring(scan::PROVIDER).Resolved();

    assert_eq!(
        selection.chosen.provider.As_Str(),
        rust::PROVIDER,
        "the preference cannot meet the floor, so it must not be honoured"
    );
    assert_eq!(
        how,
        Applicability::SupportedWithFallback,
        "and the caller must be told, or it cannot record why the answer came from \
         somewhere else"
    );
    // The positive control. A preference that *can* be served is not a fallback, and
    // without this the assertion above would pass over a registry that never honours one.
    assert_eq!(Loose(&corpus).Resolved().applicability, Applicability::Supported);
}

/// Two providers' answers about one file are two facts.
///
/// The key names the provider and its guarantee, so a store holding both holds them apart.
/// Filing them together would make "what does this file declare" answerable two ways under
/// one address, and whichever was written last would win.
#[test]
fn Test_Facts_From_Two_Providers_Should_Not_Share_A_Key()
{
    let corpus = Precision_Corpus();
    let file = Alpha_One(&corpus);
    let parsed = Slice::Over(&corpus).Syntax_Key(file);
    let scanned = Loose(&corpus).Syntax_Key(file);

    assert_eq!(parsed.subject, scanned.subject, "one file");
    assert_eq!(
        parsed.semantic_inputs, scanned.semantic_inputs,
        "and one set of bytes, so the two are answering the same question"
    );
    assert_ne!(parsed.provider, scanned.provider);
    assert_ne!(parsed.guarantee, scanned.guarantee);
    assert_ne!(
        parsed.Digest(),
        scanned.Digest(),
        "a weak answer and a strong one about one file must not share an address"
    );
}

/// The disagreement, over the corpus that can name it.
///
/// A parser refuses `gamma/broken.rs` — a stray byte order mark mid-file, which is not
/// valid Rust — so `gamma`'s rollup covers one of its two members and says so. A
/// line-reader has no refusal case: it answers for every file, weakly.
///
/// This is the trade the capability system exists to make explicit. Not "is there an
/// answer" but "what is this answer worth, and did the caller ask for one that good".
#[test]
fn Test_The_Weaker_Provider_Should_Answer_Where_The_Parser_Refuses()
{
    let corpus = Precision_Corpus();
    let parsed = Slice::Over(&corpus).Run(&corpus);
    assert_eq!(parsed.refused.len(), 1, "{:?}", parsed.refused);
    assert_eq!(parsed.degraded, vec!["gamma"]);

    let mut loose = Loose(&corpus);
    Answered_For_Everything(&loose.Run(&corpus));

    // What the coverage cost. The scanner reads gamma's second file and reports items in
    // it, which the parser could not — and the rollup that follows is an approximation,
    // which is exactly what the caller asked for by lowering its floor.
    let gamma = loose.Surface_Of(&corpus.In_Group("gamma")).expect("gamma has a rollup");
    let surface = Decode_Surface(&gamma.payload.bytes).expect("the rollup wrote this");
    assert_eq!(surface.files, 2, "both of gamma's files answered");
    assert_eq!(surface.unreachable, 0);
}

/// A line-reader has no refusal case, so every file gets an answer and no rollup is missing
/// a member — six files, six answers, where the parser managed five.
fn Answered_For_Everything(scanned: &RunReport)
{
    assert!(
        scanned.refused.is_empty(),
        "a line-reader has no refusal case: {:?}",
        scanned.refused
    );
    assert!(
        scanned.degraded.is_empty(),
        "and so no rollup is missing a member: {:?}",
        scanned.degraded
    );
    assert_eq!(
        scanned.syntax_materialized, 6,
        "six files, six answers, where the parser managed five"
    );
}

/// The two providers do not merely differ in guarantee — they differ in what they say.
///
/// Asserted over a file both can read, so the disagreement is about method rather than
/// about one of them having refused. If their payloads were identical the whole
/// arrangement would be theatre: two names for one answer, and no reason for a caller to
/// care which one it got.
#[test]
fn Test_The_Two_Providers_Should_Disagree_About_A_File_Both_Can_Read()
{
    let corpus = Precision_Corpus();
    let file = Alpha_One(&corpus);
    let mut strict = Slice::Over(&corpus);
    let mut loose = Loose(&corpus);
    strict.Run(&corpus);
    loose.Run(&corpus);

    let parsed = Answer_About(&strict, file);
    let scanned = Answer_About(&loose, file);
    assert_eq!(
        parsed.payload.schema, scanned.payload.schema,
        "one schema: what they share is the shape of an answer, which is the interface"
    );
    assert_ne!(
        parsed.payload.bytes, scanned.payload.bytes,
        "and different bytes, or there would be no reason for a caller to care which \
         provider answered"
    );
    assert_ne!(
        parsed.evidence, scanned.evidence,
        "a parse is verified and a pattern match is approximate, and the fact says which"
    );
    eprintln!(
        "alpha/one.rs — parsed: {:?}\n              scanned: {:?}",
        String::from_utf8_lossy(&parsed.payload.bytes),
        String::from_utf8_lossy(&scanned.payload.bytes)
    );
}

/// Two providers, one contract, and neither of them wrote it.
///
/// The composition is where this is observable at all. `nomos-cap-syntax` cannot name either
/// provider — it sits below both, deliberately, so that neither party can change the terms
/// the other is bound by. Each provider names the contract and not its peer. So the only
/// place all three are visible at once is here, which is the same reason the rest of this
/// file exists.
///
/// The ceiling is the sharp end. It bounds what *either* provider may claim, and while it
/// lived in `nomos-lang-rust` that crate could have raised or lowered what its peer was
/// permitted to promise, in a file the peer could not open.
#[test]
fn Test_Both_Providers_Should_Offer_Against_A_Contract_Neither_Declares()
{
    let registry = Registered();
    let capability = CapabilityId::New(syntax::CAPABILITY);
    let contract = registry
        .Declared()
        .find(|declared| return declared.id == capability)
        .expect("the syntax capability is declared");

    assert_eq!(
        contract.ceiling,
        syntax::Ceiling(),
        "the terms in the registry are the contract crate's, not a provider's"
    );
    assert_eq!(
        registry
            .Offers(&capability)
            .iter()
            .map(|offer| return offer.provider.As_Str())
            .collect::<Vec<&str>>(),
        vec![scan::PROVIDER, rust::PROVIDER],
        "both providers offer against the one contract"
    );
    // The ceiling leaves room neither provider occupies. Without this the assertion above
    // would pass over a ceiling that is merely the incumbent's guarantee restated — which
    // is a ceiling that has to be raised whenever somebody improves something, and one that
    // silently forbids a better second provider.
    for offer in registry.Offers(&capability)
    {
        assert_ne!(
            offer.guarantee,
            syntax::Ceiling(),
            "{} claims exactly the ceiling, so the ceiling is describing an implementation \
             rather than bounding the capability",
            offer.provider
        );
    }
}

/// What the registry does when more than one offer clears the floor.
///
/// # The rule, over the composition it was decided for
///
/// The strongest usable offer answers, and every offer it was chosen over comes back with
/// it. `nomos.lang.rust.scan` still sorts first by name and no longer wins by it, which is
/// the whole of what changed: selection stopped being a consequence of spelling.
///
/// This test used to assert the opposite — `..._Should_Resolve_By_Name_Order_Until_
/// Something_Says_Otherwise`, whose name recorded that the behaviour was observed rather
/// than intended. `docs/records/OD-CAPABILITY-001` records what decided it.
#[test]
fn Test_The_Strongest_Usable_Offer_Should_Answer_Whatever_The_Providers_Are_Called()
{
    let corpus = Precision_Corpus();
    let Resolved {
        selection,
        applicability: how,
    } = Slice::Over(&corpus).Accepting(Approximate_Floor()).Resolved();

    assert_eq!(how, Applicability::Supported);
    Answered_Despite_Its_Name(&selection);
}

/// The parser answers because it is stronger, not because of how it is spelled.
///
/// The name order really is against it here — `nomos.lang.rust.scan` sorts first — which is
/// what makes this a statement about names rather than a coincidence, and the guarantee
/// ranked the two, so the choice is a decision rather than a tiebreak.
fn Answered_Despite_Its_Name(selection: &nomos_capability::Selection)
{
    let passed_over = selection
        .alternatives
        .first()
        .expect("the scanner clears this floor too")
        .provider
        .clone();

    assert_eq!(
        selection.chosen.provider.As_Str(),
        rust::PROVIDER,
        "both offers clear this floor and the parser is strictly stronger, so it answers \
         — despite the scanner's name sorting first"
    );
    assert!(
        selection.chosen.provider.As_Str() > passed_over.As_Str(),
        "and the name order really is against it here, or this test proves nothing about \
         names: {} against {passed_over}",
        selection.chosen.provider
    );
    assert!(
        !selection.Passed_Over_Stronger(),
        "nothing usable was stronger than what answered, which is the rule"
    );
    assert!(
        selection.Unranked().is_empty(),
        "and the guarantee ranked them, so this composition's provider choice is a \
         decision rather than a tiebreak: {:?}",
        selection.Unranked()
    );
}

/// What the lowered floor bought, and that it did not cost the files it was not for.
///
/// The situation that raised OD-CAPABILITY-001: a caller lowers its floor to `Approximate`
/// because a parser refuses seven files in the scale corpus, and wants an answer for those
/// seven without giving up the exact answer for the other 7,573.
///
/// Both of the reflex rules fail it. Name order serves the scanner for every file, so the
/// coverage costs precision everywhere. Strongest-wins alone serves the parser for every
/// file, so lowering the floor buys *nothing* — the same seven files go unanswered and the
/// caller cannot tell its requirement changed anything. The selection is what makes the
/// floor mean something: the parser answers, and the offer the floor admitted is reachable
/// for the subjects the parser cannot serve.
///
/// The registry cannot spend it, because a requirement names a capability and not a
/// subject, and which files a parser will refuse is not knowable until it reads them.
/// Spending it per subject is `Slice::Dispatch`'s to do and is not done here.
#[test]
fn Test_A_Lowered_Floor_Should_Make_The_Weaker_Offer_Reachable_Without_Serving_It()
{
    let corpus = Precision_Corpus();
    let parsed = Slice::Over(&corpus).Resolved().selection;
    let lowered = Slice::Over(&corpus)
        .Accepting(Approximate_Floor())
        .Resolved()
        .selection;

    assert_eq!(
        parsed.chosen, lowered.chosen,
        "lowering the floor must not change who answers; it widens what is admitted, and \
         the strongest thing admitted did not change"
    );
    assert!(
        parsed.Weaker().is_empty(),
        "the strict floor admits the scanner nowhere, so there is nothing to fall back to"
    );
    assert_eq!(
        lowered
            .Weaker()
            .iter()
            .map(|offer| return offer.provider.As_Str().to_owned())
            .collect::<Vec<_>>(),
        vec![scan::PROVIDER.to_owned()],
        "and the lowered one admits exactly the scanner, which is what the caller widened \
         its requirement to reach"
    );
}
