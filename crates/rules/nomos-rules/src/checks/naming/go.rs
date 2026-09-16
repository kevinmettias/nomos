//! The Go language front ends of this module's naming rules, grouped apart from the ones that
//! judge every language alike.
//!
//! What holds these three together is what separates them: Go's own casing carries visibility,
//! so an exported constant, function or type is `Upper_Snake_Case` or `UpperCamelCase` where its
//! unexported twin is not. "How does this crate judge a Go name" is then one folder's question
//! rather than three entries in a flat list of eleven.
//!
//! Submodules are `pub(super)` and [`super`] re-exports each one's items, so grouping these files
//! moves no public path: `naming::CONSTANTS_SPLIT_BY_EXPORT` is still the name a caller uses, and
//! nothing here is reachable only through `naming::go`.

pub(super) mod data_names;
pub(super) mod function_names;
pub(super) mod type_names;
