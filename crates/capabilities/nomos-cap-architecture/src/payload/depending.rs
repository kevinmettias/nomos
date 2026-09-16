//! The name on the depending side of a directed relation.

/// The name that would do the depending: a newtype rather than the bare `&str` it wraps,
/// because [`crate::ArchitecturePayload::Permits`] and [`crate::ArchitecturePayload::Excepts`]
/// take this beside a [`crate::Depended`], and both ends are strings, so two positions of one
/// type would let a caller describe the reverse edge and nothing would say so.
///
/// The direction is the whole content of the question. That a composition root may name a
/// provider is not that the provider may name the composition root, and a query read in both
/// directions would admit exactly the cycles a component forbidding its own members exists to
/// prevent -- the argument [`crate::Exception`] makes for staying a separate type from
/// [`crate::Permission`], applied one level down, to the two ends of a single statement
/// rather than to two kinds of statement.
pub struct Depending<'a>(pub &'a str);
