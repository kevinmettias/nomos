//! Which crates produce facts, and which of them say what they promise about repeating
//! themselves.
//!
//! Derived from the source, never declared. A list of fact-producing crates checked into
//! this repository would be right on the day it was written and wrong the first time
//! somebody added a provider — and a stale list understates the set, which is the
//! direction that flatters: the crate nobody added would be the crate nobody checked.
//!
//! # What makes a crate a producer
//!
//! It constructs a [`nomos_analysis::MaterializedFact`]. That is the mechanical reading of
//! "an execution domain that produces a fact", and it is deliberately a construction
//! rather than a mention: the crate that *defines* the type names it constantly and
//! produces nothing, and a crate that merely reads facts names it too.
//!
//! This crate compiles against almost nothing on purpose, so the test is a scan of text
//! rather than a reflection over types. That is the same trade `gates.rs` makes, for the
//! same reason — an observer that linked the thing it observes would be a participant.

use crate::gates::{Source_Files, Without_Test_Modules};
use crate::Workspace;
use std::collections::BTreeSet;

/// The type whose construction marks a fact producer.
///
/// Assembled rather than written whole, so that this file — which necessarily contains the
/// spelling in order to search for it — is not itself found by the scan. `gates.rs` had to
/// learn this the hard way when its scanner found its own inventory.
#[must_use]
fn Fact_Type() -> String
{
    return concat!("Materialized", "Fact").to_owned();
}

/// The trait a producing domain is expected to implement.
#[must_use]
fn Strategy_Impl() -> String
{
    return concat!("impl ", "Strategy ", "for").to_owned();
}

/// One workspace member, and what its `src/` says about facts and determinism.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactDomain
{
    /// The package name.
    pub crate_name: String,
    /// Whether its `src/` constructs a fact.
    pub produces: bool,
    /// The strategy types it declares, by the name following `pub struct`.
    pub declarations: BTreeSet<String>,
}

/// Every workspace member that produces facts or declares a strategy, in a stable order.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Fact_Domains() -> Vec<FactDomain>
{
    let workspace = Workspace::Load();
    let fact_type = Fact_Type();
    let strategy_impl = Strategy_Impl();
    let mut found = Vec::new();

    for member in workspace.Members()
    {
        // The two test crates are excluded, and this is the one exclusion here.
        //
        // `tests/integration` does construct facts — `src/slice.rs` builds them to drive
        // the slice — and it is not an execution domain. It is the instrument that
        // measures them, and a thermometer does not have a temperature to declare. The
        // same holds for this crate, which names the type in order to search for it.
        //
        // Stated as a rule about what these crates *are* rather than as two names on a
        // list, because `docs/records/OD-GATE-001` records what an exemption list does to
        // a check: it becomes the easiest place to hide the thing the check exists to
        // find.
        if member.name.starts_with("nomos-integration-tests")
            || member.name.starts_with("nomos-contract-tests")
        {
            continue;
        }

        let source_root = member.root.join("src");
        if !source_root.is_dir()
        {
            continue;
        }

        let mut produces = false;
        let mut declarations = BTreeSet::new();

        for file in Source_Files(&source_root)
        {
            let Ok(text) = std::fs::read_to_string(&file)
            else
            {
                continue;
            };

            // A strategy declared inside a unit-test module is an example of the trait,
            // not a domain occupying a row of the table. `nomos-contracts` has two.
            let text = Without_Test_Modules(&text);

            for line in text.lines()
            {
                let trimmed = line.trim();

                // A comment naming the type describes a producer rather than being one,
                // which is the distinction that made two rows of the corpus-gate table
                // wrong before anything checked them.
                if trimmed.starts_with("//")
                {
                    continue;
                }

                if trimmed.contains(&format!("{fact_type} {{")) && !trimmed.contains("struct ")
                {
                    produces = true;
                }

                if let Some(name) = trimmed.strip_prefix(&strategy_impl)
                {
                    if let Some(declared) = name.split_whitespace().next()
                    {
                        declarations.insert(declared.to_owned());
                    }
                }
            }
        }

        if produces || !declarations.is_empty()
        {
            found.push(FactDomain {
                crate_name: member.name.clone(),
                produces,
                declarations,
            });
        }
    }

    found.sort();
    return found;
}
