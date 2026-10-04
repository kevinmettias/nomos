//! Reading one Go file for `nomos.cap.syntax.sites`, the family `OD-CAPABILITY-019` placed beside
//! `nomos.cap.syntax.items`.
//!
//! One kind so far, the labeled jump, read from `tree-sitter-go`'s tree. It is the corpus's Go
//! kernel for `labeledloop` (`go_labeled_loop.go` at code-standards `f0d820729`) re-expressed over
//! tree-sitter rather than `go/ast`: every `break` or `continue` naming a label is resolved against
//! the constructs enclosing it and recorded only when the label resolves to a loop.
//!
//! What makes this more than the Rust walk is Go's binding rule for a bare `break`: a `switch`, a
//! type switch or a `select` catches it before the loop does, so a label naming the innermost loop
//! can still be the only way out of it. Each enclosing construct is therefore remembered both as a
//! loop or not and as catching a bare `break` or not, and the two truth values a site carries are
//! computed from that, exactly as the corpus kernel computes them.

mod guarantee;
mod provider;
mod walk;

pub use guarantee::{Declared_Guarantee, Provider_Offer};
pub use provider::Materialize_Sites_Fact;

use crate::ParseFailure;
use crate::syntax::{Parsed_Tree, Root_Error};
use nomos_cap_syntax::LabeledJump;

/// The result of reading one Go file for its sites -- the shape [`crate::Reading`] has, for the
/// reason it gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SitesReading
{
    Parsed(Vec<LabeledJump>),
    Unparseable(ParseFailure),
}

/// Reads Go source for the labeled jumps in it, in source order.
#[must_use]
pub fn Read_Sites(source: &str) -> SitesReading
{
    let tree = match Parsed_Tree(source)
    {
        Ok(tree) => tree,
        Err(failure) => return SitesReading::Unparseable(failure),
    };
    let root = tree.root_node();
    if let Some(failure) = Root_Error(root)
    {
        return SitesReading::Unparseable(failure);
    }

    let mut walk = walk::Walk::New(source.as_bytes());
    walk.Descend(root, &[], None);

    return SitesReading::Parsed(walk.jumps);
}
