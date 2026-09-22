//! Every provider offer this workspace exports, against the offers a real check run takes.
//!
//! Being uncomposed is not a defect. `OD-ROADMAP-001` licenses building a provider before a
//! consumer exists, and `README.md`'s own row for `nomos-lang-rust-compiler` says the crate is
//! "Not yet composed into a real gate run". So a check that failed on any uncomposed offer would
//! be asserting something this workspace has decided is false, and it would be deleted within a
//! day.
//!
//! What was missing is the distinction. Nothing separated an offer deliberately built ahead of
//! its consumer from one whose composition line somebody forgot, and nothing noticed when the
//! number moved. It moved: the measurement behind `OD-PROJECT-007` counted three uncomposed
//! offers, and a fourth arrived with the C# provider hours later, with no check reporting the
//! change. `OD-PROJECT-007` then decided that a verb reports declared-and-unobserved as a row
//! state, which makes the honesty of that row depend on a distinction nothing recorded.
//!
//! So this module is a declared list with a reason per entry, in the shape
//! `tests/contract/tests/completeness_universes/table.rs` uses for declared universes: discovery
//! is mechanical, classification is not, and the classification lives beside the check that reads
//! it rather than in a second file the two could drift apart in.
//!
//! # Where the exported set comes from
//!
//! From the blessed surface snapshots under `tests/contract/surface`, not from a grep of
//! `crates/*/*/src`, and not from a list written here. A twenty-second provider is compared
//! without editing this file.
//!
//! The snapshots are the right oracle for three reasons, and the second is the one that decided
//! it:
//!
//! - **They say what is reachable, which is what "exported" means.** A `pub fn` in a private
//!   module cannot be named by another crate whatever its own modifier says, and a composition
//!   root that cannot name an offer cannot compose it. The snapshot resolver already answers
//!   exactly that question.
//! - **They carry the path a composition line has to spell.** A file-path scan would find
//!   `crates/languages/nomos-lang-rust-compiler/src/nested_lock/guarantee.rs` and infer
//!   `nomos_lang_rust_compiler::nested_lock::guarantee::Provider_Offer`, which is not a path that
//!   exists: that crate declares the module with `#[path]` under a different name and re-exports
//!   the function as `Nested_Locks_Provider_Offer`. Comparing inferred paths against composition
//!   lines would report a composed offer as missing and an uncomposed one as composed, for six
//!   `#[path]`-renamed modules in that crate alone.
//! - **They are derived and already held to the tree.** A snapshot is written by
//!   `NOMOS_SURFACE_BLESS`, never by hand, and `public_surface`'s own
//!   `Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot` fails on a stale one while
//!   `Test_Every_Snapshot_Should_Belong_To_A_Crate_That_Has_One` fails on a library with no
//!   snapshot at all. Reading them here is reading a derivation somebody else keeps honest.
//!
//! An offer is recognised by its **return type**, not by its name: any exported function
//! answering a bare `ProviderOffer` counts, so a future offer that is not called `Provider_Offer`
//! is still compared. [`Offer_Declared_By`] states the exact shape it reads.
//!
//! # What this derivation would miss
//!
//! - A provider crate added without blessing its snapshot. Its offer is invisible here — and
//!   `public_surface` is red in the same package until somebody blesses it, so the hole cannot be
//!   held open.
//! - An offer function that takes arguments, or one returning a `ProviderOffer` wrapped in
//!   anything. Neither exists today and both would read as uncomposed, which is the loud
//!   direction.
//! - A crate outside this workspace. Nothing composes one and no snapshot describes one.
//!
//! # Where the composed set comes from
//!
//! From `registry.Offer(...)` lines in the check run's own composition root, read as text.
//! `nomos-contract-tests` cannot call `Registered()` and compare `ProviderId`s against these
//! paths — an offer's provider id (`nomos.lang.rust.syn`) and the path that produces it
//! (`nomos_lang_rust::Provider_Offer`) are different strings, and bridging them would need this
//! crate to depend on all twenty-one provider crates to read their `PROVIDER` constants. The text
//! is what names the offer in the spelling the snapshot also uses, so the two sets compare
//! directly.
//!
//! The scan reads a line whose trimmed text *begins* with `registry.Offer(`, so this file's own
//! prose naming an offer in a doc comment is not read as composing it — the mistake
//! `boundaries/bundled_contract_half.rs` made once and documents. If the receiver is ever spelled
//! differently the scan finds nothing, every offer reads as uncomposed, and twenty-one entries
//! fail at once; and
//! [`Test_Every_Composed_Offer_Should_Be_An_Exported_Offer`] catches a mis-parse that produces a
//! path no crate exports. Both failure modes are loud.
//!
//! # Both directions
//!
//! An offer that is uncomposed and undeclared fails. An offer declared here as intentionally
//! uncomposed that has since been composed *also* fails, and so does one this workspace no longer
//! exports, because a stale exemption is how a list quietly stops meaning anything.
//!
//! There is deliberately no count constant beside [`UNCOMPOSED`]. The list is the count, and a
//! separate total would be a second spelling of its own length.

