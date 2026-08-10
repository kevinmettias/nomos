//! Every execution domain this workspace has declares what it promises about repeating
//! itself, and every declaration has something holding it to it.
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
//! checks that the declarations are *true*; this checks that a domain cannot arrive without
//! making one — which is the failure that would otherwise be invisible, because a crate
//! that declares nothing has no test to fail.
//!
//! # Why one predicate was not enough
//!
//! The producer predicate — a crate that constructs a fact — was the whole of this file
//! until `P10-SPEC-DETERMINISM`. It cannot see a domain that serializes or projects, and
//! that is not hypothetical: two rows of the table went undeclared for exactly as long as
//! this guard was the only thing looking, and were found by a person reading the table
//! rather than by anything failing. `docs/records/OD-DETERMINISM-002` records the repair.
//!
//! So the universe is now derived from three directions at once. What produces facts, what
//! the contracts table says exists, and what the harness measures. A row nobody occupies
//! fails [`Test_Every_Occupied_Row_Of_The_Domain_Table_Should_Be_Declared`]; a declaration
//! nobody checks, and a check whose declaration was deleted, both fail
//! [`Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness`]. None of the three is a
//! list kept by hand, which is what `OD-COMPLETENESS-001` requires of a completeness guard:
//! a derived universe owes nothing further on that axis, and the cost is paid by deriving.

use nomos_contract_tests::{
    Declaration, Domain_Table, DomainRow, Fact_Domains, FactDomain, Harnessed_Strategies,
};
use std::collections::BTreeSet;

/// A producer with no declaration is the defect this file started as.
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
/// A crate that declared a strategy and has nothing checking it has a promise nobody can
/// falsify, which is the same defect as the missing declaration wearing the other face —
/// and worse, because a declaration is what another system reads and plans around.
///
/// This replaced a check that asked whether a declaring crate produced facts, with two
/// crates named beside it as "serving" them instead. That list was about to grow to four,
/// and a list that grows every time the check is right is a list that will one day be
/// wrong. The question worth asking is not what a declaring crate does; it is whether
/// anything would notice if the declaration were false.
#[test]
fn Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness()
{
    let declared: BTreeSet<String> = Fact_Domains()
        .iter()
        .flat_map(|domain| return domain.declarations.iter())
        .map(|declaration| return declaration.strategy.clone())
        .collect();
    let harnessed = Harnessed_Strategies();

    assert!(
        declared.len() >= 4,
        "only {} strategy declarations were found in the workspace; the scan found nothing \
         rather than the workspace holding nothing",
        declared.len()
    );
    assert!(
        harnessed.len() >= 4,
        "only {} strategies are registered with the integration harness; the scan found \
         nothing rather than the harness measuring nothing",
        harnessed.len()
    );

    let unchecked: Vec<&String> = declared.difference(&harnessed).collect();
    assert!(
        unchecked.is_empty(),
        "these strategies are declared and nothing holds them to it: {unchecked:?}.\n\
         A declaration with no test behind it is worse than no declaration, because a \
         declaration is cited. Register it in tests/integration/tests/determinism.rs so \
         that the obligations its own triple implies are discharged against its behaviour."
    );

    let orphaned: Vec<&String> = harnessed.difference(&declared).collect();
    assert!(
        orphaned.is_empty(),
        "the harness measures these strategies and no crate declares one: {orphaned:?}.\n\
         Either an `impl Strategy` was removed and the check left behind, in which case the \
         domain now promises nothing while a test still reports on it, or the type was \
         renamed and this is the rename nobody finished."
    );
}

// ---------------------------------------------------------------------------
// The table is the universe, and it is read from the crate that publishes it.
// ---------------------------------------------------------------------------

/// A row of the table with no domain in this tree, and what it would take to change that.
///
/// The `OD-GATE-001` remedy rather than a gate that can never be green: an unoccupied row
/// is counted rather than hidden, and a crate arriving to occupy one has to remove its row
/// from here deliberately. Kept private, and small, and mirrored by
/// [`Test_Every_Unoccupied_Row_Should_Still_Be_A_Row_Of_The_Table`] — a declared list whose
/// members are never compared against the thing they claim to describe is precisely what
/// `OD-COMPLETENESS-001` is about.
const UNOCCUPIED: &[(&str, &str)] = &[
    (
        "Correction planning and staging",
        "nothing in this tree plans or stages a correction. `nomos-rules` judges and \
         reports; the step that would apply a fix does not exist, so there is no domain to \
         declare and a declaration would be a promise about code nobody has written.",
    ),
    (
        "Progress UI, logs, telemetry, agent execution",
        "the `None` row, and the one that keeps determinism affordable. The CLI prints, \
         and nothing about what it prints is a fact. A domain here would be declaring that \
         it promises nothing, which is what the row already says.",
    ),
];

/// The table has to be readable before anything below it means anything.
#[test]
fn Test_The_Domain_Table_Should_Be_Read_From_The_Contracts_Crate()
{
    let rows = Domain_Table();

    assert!(
        rows.len() >= 6,
        "only {} row(s) were read out of the contracts domain table. Every assertion below \
         quantifies over these rows, so a table this scanner cannot read passes them all \
         having looked at nothing — and that crate's module documentation is where the \
         published claim lives.\nRead: {rows:#?}",
        rows.len()
    );

    for row in &rows
    {
        assert!(
            !row.domain.is_empty() && !row.scope.is_empty() && !row.trace.is_empty(),
            "a row was read with an empty cell, so the parse is wrong rather than the \
             table: {row:#?}"
        );
    }

    let claiming = rows
        .iter()
        .filter(|row| return row.Claims_Reproducibility())
        .count();
    assert!(
        claiming > 0,
        "no row of the table claims reproducibility, which cannot be true of the table \
         this workspace's determinism rests on"
    );
}

