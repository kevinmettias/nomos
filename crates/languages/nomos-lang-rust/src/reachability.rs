//! Reading one file for `nomos.cap.controlflow.reachability`, this crate's second
//! capability alongside `nomos.cap.syntax.items`.
//!
//! # Scope, narrower than the capability's own ceiling
//!
//! This is a `Syntactic`-tier reading: pattern-matching over one file's parse tree, no
//! name or type resolution. It looks for exactly one shape — a `match` arm whose pattern
//! is `Err(applicability)`, the binding name `crates/rules/nomos-rules` already uses three
//! times for a fact-read failure — and records it only when the arm's body is one of four
//! syntactically obvious wrong shapes: empty, a bare `continue`, a bare `return`, or a
//! tail call to `Ok(...)`. Every other shape is not recorded, which is not a claim that it
//! is correct — see [`crate::reachability::ArmShape`]'s own doc.
//!
//! Two things this reading does not attempt, stated rather than discovered later: an
//! `if let Err(applicability) = ...` outside a `match` is not walked, and a binding
//! spelled anything but `applicability` is not recognised at all. Both are real gaps a
//! sound, `SemanticallyResolved` provider would need to close — `OD-RULES-008`'s own tier
//! 2 — and neither is silently claimed here.

mod guarantee;
mod provider;
mod walk;

pub use guarantee::{Declared_Guarantee, Provider_Offer};
pub use provider::Materialize;

use crate::ParseFailure;
use nomos_cap_controlflow::ReachabilitySite;
use syn::visit::Visit;

/// The result of reading one file for reachability sites.
///
/// Mirrors [`crate::Reading`] deliberately, for the same reason: a `Result` would invite
/// `.ok()`, and a walk that maps failures to `None` and counts the sites it found is
/// exactly the shape of a run that reports a clean corpus it never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReachabilityReading
{
    Parsed(Vec<ReachabilitySite>),
    Unparseable(ParseFailure),
}

/// Reads Rust source for reachability sites.
///
/// Takes the text, not a path, for the identical reason [`crate::Read_Source`] does: this
/// function has no way to reach a second file, so [`nomos_contracts::IncrementalGranularity::File`]
/// stays true rather than asserted.
#[must_use]
pub fn Read_Reachability(source: &str) -> ReachabilityReading
{
    use walk::Walk;

    let file = match syn::parse_file(source)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let at = error.span().start();

            return ReachabilityReading::Unparseable(ParseFailure {
                line: at.line,
                column: at.column,
                message: error.to_string(),
            });
        }
    };

    let mut walk = Walk::New();
    walk.visit_file(&file);

    return ReachabilityReading::Parsed(walk.sites);
}