use crate::bands::Repository_Root;
use std::collections::BTreeSet;
use std::path::Path;

/// One offer this workspace exports and composes into nothing on purpose.
struct Uncomposed
{
    /// The path a composition line would spell, exactly as a surface snapshot renders it.
    offer: &'static str,
    /// Why nothing composes it, read from the crate's own documentation and the records and
    /// items that govern it.
    ///
    /// Never a placeholder. Four entries carrying one sentence repeated four times would make
    /// this list worthless on the day one of them stops being true, which is the only day it
    /// has to work.
    reason: &'static str,
}

/// Every exported offer no composition line takes, and why.
///
/// Classified by hand against each crate's own doc, the record that licensed it and the ledger
/// item that carries the remaining work, on 2026-09-21. Three of the four are held out by work
/// that is scheduled and reserved; the fourth is held out by having no consumer to be composed
/// for. None of the four is uncomposed by omission.
const UNCOMPOSED: &[Uncomposed] = &[
    Uncomposed {
        offer: "nomos_lang_csharp::Provider_Offer",
        reason: "The fourth offer against `nomos.cap.syntax.items`, built by \
                 P123-CSHARP-SYNTAX-PROVIDER-AND-PACKAGE, which could compose nothing because \
                 the composition root and the shared walk were other items' territory when it \
                 was authored. Composing it is not one Offer line: OD-CAPABILITY-009 has this \
                 root narrow a subject to a provider by name, so Recognized_Syntax_Provider and \
                 Recognized_Language each need a C# arm and the dependency-materialization write \
                 side needs the matching one, nomos-workspace-discovery's Registered_Extensions \
                 needs the C# extension or a .cs file is never walked at all, and the C# twin of \
                 the test asserting nomos_rules::GO_LANGUAGE equals nomos_lang_go::LANGUAGE has \
                 nothing to name until a CSHARP_LANGUAGE constant exists in nomos-rules. \
                 P123-CSHARP-JOINS-THE-RUN-AND-THE-WALK-2 reserves exactly those paths and is \
                 ready to claim.",
    },
    Uncomposed {
        offer: "nomos_lang_rust_compiler::Provider_Offer",
        reason: "nomos.cap.rust.copy_clones, this workspace's only offer backed by a real \
                 compiler semantic engine, and README.md's own row for the crate says \"Not yet \
                 composed into a real gate run\". What holds it out is the zone rule rather than \
                 any doubt about the provider: its rule Check_Copy_Clones lives inside the \
                 provider crate, composing means a nomos-rules descriptor naming the contract, \
                 and nomos-lang-rust-compiler is zoned Provider, which Permits forbids the Rules \
                 zone from naming. OD-ANALYSIS-007 version 2 decides that at that moment the \
                 contract moves to its own crate under crates/capabilities, one crate per \
                 capability and never one for the family, and \
                 P123-THE-COMPILER-CONTRACTS-LEAVE-THEIR-PROVIDER-AND-ITS-RULES-JOIN-THE-RUN-2 \
                 reserves that move. OD-ROADMAP-002 also paused this wiring; OD-ROADMAP-003 \
                 records that pause as lapsed, so what remains is the crate move and not a \
                 prohibition.",
    },
    Uncomposed {
        offer: "nomos_lang_rust_compiler::Nested_Locks_Provider_Offer",
        reason: "nomos.cap.rust.nested_locks, the same crate's second capability \
                 (P42-SEMANTIC-FACT-FAMILY), held out by the identical zone rule and carried by \
                 the identical item. A separate entry rather than one shared with copy_clones \
                 because the two are separate contracts with separate ids, schemas, payloads and \
                 guarantees: OD-ANALYSIS-007 version 2 measured the family-crate question \
                 against the one item that reserved such a crate and decided one crate per \
                 capability, so the two can be composed on different days and this list has to be \
                 able to say so. It is also the entry that would go stale invisibly under a name \
                 match, since the crate re-exports it renamed rather than as Provider_Offer.",
    },
    Uncomposed {
        offer: "nomos_lang_rust::rollup::Provider_Offer",
        reason: "nomos.cap.module.index, and the only one of the four whose reason is an absent \
                 consumer rather than scheduled work. OD-ANALYSIS-002 built it to give the \
                 invalidation engine its first derived fact with real dependency edges -- before \
                 it, InvalidationReport::dependent was a field no run could make non-empty -- and \
                 the consumer it was built for is nomos-analysis, not a rule. No variant of \
                 nomos_rules::RequiredFact names this capability, so no descriptor can ask for \
                 it, and declaring it in Registered() would compose a capability nothing reads, \
                 which the composition root's own comment about the words-policy family already \
                 calls \"wiring something nobody reads\". It is exercised by nomos-lang-rust's \
                 own tests/rollup.rs and by tests/integration's determinism productions. No item \
                 schedules composing it, and none should until a rule needs a module-level index.",
    },
];