/// The check that would have caught the two rows `P9-DETERMINISM` could not reach.
///
/// Every row that claims reproducibility must be occupied by a declaration somewhere in the
/// workspace, matched on all three axes. A row that is not occupied has to say so in
/// [`UNOCCUPIED`], with a reason the next reader can act on.
#[test]
fn Test_Every_Occupied_Row_Of_The_Domain_Table_Should_Be_Declared()
{
    let rows = Domain_Table();
    let declarations: Vec<Declaration> = Fact_Domains()
        .iter()
        .flat_map(|domain| return domain.declarations.iter())
        .cloned()
        .collect();

    assert!(
        !rows.is_empty() && !declarations.is_empty(),
        "one side of this comparison is empty, so it cannot fail"
    );

    let missing: Vec<&str> = rows
        .iter()
        .filter(|row| return row.Claims_Reproducibility())
        .filter(|row| {
            return !declarations
                .iter()
                .any(|declaration| return declaration.Occupies(row));
        })
        .filter(|row| return !Recorded_Unoccupied(row))
        .map(|row| return row.domain.as_str())
        .collect();

    assert!(
        missing.is_empty(),
        "these rows of the contracts domain table claim reproducibility and nothing in \
         this workspace declares them: {missing:?}.\n\
         Every peer that reimplements nomos-contracts reads that table, so an unoccupied \
         row is a promise made on behalf of an implementation nobody has held to anything. \
         Add an `impl Strategy` with the row's triple and a test in \
         tests/integration/tests/determinism.rs, or — if this tree genuinely has no such \
         domain — say so in UNOCCUPIED."
    );
}

/// A declaration that occupies no row is a promise the published table does not make.
///
/// The other direction, and the cheaper mistake: a crate that declares a triple the table
/// does not have is a crate claiming something no peer has agreed to, in a vocabulary they
/// share. It reads as rigour and means nothing.
#[test]
fn Test_Every_Declaration_Should_Occupy_A_Row_Of_The_Table()
{
    let rows = Domain_Table();
    assert!(!rows.is_empty(), "no rows were read, so this cannot fail");

    let stray: Vec<String> = Fact_Domains()
        .iter()
        .flat_map(|domain| {
            return domain.declarations.iter().map(move |declaration| {
                return (domain.crate_name.clone(), declaration.clone());
            });
        })
        .filter(|(_, declaration)| {
            return !rows.iter().any(|row| return declaration.Occupies(row));
        })
        .map(|(crate_name, declaration)| {
            return format!(
                "{crate_name}::{} declares {}/{}/{}",
                declaration.strategy, declaration.strength, declaration.scope, declaration.trace
            );
        })
        .collect();

    assert!(
        stray.is_empty(),
        "these declarations match no row of the contracts domain table: {stray:#?}.\n\
         Either the declaration is wrong, or the table has grown a domain and nobody wrote \
         the row — and the table is what every peer reads."
    );
}

/// The mirror for [`UNOCCUPIED`], in the direction that flatters.
///
/// A row named here that the table does not have would silence a real gap the moment the
/// table was reworded, and nothing would say so.
#[test]
fn Test_Every_Unoccupied_Row_Should_Still_Be_A_Row_Of_The_Table()
{
    let rows = Domain_Table();
    assert!(!rows.is_empty(), "no rows were read, so this cannot fail");

    let vanished: Vec<&str> = UNOCCUPIED
        .iter()
        .filter(|(domain, _)| {
            return !rows.iter().any(|row| return row.domain == *domain);
        })
        .map(|(domain, _)| return *domain)
        .collect();

    assert!(
        vanished.is_empty(),
        "UNOCCUPIED names rows the contracts table does not have: {vanished:?}.\n\
         An exception for a row that no longer exists excuses nothing and hides the row \
         that replaced it."
    );

    let occupied_anyway: Vec<&str> = UNOCCUPIED
        .iter()
        .filter(|(domain, _)| {
            return Fact_Domains().iter().any(|found| {
                return found.declarations.iter().any(|declaration| {
                    return rows
                        .iter()
                        .any(|row| return row.domain == *domain && declaration.Occupies(row));
                });
            });
        })
        .map(|(domain, _)| return *domain)
        .collect();

    assert!(
        occupied_anyway.is_empty(),
        "these rows are recorded as having no domain in this tree and something declares \
         them: {occupied_anyway:?}.\n\
         Remove the row from UNOCCUPIED — an exception that is no longer needed is an \
         exception that will excuse the next gap."
    );
}

/// Whether a row is recorded as having no domain in this tree.
fn Recorded_Unoccupied(row: &DomainRow) -> bool
{
    return UNOCCUPIED
        .iter()
        .any(|(domain, _)| return *domain == row.domain);
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

    for row in Domain_Table()
    {
        eprintln!(
            "domain table row: {} — {}/{}/{}{}",
            row.domain,
            row.strength,
            row.scope,
            row.trace,
            if Recorded_Unoccupied(&row)
            {
                " (no domain in this tree)"
            }
            else
            {
                ""
            }
        );
    }
}
