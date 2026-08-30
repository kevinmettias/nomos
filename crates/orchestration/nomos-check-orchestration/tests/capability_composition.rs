//! The seams between `nomos_check_orchestration` and the eleven capability/provider crates its
//! own composition (`Registered`) declares and offers, exercised from outside the crate through
//! public types only.
//!
//! `composition.rs`'s own internal `#[cfg(test)] mod tests` already proves `Registered` wires
//! exactly five `Declare` calls, but its assertions never spell any of the eleven crate names
//! themselves -- it reads `registry.Declared().count()` rather than comparing against each
//! capability's own real contract. So nothing outside this crate proves the registry `Registered`
//! builds actually carries the SAME contracts and offers each capability/provider crate declares
//! for itself, rather than a stale or hand-copied version of them drifting quietly apart. This
//! file is that proof: it drives `nomos_check_orchestration::Registered` -- the crate's own
//! public composition root -- and checks every declared capability and every offered provider
//! against each source crate's own public `Capability_Contract()`/`Provider_Offer()`, through
//! nothing but public API on every side of the boundary.

use nomos_check_orchestration::Registered;

/// The five capabilities `Registered` declares, each checked against the real contract its own
/// capability crate publishes -- not a value this test invents.
#[test]
fn Test_Registered_Should_Declare_Every_Capability_Crates_Own_Real_Contract()
{
    let registry: nomos_capability::Registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let declared: Vec<_> = registry.Declared().collect();

    assert!(declared.contains(&&nomos_cap_syntax::Capability_Contract()), "the syntax capability must match nomos_cap_syntax's own contract");
    assert!(declared.contains(&&nomos_cap_dependency::Capability_Contract()), "the dependency capability must match nomos_cap_dependency's own contract");
    assert!(declared.contains(&&nomos_cap_controlflow::Capability_Contract()), "the controlflow capability must match nomos_cap_controlflow's own contract");
    assert!(declared.contains(&&nomos_cap_lint::Capability_Contract()), "the lint capability must match nomos_cap_lint's own contract");
    assert!(declared.contains(&&nomos_cap_dependency_policy::Capability_Contract()), "the dependency-policy capability must match nomos_cap_dependency_policy's own contract");
}

/// The syntax capability's three offers, each checked against the real offer its own language
/// provider crate publishes.
#[test]
fn Test_Registered_Should_Offer_Every_Syntax_Providers_Own_Real_Offer()
{
    let registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_syntax::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust::Provider_Offer()), "the parser offer must match nomos_lang_rust's own offer");
    assert!(offers.contains(&nomos_lang_rust_scan::Provider_Offer()), "the scanner offer must match nomos_lang_rust_scan's own offer");
    assert!(offers.contains(&nomos_lang_go::Provider_Offer()), "the Go offer must match nomos_lang_go's own offer");
}

/// The dependency capability's two offers, each checked against the real offer its own manifest
/// provider crate publishes.
#[test]
fn Test_Registered_Should_Offer_Every_Dependency_Providers_Own_Real_Offer()
{
    let registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_dependency::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_cargo::Provider_Offer()), "the Cargo offer must match nomos_lang_rust_cargo's own offer");
    assert!(offers.contains(&nomos_lang_go_modules::Provider_Offer()), "the Go modules offer must match nomos_lang_go_modules's own offer");
}

/// The controlflow capability's one offer, checked against `nomos_lang_rust::reachability`'s
/// own real offer.
#[test]
fn Test_Registered_Should_Offer_The_Reachability_Providers_Own_Real_Offer()
{
    let registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_controlflow::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust::reachability::Provider_Offer()), "the reachability offer must match nomos_lang_rust::reachability's own offer");
}

/// The lint capability's one offer, checked against `nomos_lang_rust_clippy`'s own real offer.
#[test]
fn Test_Registered_Should_Offer_The_Clippy_Providers_Own_Real_Offer()
{
    let registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_lint::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_clippy::Provider_Offer()), "the clippy offer must match nomos_lang_rust_clippy's own offer");
}

/// The dependency-policy capability's one offer, checked against `nomos_lang_rust_deny`'s own
/// real offer.
#[test]
fn Test_Registered_Should_Offer_The_Deny_Providers_Own_Real_Offer()
{
    let registry = Registered().expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_dependency_policy::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_deny::Provider_Offer()), "the deny offer must match nomos_lang_rust_deny's own offer");
}