/// The composition root whose `Offer` calls are what a real check run takes.
const COMPOSITION_ROOT: &str = "crates/orchestration/nomos-check-orchestration/src/composition.rs";

/// Where the blessed surface snapshots live.
const SURFACE_DIRECTORY: &str = "tests/contract/surface";

/// The path of the offer one snapshot line declares, if it declares one.
///
/// The shape read is `pub fn <path>(<parameters>) -> <type>`, where `<type>`'s last `::` segment
/// is exactly `ProviderOffer` and carries no generic. So `-> ProviderOffer` and
/// `-> nomos_capability::ProviderOffer` both count, while `-> Vec<nomos_capability::ProviderOffer>`
/// does not -- a function answering a collection of offers is describing candidates rather than
/// making one, which is what `nomos_integration_tests::Slice::Candidates` really does.
///
/// A parameter list mentioning `self` is skipped: a method on a type is not an offer a
/// composition root can call.
fn Offer_Declared_By(line: &str) -> Option<String>
{
    let declaration = line.trim().strip_prefix("pub fn ")?;
    let (signature, returned) = declaration.rsplit_once(" -> ")?;

    if !Answers_An_Offer(returned)
    {
        return None;
    }

    let (path, parameters) = signature.split_once('(')?;

    if parameters.contains("self")
    {
        return None;
    }

    return Some(path.trim().to_owned());
}

/// Whether a rendered return type is a `ProviderOffer` and nothing wrapping one.
fn Answers_An_Offer(returned: &str) -> bool
{
    let returned = returned.trim();

    if returned.contains('<')
    {
        return false;
    }

    return returned.rsplit("::").next() == Some("ProviderOffer");
}

/// Every offer one snapshot declares.
fn Offers_Declared_In(snapshot: &str) -> BTreeSet<String>
{
    return snapshot.lines().filter_map(Offer_Declared_By).collect();
}

/// Every offer this workspace exports, from every committed snapshot.
fn Exported_Offers() -> BTreeSet<String>
{
    let directory = Repository_Root().join(SURFACE_DIRECTORY);
    let entries = std::fs::read_dir(&directory)
        // A directory that cannot be opened is not a directory holding no offers. Folding the
        // error into an empty set would report every composition line as naming an offer nobody
        // exports, over a tree where every snapshot is checked in.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut found = BTreeSet::new();

    for entry in entries.flatten()
    {
        let path = entry.path();

        // A guard clause rather than a nested `if`: the extension test and the read one inside
        // the other put this at four levels of control flow, which `nesting-depth` reports.
        if !Is_A_Snapshot(&path)
        {
            continue;
        }

        let snapshot = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

        found.extend(Offers_Declared_In(&snapshot));
    }

    return found;
}

/// Whether a file in the snapshot directory is a snapshot.
fn Is_A_Snapshot(path: &Path) -> bool
{
    return path.extension().is_some_and(|extension| return extension == "txt");
}

/// The offer one composition line takes, if it takes one.
///
/// The trimmed line must *begin* with `registry.Offer(`, so prose naming an offer inside a doc
/// comment is not read as composing it.
fn Offer_Composed_By(line: &str) -> Option<String>
{
    let call = line.trim().strip_prefix("registry.Offer(")?;
    let (path, _) = call.split_once('(')?;

    return Some(path.trim().to_owned());
}

/// Every offer `source` composes.
fn Offers_Composed_In(source: &str) -> BTreeSet<String>
{
    return source.lines().filter_map(Offer_Composed_By).collect();
}

