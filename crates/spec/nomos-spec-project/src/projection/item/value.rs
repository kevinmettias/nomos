//! The value half of a projection item's field pair.

/// A field value given to `Item::With`, kept distinct from `Name` for the same reason.
pub struct Value<'a>(pub &'a str);
