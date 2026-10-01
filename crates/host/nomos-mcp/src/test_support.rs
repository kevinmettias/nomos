//! What more than one test module here needs, and nothing else.

use serde_json::Value;

/// The value at `pointer` inside `value`, or null if nothing is there.
///
/// `serde_json`'s own `Index` implementation reads better and is not available: this
/// workspace denies `clippy::indexing_slicing`, and `Cargo.toml`'s own table carves out no
/// exception for tests the way `clippy.toml` does for `unwrap`. A JSON pointer asks the same
/// question and cannot panic, so the assertion that reads a field of an answer fails on the
/// field being wrong rather than on the field being absent.
pub(crate) fn Field_At(value: &Value, pointer: &str) -> Value
{
    return value.pointer(pointer).cloned().unwrap_or(Value::Null);
}