/// Every offer the check run's composition root takes.
fn Composed_Offers() -> BTreeSet<String>
{
    let path = Repository_Root().join(COMPOSITION_ROOT);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    return Offers_Composed_In(&source);
}

/// Every offer [`UNCOMPOSED`] declares.
fn Declared_Uncomposed() -> BTreeSet<String>
{
    return UNCOMPOSED.iter().map(|entry| return entry.offer.to_owned()).collect();
}

/// Every exported offer that is neither composed nor declared here.
///
/// Over two sets rather than over the tree, so the control below can construct a workspace that
/// violates the rule. A tree that satisfies a rule cannot demonstrate that the rule has teeth.
fn Unaccounted_Offers(exported: &BTreeSet<String>, composed: &BTreeSet<String>) -> Vec<String>
{
    let declared = Declared_Uncomposed();

    return exported
        .iter()
        .filter(|offer| return !composed.contains(*offer) && !declared.contains(*offer))
        .cloned()
        .collect();
}

/// Every entry of [`UNCOMPOSED`] whose offer is composed after all, with the reason that has gone
/// false printed beside it.
fn Stale_Exemptions(composed: &BTreeSet<String>) -> Vec<String>
{
    return UNCOMPOSED
        .iter()
        .filter(|entry| return composed.contains(entry.offer))
        .map(|entry| return format!("{}\n  declared uncomposed because: {}", entry.offer, entry.reason))
        .collect();
}

/// Every entry of [`UNCOMPOSED`] naming an offer this workspace no longer exports.
fn Vanished_Exemptions(exported: &BTreeSet<String>) -> Vec<String>
{
    return UNCOMPOSED
        .iter()
        .filter(|entry| return !exported.contains(entry.offer))
        .map(|entry| return entry.offer.to_owned())
        .collect();
}

/// An exported offer is composed, or this workspace has said why it is not.
///
/// The assertion this module exists for. A twenty-second provider fails here until somebody
/// either composes it or writes down the reason it waits.
#[test]
fn Test_Every_Exported_Offer_Should_Be_Composed_Or_Declared_Uncomposed()
{
    let exported = Exported_Offers();

    assert!(
        !exported.is_empty(),
        "no snapshot under {SURFACE_DIRECTORY} declares a ProviderOffer. Every comparison in \
         this module quantifies over that set, so an empty one passes having read nothing"
    );

    let unaccounted = Unaccounted_Offers(&exported, &Composed_Offers());

    assert!(
        unaccounted.is_empty(),
        "these offers are exported and reach nothing: {unaccounted:?}.\n\n\
         Being uncomposed is allowed -- OD-ROADMAP-001 licenses a provider built ahead of its \
         consumer -- but it has to be said out loud. Either add the Offer line to \
         {COMPOSITION_ROOT}, or add an entry to UNCOMPOSED in this file carrying the real reason \
         this one waits, read from the crate's own doc and the record or item that governs it."
    );
}

/// An offer declared as intentionally uncomposed is still uncomposed.
///
/// The direction usually forgotten. A stale exemption is how a list stops meaning anything: it
/// goes on excusing something that no longer needs excusing, and the next reader trusts it.
#[test]
fn Test_Every_Declared_Uncomposed_Offer_Should_Still_Be_Uncomposed()
{
    let composed = Composed_Offers();

    assert!(
        !composed.is_empty(),
        "no line of {COMPOSITION_ROOT} was read as composing an offer. Either the composition \
         root stopped offering anything, or the scan in this module no longer recognises how it \
         spells one"
    );

    let stale = Stale_Exemptions(&composed);

    assert!(
        stale.is_empty(),
        "these offers are declared here as intentionally uncomposed and are composed:\n\n{}\n\n\
         Delete their entries. The reason printed beside each one is now false, and a list \
         carrying a false reason is worse than no list, because the next reader takes it as \
         measurement.",
        stale.join("\n\n")
    );
}

/// An offer declared as intentionally uncomposed is still exported.
///
/// The second way the list goes stale, and the quiet one: a crate that drops or renames an offer
/// leaves an entry here excusing a path nothing spells, and the entry then looks like coverage.
#[test]
fn Test_Every_Declared_Uncomposed_Offer_Should_Still_Be_Exported()
{
    let vanished = Vanished_Exemptions(&Exported_Offers());

    assert!(
        vanished.is_empty(),
        "these offers are declared here and no snapshot under {SURFACE_DIRECTORY} exports them: \
         {vanished:?}.\n\n\
         Either the crate dropped or renamed the offer, in which case drop the entry, or its \
         snapshot is stale, in which case public_surface is red beside this."
    );
}

