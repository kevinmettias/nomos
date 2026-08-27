//! One row of a rendered projection.

mod row;
mod value;

pub use row::Item;
pub use value::Value;

/// A field name given to `Item::With`, kept distinct from `Value` so the two positions
/// cannot be swapped at a call site — both are plain strings and nothing else would tell
/// them apart.
pub struct Name<'a>(pub &'a str);
