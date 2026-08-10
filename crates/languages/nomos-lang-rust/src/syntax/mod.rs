//! Reading one file, and what comes back when that fails.

mod walk;
mod shape;
mod documentation;
#[cfg(test)]
mod tests;

use walk::Walk;
pub(crate) use shape::Path_As_Written;
use shape::{Bound_By, Function_Shape, Impl_Shape, Type_Head, Type_Shape};
use documentation::Documentation;

use crate::item_kind::ItemKind;
use crate::visibility::Visibility;
use crate::syntax_item::SyntaxItem;
use crate::syntax_facts::SyntaxFacts;
use crate::parse_failure::ParseFailure;
use crate::reading::Reading;
use syn::visit::Visit;

/// Reads Rust source.
///
/// Takes the text, not a path. The signature is where
/// [`nomos_contracts::IncrementalGranularity::File`] stops being a claim: this function
/// has no way to reach a second file, so a fact it produces cannot depend on one, and a
/// change elsewhere cannot invalidate it.
#[must_use]
pub fn Read_Source(source: &str) -> Reading
{
    let file = match syn::parse_file(source)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let at = error.span().start();

            return Reading::Unparseable(ParseFailure {
                line: at.line,
                column: at.column,
                message: error.to_string(),
            });
        }
    };

    let mut walk = Walk::New();
    walk.visit_file(&file);

    return Reading::Parsed(SyntaxFacts {
        items: walk.items,
        unexpanded: walk.unexpanded,
    });
}
