//! What one Go file says on its face — the walk over `tree-sitter-go`'s parse tree, and
//! the facts it produces.

mod documentation;
#[path = "syntax_facts.rs"]
mod facts;
#[path = "syntax_item.rs"]
mod item;
mod item_kind;
#[cfg(test)]
mod tests;
mod visibility;
mod walk;

pub use facts::SyntaxFacts;
pub use item::SyntaxItem;
pub use item_kind::ItemKind;
pub use visibility::Visibility;
pub use walk::Read_Source;

use documentation::Documentation_Of_Declaration;