/// Every composed offer is one a snapshot exports.
///
/// Not a restatement of the assertion above: this one watches the derivation rather than the
/// workspace. The composition root compiles, so every path it names is reachable; a path missing
/// from the snapshots therefore means either that a snapshot is stale or that the scan in this
/// module mis-read a line, and both are reasons to stop trusting the two sets it compares.
#[test]
fn Test_Every_Composed_Offer_Should_Be_An_Exported_Offer()
{
    let exported = Exported_Offers();
    let composed = Composed_Offers();

    assert!(
        !composed.is_empty(),
        "no line of {COMPOSITION_ROOT} was read as composing an offer, so this comparison would \
         pass having read nothing"
    );

    let unknown: Vec<&String> = composed.difference(&exported).collect();

    assert!(
        unknown.is_empty(),
        "{COMPOSITION_ROOT} composes offers no snapshot under {SURFACE_DIRECTORY} exports: \
         {unknown:?}.\n\n\
         That file compiles, so these paths exist. Either the snapshots are stale -- \
         public_surface says which -- or the scan in this module read a line wrongly, and the \
         uncomposed set it derives cannot be believed until it is fixed."
    );
}

/// The two derivations reject what they must and accept what they must.
///
/// The negative control `OD-GATE-028` requires. Every assertion above passes over a tree that
/// already agrees with it, and from a green run that is indistinguishable from a scan that reads
/// nothing at all.
#[test]
fn Test_The_Derivations_Should_Reject_What_They_Must_And_Accept_What_They_Must()
{
    Test_The_Exported_Scan_Should_Read_A_Declaration_And_Not_A_Collection();
    Test_The_Composed_Scan_Should_Read_A_Call_And_Not_A_Comment();
    Test_The_Classification_Should_Report_Each_Of_The_Three_Ways_It_Can_Be_Wrong();
}

/// A set holding one offer, for a constructed workspace the real one cannot demonstrate.
fn One(offer: &str) -> BTreeSet<String>
{
    return std::iter::once(offer.to_owned()).collect();
}

/// [`Unaccounted_Offers`], [`Stale_Exemptions`] and [`Vanished_Exemptions`] each report the
/// violation they exist for, and each stays quiet over a workspace that is not violating it.
///
/// Every one of the three passes today over a tree that agrees with it, so without these the
/// three assertions above are indistinguishable from three comparisons that cannot fail. The
/// first entry of [`UNCOMPOSED`] is used as the declared offer throughout; its identity does not
/// matter, only that this list really holds it.
fn Test_The_Classification_Should_Report_Each_Of_The_Three_Ways_It_Can_Be_Wrong()
{
    let Some(declared) = UNCOMPOSED.first()
    else
    {
        panic!("UNCOMPOSED is empty, so these controls prove nothing about a list nothing holds");
    };

    let forgotten = "nomos_lang_nobody::Provider_Offer";

    assert!(
        !Unaccounted_Offers(&One(forgotten), &BTreeSet::new()).is_empty(),
        "an offer that is exported, uncomposed and undeclared was accepted. That is the whole \
         defect this module was opened about, and a check that accepts it is worse than none"
    );
    assert!(
        Unaccounted_Offers(&One(declared.offer), &BTreeSet::new()).is_empty(),
        "a declared offer was reported as unaccounted. The guard is wrong in the direction that \
         forbids what OD-ROADMAP-001 explicitly licenses, and it would be reverted rather than \
         fixed"
    );
    assert!(
        Unaccounted_Offers(&One(forgotten), &One(forgotten)).is_empty(),
        "a composed offer was reported as reaching nothing"
    );

    assert!(
        !Stale_Exemptions(&One(declared.offer)).is_empty(),
        "an exemption for an offer that has since been composed was accepted. A stale exemption \
         is how a list quietly stops meaning anything, and this is the direction usually forgotten"
    );
    assert!(
        Stale_Exemptions(&One(forgotten)).is_empty(),
        "composing an offer nobody exempted was reported as a stale exemption"
    );

    assert!(
        !Vanished_Exemptions(&BTreeSet::new()).is_empty(),
        "an exemption naming an offer this workspace no longer exports was accepted; the entry \
         would sit here reading as coverage of something nothing spells"
    );
    assert!(
        Vanished_Exemptions(&Declared_Uncomposed()).is_empty(),
        "an exemption whose offer is exported was reported as vanished"
    );
}

