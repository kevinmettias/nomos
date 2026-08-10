//! Which crates are execution domains, what each promises about repeating itself, and
//! which row of the contracts table it is promising against.
//!
//! Derived from the source, never declared. A list of domains checked into this repository
//! would be right on the day it was written and wrong the first time somebody added one —
//! and a stale list understates the set, which is the direction that flatters: the crate
//! nobody added would be the crate nobody checked.
//!
//! # Three derivations, because one predicate was not enough
//!
//! [`Fact_Domains`] answers "which crates produce facts, and what do they declare". Its
//! producer predicate is the construction of a [`nomos_analysis::MaterializedFact`], which
//! is the mechanical reading of "an execution domain that produces a fact" — and it is
//! deliberately a construction rather than a mention: the crate that *defines* the type
//! names it constantly and produces nothing.
//!
//! That predicate cannot see a domain that serializes or projects rather than producing,
//! which is how spec-bundle serialization and the projection engine sat undeclared for as
//! long as they did — the completeness guard that would have caught them could not reach
//! them. So two more derivations answer the question from the other end:
//! [`Domain_Table`] reads the six-row table out of `nomos-contracts` itself, which is the
//! authority on what rows exist, and [`Harnessed_Strategies`] reads which declarations the
//! integration harness actually measures. Between them, a row nobody occupies and a
//! declaration nobody checks are both visible without any list being kept by hand.
//!
//! This crate compiles against almost nothing on purpose, so all three are scans of text
//! rather than reflection over types. That is the same trade `gates.rs` makes, for the
//! same reason — an observer that linked the thing it observes would be a participant.

use crate::gates::{Source_Files, Without_Test_Modules};
use crate::Workspace;
use std::collections::BTreeSet;
use std::path::PathBuf;

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

/// The trait a domain implements in order to say what it promises.
#[must_use]
fn Strategy_Impl() -> String
{
    return concat!("impl ", "Strategy ", "for").to_owned();
}

/// One declaration: the type that made it, and the triple it declared.
///
/// The triple is carried as text rather than as the three `nomos-contracts` enums, because
/// this crate does not link `nomos-contracts` and must not start: the whole point of that
/// crate is that it is reimplemented by peers who never compile it, and an observer that
/// compiled it would be a participant in the protocol it is watching.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration
{
    /// The type name following `impl Strategy for`.
    pub strategy: String,
    /// The variant named by `const STRENGTH`.
    pub strength: String,
    /// The variant named by `const SCOPE`.
    pub scope: String,
    /// The variant named by `const TRACE`.
    pub trace: String,
}

impl Declaration
{
    /// Whether this declaration is the row's occupant.
    ///
    /// All three axes, because two rows of the table differ only in scope and matching on
    /// strength alone would report the weaker one as occupied by the stronger one's
    /// declaration — a completeness guard passing because it compared too little.
    #[must_use]
    pub fn Occupies(&self, row: &DomainRow) -> bool
    {
        return self.strength == row.strength
            && self.scope == row.scope
            && self.trace == row.trace;
    }
}

