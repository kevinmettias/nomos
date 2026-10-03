//! Reading one file for `nomos.cap.syntax.sites`, the family `OD-CAPABILITY-019` placed beside
//! `nomos.cap.syntax.items`.
//!
//! # What this provider offers
//!
//! One kind so far, the labeled jump, read by walking `syn`'s tree into function bodies -- the
//! descent `OD-CAPABILITY-011` declined for the items payload and the sites family takes up. It is
//! the corpus's Rust kernel for `labeledloop` (`rust_labeled_loop.go` at code-standards
//! `f0d820729`) re-expressed over `syn` rather than tree-sitter: every `break` or `continue` that
//! names a label is resolved against the constructs enclosing it, and recorded only when the label
//! resolves to a loop.
//!
//! What the payload states about the kinds this provider does not offer is nothing at all: a kind
//! declared after this one is a gap here until this provider offers it, which a rule reports as
//! `MissingCapability` -- the honest answer, and the reason a payload names the kinds it offers
//! rather than the reader assuming every declared kind was looked for.

mod guarantee;
mod provider;
mod walk;

pub use guarantee::{Declared_Guarantee, Provider_Offer};
pub use provider::Materialize_Sites_Fact;

use crate::ParseFailure;
use nomos_cap_syntax::LabeledJump;
use syn::visit::Visit;

/// The result of reading one file for its sites.
///
/// Mirrors [`crate::Reading`] for the reason that type gives: a `Result` would invite `.ok()`, and
/// a walk that maps a failure to no sites reports a clean file it never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SitesReading
{
    Parsed(Vec<LabeledJump>),
    Unparseable(ParseFailure),
}

/// Reads Rust source for the labeled jumps in it, in source order.
///
/// Takes the text, not a path, so [`nomos_contracts::IncrementalGranularity::File`] stays true by
/// construction: this function has no way to reach a second file.
#[must_use]
pub fn Read_Sites(source: &str) -> SitesReading
{
    let file = match syn::parse_file(source)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let at = error.span().start();
            return SitesReading::Unparseable(ParseFailure { line: at.line, column: at.column, message: error.to_string() });
        }
    };

    let mut walk = walk::Walk::New();
    walk.visit_file(&file);

    return SitesReading::Parsed(walk.jumps);
}
