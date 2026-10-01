//! The table and the source must agree, in both directions.

use nomos_contract_tests::{Declared_Universes, UniverseKind};
use std::collections::BTreeSet;

/// This check is itself a completeness guard, and its universe is derived rather than
/// declared — so by `OD-COMPLETENESS-001`'s own corollary it owes nothing further on that
/// axis. The cost is paid by deriving.
#[test]
fn Test_The_Declared_Table_Should_Match_What_Is_Derived()
{
    let derived = Derived_Identities();
    let declared = Declared_Identities();
    let unclassified: Vec<&Identity> = derived.difference(&declared).collect();
    let vanished: Vec<&Identity> = declared.difference(&derived).collect();

    assert!(
        !derived.is_empty(),
        "nothing was derived, so every assertion here would pass having read nothing"
    );
    assert!(
        unclassified.is_empty(),
        "these declared universes are not classified in UNIVERSES: {unclassified:#?}.\n\
         Add a row saying whether something compares the list against the reality it \
         claims to enumerate. If nothing does, say so and raise UNMIRRORED_TOTAL — a \
         universe nobody classified is the shape OD-COMPLETENESS-001 exists to stop."
    );
    assert!(
        vanished.is_empty(),
        "these rows name universes that are no longer in the source: {vanished:#?}.\n\
         A table that keeps rows for things that are gone flatters itself in the other \
         direction."
    );
}

/// A universe's identity.
///
/// The kind is part of it, so a row that calls a constant an enumeration is a mismatch rather
/// than a harmless mislabel — the two carry different risks, and the risk text is what the
/// next reader acts on.
type Identity = (String, String, UniverseKind);

/// The identities the source declares, read through `nomos-rules`.
fn Derived_Identities() -> BTreeSet<Identity>
{
    return Declared_Universes()
        .into_iter()
        .map(|universe| return (universe.path, universe.name, universe.kind))
        .collect();
}

/// The identities this table declares.
fn Declared_Identities() -> BTreeSet<Identity>
{
    use crate::table::UNIVERSES;

    return UNIVERSES
        .iter()
        .map(|universe| {
            return (
                universe.path.to_owned(),
                universe.name.to_owned(),
                universe.kind,
            );
        })
        .collect();
}