/// One workspace member, and what its `src/` says about facts and determinism.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactDomain
{
    /// The package name.
    pub crate_name: String,
    /// Whether its `src/` constructs a fact.
    pub produces: bool,
    /// The strategies it declares, with the triple each declared.
    pub declarations: BTreeSet<Declaration>,
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

            produces = produces || Constructs(&text, &fact_type);
            declarations.extend(Declarations_In(&text));
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

/// Whether a source file builds the named type rather than mentioning it.
fn Constructs(text: &str, fact_type: &str) -> bool
{
    for line in text.lines()
    {
        let trimmed = line.trim();

        // A comment naming the type describes a producer rather than being one, which is
        // the distinction that made two rows of the corpus-gate table wrong before
        // anything checked them.
        if trimmed.starts_with("//")
        {
            continue;
        }

        if trimmed.contains(&format!("{fact_type} {{")) && !trimmed.contains("struct ")
        {
            return true;
        }
    }

    return false;
}

/// Every `impl Strategy for` block in one file, with the triple each declares.
///
/// A block that names fewer than three constants is still reported, with the missing axes
/// empty. Rust will not compile such an implementation, so it cannot reach a green build —
/// but reporting it as *absent* would let a syntax this scanner failed to read look exactly
/// like a crate that declared nothing, and those two must not be the same answer.
fn Declarations_In(text: &str) -> Vec<Declaration>
{
    let strategy_impl = Strategy_Impl();
    let mut found: Vec<Declaration> = Vec::new();

    for line in text.lines()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("//")
        {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix(&strategy_impl)
        {
            if let Some(name) = rest.split_whitespace().next()
            {
                found.push(Declaration {
                    strategy: name.to_owned(),
                    strength: String::new(),
                    scope: String::new(),
                    trace: String::new(),
                });
            }
            continue;
        }

        // The constants follow the `impl` line, so they belong to the most recent one.
        // A file with no `impl Strategy` in it has nothing to attach them to and the
        // `else` below drops them, which is right: `const STRENGTH` in the contracts
        // crate's own trait definition is the declaration of an obligation, not one.
        let Some(last) = found.last_mut()
        else
        {
            continue;
        };

        if let Some(value) = Constant_Value(trimmed, "STRENGTH")
        {
            last.strength = value;
        }
        else if let Some(value) = Constant_Value(trimmed, "SCOPE")
        {
            last.scope = value;
        }
        else if let Some(value) = Constant_Value(trimmed, "TRACE")
        {
            last.trace = value;
        }
    }

    return found;
}

/// The variant a `const NAME: Type = Type::Variant;` line assigns.
fn Constant_Value(line: &str, name: &str) -> Option<String>
{
    let rest = line.strip_prefix(&format!("const {name}:"))?;
    let (_, value) = rest.split_once('=')?;
    let (_, variant) = value.trim().trim_end_matches(';').rsplit_once("::")?;

    return Some(variant.trim().to_owned());
}

/// One row of the domain table in `nomos-contracts`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DomainRow
{
    /// The domain as the table names it, for diagnostics.
    pub domain: String,
    /// The strength the row claims, without its backticks.
    pub strength: String,
    /// The scope the row claims.
    pub scope: String,
    /// The trace equivalence the row claims.
    pub trace: String,
}

impl DomainRow
{
    /// Whether the row promises anything at all.
    ///
    /// The `None` row promises nothing by design — it is what keeps determinism affordable,
    /// because the CLI, the reporting layer and the agent host pay nothing for it. A row
    /// that promises nothing owes no implementation, so a completeness guard that demanded
    /// one would be demanding a declaration that says "no promise" for every part of the
    /// system that was never going to make one.
    #[must_use]
    pub fn Claims_Reproducibility(&self) -> bool
    {
        return self.strength != "None";
    }
}

/// Where the table that says which domain claims what is written.
///
/// The module documentation of `nomos-contracts`, read as text. It is the authority: every
/// peer that reimplements these types reads that table, so a row there is a published claim
/// whether or not anything in this workspace occupies it.
#[must_use]
fn Domain_Table_Path() -> Option<PathBuf>
{
    let workspace = Workspace::Load();
    let contracts = workspace.Get("nomos-contracts")?;

    return Some(contracts.root.join("src/determinism/mod.rs"));
}

/// Every row of the domain table, derived from the contracts crate's own source.
///
/// Returns an empty vector when the file cannot be read or holds no table, which every
/// caller must treat as a failure rather than as "there are no rows" — a completeness guard
/// quantifying over nothing passes having read nothing, and that is the shape this whole
/// directory exists to refuse.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Domain_Table() -> Vec<DomainRow>
{
    let Some(path) = Domain_Table_Path()
    else
    {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path)
    else
    {
        return Vec::new();
    };

    let mut rows = Vec::new();

    for line in text.lines()
    {
        let Some(row) = line.trim().strip_prefix("//! |")
        else
        {
            continue;
        };

        let cells: Vec<&str> = row.trim_end_matches('|').split('|').map(str::trim).collect();
        let [domain, strength, scope, trace] = cells.as_slice()
        else
        {
            continue;
        };

        let row = DomainRow {
            domain: (*domain).to_owned(),
            strength: Unquoted(strength),
            scope: Unquoted(scope),
            trace: Unquoted(trace),
        };

        // The header row and the `|---|` rule under it are table syntax rather than
        // content, and both survive the cell split. A row whose axes are not written as
        // code spans is one of those two.
        if row.strength.is_empty() || strength.len() == row.strength.len()
        {
            continue;
        }

        rows.push(row);
    }

    return rows;
}

/// A markdown code span with its backticks removed.
fn Unquoted(cell: &str) -> String
{
    return cell.trim_matches('`').trim().to_owned();
}

/// The strategies the integration harness actually holds to their declarations.
///
/// Read from the harness's own source, by the turbofish it registers a domain with. A
/// declaration this set does not contain is a promise with no test behind it, and a member
/// of this set that no crate declares is a test measuring something that no longer exists —
/// both directions matter, and neither can be seen from inside the harness, which is why
/// the observer reads it from outside.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Harnessed_Strategies() -> BTreeSet<String>
{
    let workspace = Workspace::Load();
    let mut found = BTreeSet::new();

    let Some(harness) = workspace.Get("nomos-integration-tests")
    else
    {
        return found;
    };

    let tests = harness.root.join("tests");
    if !tests.is_dir()
    {
        return found;
    }

    for file in Source_Files(&tests)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };

        found.extend(Registered_In(&text));
    }

    return found;
}

/// Every `Check::<Strategy>` in one file.
///
/// Assembled from its parts for the reason [`Fact_Type`] is: this file would otherwise
/// find itself if the harness and the observer ever shared a directory, and the two have
/// no rule keeping them apart.
fn Registered_In(text: &str) -> Vec<String>
{
    let marker = concat!("Check", "::<");
    let mut found = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find(marker)
    {
        let Some(after) = rest.get(start.saturating_add(marker.len())..)
        else
        {
            break;
        };

        let Some((name, remainder)) = after.split_once('>')
        else
        {
            break;
        };

        let name = name.trim();
        // A single-letter name is the generic parameter of the registering function itself,
        // not a domain. Reporting `S` as a strategy would put a name in the harnessed set
        // that no crate can ever declare.
        if name.len() > 1 && name.chars().all(|letter| return letter.is_alphanumeric())
        {
            found.push(name.to_owned());
        }

        rest = remainder;
    }

    return found;
}
