//! Which crates produce facts, and what each declares about repeating itself.
//!
//! Derived from the source, never declared. A list of domains checked into this repository
//! would be right on the day it was written and wrong the first time somebody added one —
//! and a stale list understates the set, which is the direction that flatters: the crate
//! nobody added would be the crate nobody checked.
//!
//! The producer predicate is the construction of a [`nomos_analysis::MaterializedFact`],
//! which is the mechanical reading of "an execution domain that produces a fact" — and it
//! is deliberately a construction rather than a mention: the crate that *defines* the type
//! names it constantly and produces nothing.
//!
//! This predicate cannot see a domain that serializes or projects rather than producing,
//! which is how spec-bundle serialization and the projection engine sat undeclared for as
//! long as they did. [`crate::Domain_Table`] and [`crate::Harnessed_Strategies`] answer the
//! question from the other end; between the three, a row nobody occupies and a declaration
//! nobody checks are both visible without any list being kept by hand.
//!
//! This crate compiles against almost nothing on purpose, so this is a scan of text rather
//! than reflection over types. That is the same trade `gates.rs` makes, for the same reason —
//! an observer that linked the thing it observes would be a participant.

use crate::reading::source_files::{Source_Files, Without_Test_Modules};
use crate::{Declaration, Workspace};
use std::collections::BTreeSet;
use std::path::Path;

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
        if Measures_Rather_Than_Declares(&member.name)
        {
            continue;
        }
        let domain = Domain_Of(&member.name, &member.root, &fact_type);

        found.extend(domain);
    }

    found.sort();
    return found;
}

/// The two test crates are excluded, and this is the one exclusion here.
///
/// `tests/integration` does construct facts — `src/slice.rs` builds them to drive the slice —
/// and it is not an execution domain. It is the instrument that measures them, and a
/// thermometer does not have a temperature to declare. The same holds for this crate, which
/// names the type in order to search for it.
///
/// Stated as a rule about what these crates *are* rather than as two names on a list, because
/// `docs/records/OD-GATE-001` records what an exemption list does to a check: it becomes the
/// easiest place to hide the thing the check exists to find.
fn Measures_Rather_Than_Declares(name: &str) -> bool
{
    return name.starts_with("nomos-integration-tests") || name.starts_with("nomos-contract-tests");
}

/// One crate's domain, if it produces facts or declares a strategy.
fn Domain_Of(name: &str, root: &Path, fact_type: &str) -> Option<FactDomain>
{
    // A strategy declared inside a unit-test module is an example of the trait, not a domain
    // occupying a row of the table. `nomos-contracts` has two.
    let sources: Vec<String> = Source_Files(&root.join("src"))
        .iter()
        .filter_map(|file| return std::fs::read_to_string(file).ok())
        .map(|text| return Without_Test_Modules(&text))
        .filter(|text| return Speaks_This_Workspaces_Vocabulary(text))
        .collect();
    let produces = sources.iter().any(|text| return Constructs(text, fact_type));
    let declarations: BTreeSet<Declaration> = sources
        .iter()
        .flat_map(|text| return Declarations_In(text))
        .filter(|declaration| return !Forwards_Its_Axes(declaration))
        .collect();
    if !produces && declarations.is_empty()
    {
        return None;
    }

    return Some(FactDomain {
        crate_name: name.to_owned(),
        produces,
        declarations,
    });
}

