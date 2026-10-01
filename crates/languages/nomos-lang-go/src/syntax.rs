//! What one Go file says on its face — the walk over `tree-sitter-go`'s parse tree, and
//! the facts it produces.

mod documentation;
mod facts;
mod item;
mod item_kind;
#[cfg(test)]
mod tests;
mod visibility;
mod walk;

pub use facts::Facts;
pub use item::Item;
pub use item_kind::ItemKind;
pub use visibility::Visibility;
pub use walk::Read_Source;

use documentation::Documentation_Of_Declaration;
