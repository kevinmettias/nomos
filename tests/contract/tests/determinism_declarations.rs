//! Every crate that produces a fact declares what it promises about repeating itself.
//!
//! `nomos-contracts` has carried `Strategy`, `DeterminismStrength`, `ReproducibilityScope`
//! and `TraceEquivalence` — plus a six-row table naming which execution domain claims
//! what — since it was written, and for that whole time no crate in the workspace
//! implemented any of them. "A schema is not a feature" is this project's own rule, and it
//! was sitting unenforced in the one crate whose every type is reimplemented by peers that
//! will never compile it. An unimplemented declaration there is a published protocol
//! commitment nothing has ever been held to.
//!
//! This file is the half that stops it recurring. `tests/integration/tests/determinism.rs`
//! checks that the four declarations are *true*; this checks that a new fact producer
//! cannot arrive without making one — which is the failure that would otherwise be
//! invisible, because a crate that declares nothing has no test to fail.

use nomos_contract_tests::{FactDomain, Fact_Domains};

/// A producer with no declaration is the defect this file exists for.
#[test]
fn Test_Every_Fact_Producing_Crate_Should_Declare_A_Strategy()
{
    let domains = Fact_Domains();

    let producers: Vec<&FactDomain> = domains
        .iter()
        .filter(|domain| return domain.produces)
        .collect();

    // The vacuity guard, and it is not decoration. Every assertion below iterates over
    // this set, so a scanner that matched nothing — a renamed type, a walk that found no
    // files, a `cargo metadata` that returned an empty workspace — would pass every one
    // of them while checking nothing at all. That is the exact shape
    // `docs/records/OD-GATE-001` measures at 68 tests, and the reason it is written here
    // is that this check would otherwise be the sixty-ninth.
    assert!(
        producers.len() >= 2,
        "only {} fact-producing crates were found, which is fewer than the two providers \
         this workspace is known to have. The scan found nothing rather than the workspace \
         holding nothing.",
        producers.len()
    );

    let undeclared: Vec<&str> = producers
        .iter()
        .filter(|domain| return domain.declarations.is_empty())
        .map(|domain| return domain.crate_name.as_str())
        .collect();

    assert!(
        undeclared.is_empty(),
        "these crates construct facts and declare no Strategy: {undeclared:?}.\n\
         A fact whose producer promises nothing about reproducing it cannot be cached, \
         compared across machines, or used as a baseline — and the absence of a promise \
         reads exactly like a promise that was kept. Add an `impl Strategy` naming the row \
         of the domain table this crate occupies, and a test in \
         tests/integration/tests/determinism.rs that holds it to it."
    );
}

/// The inverse direction, and the one that catches a declaration going stale.
///
/// A crate that declared a strategy and then stopped producing facts has a promise nobody
/// can check, which is the same defect as the missing declaration wearing the other face.
#[test]
fn Test_Every_Declaring_Crate_Should_Produce_Facts()
{
    let domains = Fact_Domains();

    let declaring: Vec<&FactDomain> = domains
        .iter()
        .filter(|domain| return !domain.declarations.is_empty())
        .collect();

    assert!(
        declaring.len() >= 2,
        "only {} crates declare a Strategy; the scan found nothing rather than the \
         workspace holding nothing",
        declaring.len()
    );

    for domain in declaring
    {
        assert!(
            domain.produces || Serves_Facts(&domain.crate_name),
            "{} declares {:?} and neither constructs a fact nor is one of the domains \
             recorded as serving them. A declaration with no behaviour behind it is the \
             thing this item was opened to remove.",
            domain.crate_name,
            domain.declarations
        );
    }
}

/// The two domains that answer for facts without constructing one.
///
/// `nomos-analysis` is the cache — it stores and serves what a provider built, and its
/// `FactReuse` declaration is about reuse rather than production. `nomos-workspace`
/// encodes the snapshot every fact is measured against, and its bytes are an identity
/// those facts inherit.
///
/// Named here rather than exempted in the scanner, so that the list is an assertion in a
/// test somebody reads and not a `continue` somebody skims.
fn Serves_Facts(crate_name: &str) -> bool
{
    return matches!(crate_name, "nomos-analysis" | "nomos-workspace");
}

/// What the scan actually found, printed.
///
/// `docs/records/OD-GATE-001` had to be amended when a column it cited turned out to be
/// declared and never derived. A run that prints its inventory is how the next such
/// disagreement gets noticed by somebody reading a log rather than by somebody auditing.
#[test]
fn Test_The_Fact_Domain_Inventory_Should_Be_Reported()
{
    let domains = Fact_Domains();

    assert!(
        !domains.is_empty(),
        "no crate in this workspace produces facts or declares a strategy, which cannot \
         be true of a workspace with two providers in it"
    );

    for domain in &domains
    {
        eprintln!(
            "fact domain: {} produces={} declares={:?}",
            domain.crate_name, domain.produces, domain.declarations
        );
    }
}