/// Whether a declaration inherits all three axes from another type rather than naming any.
///
/// `impl<Referent: Strategy> Strategy for &Referent` writes `Referent::STRENGTH` into each
/// axis, so the value this scanner reads back is the axis's own name. That is a forwarding
/// implementation -- a reference promises exactly what it refers to -- and it is not a
/// domain: it occupies no row, owes no harness test, and registering one would be
/// registering the language's own indirection.
///
/// The same distinction this module already draws for a declaration inside a unit-test
/// module, which is an example of the trait rather than a domain occupying a row.
fn Forwards_Its_Axes(declaration: &Declaration) -> bool
{
    return declaration.strength == "STRENGTH"
        && declaration.scope == "SCOPE"
        && declaration.trace == "TRACE";
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

/// Whether a file's `Strategy` is the one this table is about.
///
/// `Strategy` is a trait name two vocabularies share. This workspace's is
/// `nomos_contracts::Strategy`, and the rows of the domain table are its rows. The sibling
/// engine has a trait of the same name and the same three axes, and since 2026-09-10 this
/// workspace implements it in several places -- `nomos-platform-xvpe`'s launcher bridge,
/// and the three host adapters that serve a protocol -- because the engine's own surfaces
/// require it of anything plugged into them.
///
/// Those are real declarations and they are held to something; they are simply held to it
/// *there*, by the engine's own suites, against the engine's own table. Counting them here
/// would demand that `tests/integration` discharge a promise made in another workspace's
/// vocabulary, against a harness whose `Assert_Meets_Declared_Strategy` is generic over
/// this workspace's trait and could not accept them.
///
/// Read from the file's own imports rather than from a list of type names: a list would
/// have to be edited every time an adapter arrives, and the edit that was forgotten would
/// look exactly like a domain that was never declared.
fn Speaks_This_Workspaces_Vocabulary(text: &str) -> bool
{
    for line in text.lines()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("use xvpe_primitives::") && trimmed.contains("Strategy")
        {
            return false;
        }
    }

    return true;
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
        Read_One_Line(line.trim(), &strategy_impl, &mut found);
    }

    return found;
}

/// One line: an `impl Strategy for` opening a declaration, a constant filling in an axis of
/// the most recent one, or neither.
fn Read_One_Line(trimmed: &str, strategy_impl: &str, found: &mut Vec<Declaration>)
{
    if trimmed.starts_with("//")
    {
        return;
    }
    if let Some(rest) = Implemented_Type(trimmed, strategy_impl)
    {
        Open_A_Declaration(rest, found);
    }
    else
    {
        Fill_In_An_Axis(trimmed, found);
    }
}

/// What follows `Strategy for` on a line that opens a declaration, or `None`.
///
/// Two spellings reach this, and only the first used to: the bare `impl Strategy for X`,
/// and the generic `impl<T: Bound> Strategy for X<'_, T>`. A prefix match saw only the
/// bare one, so a generic declaration was invisible to every assertion in this file --
/// found 2026-09-11, when a bare one arrived in a crate that already had a generic one
/// nothing had ever reported.
fn Implemented_Type<'line>(trimmed: &'line str, strategy_impl: &str) -> Option<&'line str>
{
    if !trimmed.starts_with("impl")
    {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix(strategy_impl)
    {
        return Some(rest);
    }

    // The generic form. The marker is assembled rather than written out for the reason
    // `Strategy_Impl` is: this file must not match itself.
    let infix = concat!(" Strategy ", "for ");

    return trimmed.split_once(infix).map(|(_generics, rest)| return rest);
}

/// A new declaration, with every axis still empty.
fn Open_A_Declaration(rest: &str, found: &mut Vec<Declaration>)
{
    let Some(name) = rest.split_whitespace().next()
    else
    {
        return;
    };

    // `XvpeLauncher<'_, Launcher>` is the same declaration as `XvpeLauncher`: the type's
    // own parameters are not part of the name the harness registers it under.
    let name = name.split('<').next().unwrap_or(name);

    found.push(Declaration {
        strategy: name.to_owned(),
        strength: String::new(),
        scope: String::new(),
        trace: String::new(),
    });
}

/// The constants follow the `impl` line, so they belong to the most recent declaration.
///
/// A file with no `impl Strategy` in it has nothing to attach them to and this drops them,
/// which is right: `const STRENGTH` in the contracts crate's own trait definition is the
/// declaration of an obligation, not one.
fn Fill_In_An_Axis(trimmed: &str, found: &mut [Declaration])
{
    let Some(last) = found.last_mut()
    else
    {
        return;
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

/// The variant a `const NAME: Type = Type::Variant;` line assigns.
fn Constant_Value(line: &str, name: &str) -> Option<String>
{
    let rest = line.strip_prefix(&format!("const {name}:"))?;
    let (_, value) = rest.split_once('=')?;
    let (_, variant) = value.trim().trim_end_matches(';').rsplit_once("::")?;

    return Some(variant.trim().to_owned());
}