/// [`Offer_Declared_By`]'s own contract, over the four shapes the snapshots really contain.
fn Test_The_Exported_Scan_Should_Read_A_Declaration_And_Not_A_Collection()
{
    let bare = "pub fn nomos_lang_rust_cargo::Provider_Offer() -> ProviderOffer";
    let nested = "pub fn nomos_lang_rust::reachability::Provider_Offer() -> ProviderOffer";
    let renamed = "pub fn nomos_lang_rust_compiler::Nested_Locks_Provider_Offer() -> ProviderOffer";
    let qualified = "pub fn a::b::Offer_It() -> nomos_capability::ProviderOffer";
    let collection = "pub fn nomos_integration_tests::Slice::Candidates(&self) -> Vec<nomos_capability::ProviderOffer>";
    let borrowed = "pub fn a::b::Held(&self) -> ProviderOffer";
    let unrelated = "pub fn nomos_lang_rust::Recognition::Of_Path(path: &str) -> Recognition";

    assert_eq!(
        Offer_Declared_By(bare).as_deref(),
        Some("nomos_lang_rust_cargo::Provider_Offer"),
        "the plainest declaration in the snapshots was not read as an offer"
    );
    assert_eq!(
        Offer_Declared_By(nested).as_deref(),
        Some("nomos_lang_rust::reachability::Provider_Offer"),
        "an offer inside a module was not read; this is the spelling composition.rs uses for \
         reachability, and missing it would report a composed offer as uncomposed"
    );
    assert_eq!(
        Offer_Declared_By(renamed).as_deref(),
        Some("nomos_lang_rust_compiler::Nested_Locks_Provider_Offer"),
        "an offer re-exported under a different name was not read. Matching the name \
         Provider_Offer rather than the return type would lose exactly this one, which is one of \
         the four entries UNCOMPOSED carries"
    );
    assert_eq!(
        Offer_Declared_By(qualified).as_deref(),
        Some("a::b::Offer_It"),
        "an offer whose return type is written through its crate path was not read; the \
         snapshots render a return type as the source spells it, so both forms occur"
    );
    assert_eq!(
        Offer_Declared_By(collection),
        None,
        "a function answering a collection of offers was read as making one. That is the real \
         shape of nomos_integration_tests::Slice::Candidates, and counting it would add an \
         offer nobody could compose"
    );
    assert_eq!(
        Offer_Declared_By(borrowed),
        None,
        "a method taking a receiver was read as an offer a composition root could call"
    );
    assert_eq!(
        Offer_Declared_By(unrelated),
        None,
        "a declaration answering something else entirely was read as an offer"
    );
}

/// [`Offer_Composed_By`]'s own contract, including the prose case that cost
/// `bundled_contract_half` a false positive.
fn Test_The_Composed_Scan_Should_Read_A_Call_And_Not_A_Comment()
{
    let composed = "    registry.Offer(nomos_lang_go::Provider_Offer())?;";
    let renamed = "    registry.Offer(nomos_lang_rust_compiler::Nested_Locks_Provider_Offer())?;";
    let prose = "/// `nomos_lang_rust::reachability::Provider_Offer` states its own guarantee.";
    let described = "/// registry.Offer(nomos_lang_csharp::Provider_Offer()) is what composing it \
                     would look like.";
    let declared = "    registry.Declare(nomos_cap_lint::Capability_Contract())?;";

    assert_eq!(
        Offer_Composed_By(composed).as_deref(),
        Some("nomos_lang_go::Provider_Offer"),
        "a real composition line was not read; every offer would then report as uncomposed"
    );
    assert_eq!(
        Offer_Composed_By(renamed).as_deref(),
        Some("nomos_lang_rust_compiler::Nested_Locks_Provider_Offer"),
        "composing a renamed offer was not read, so the day that entry leaves UNCOMPOSED this \
         check would go on excusing it"
    );
    assert_eq!(
        Offer_Composed_By(prose),
        None,
        "an offer named in prose was read as composed. composition.rs really does name \
         reachability's offer in a doc comment, and reading it as a call is the false positive \
         boundaries/bundled_contract_half.rs already paid for once"
    );
    assert_eq!(
        Offer_Composed_By(described),
        None,
        "a doc comment quoting a whole composition line was read as one. This is the dangerous \
         direction: a stale exemption would then hide behind a description of it"
    );
    assert_eq!(
        Offer_Composed_By(declared),
        None,
        "declaring a capability was read as offering against it"
    );
}
