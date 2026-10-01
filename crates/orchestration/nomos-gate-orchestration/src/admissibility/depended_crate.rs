//! The crate that would be named.

/// The crate that would be named: distinct from [`crate::DependingCrate`] for the reason the edge
/// is directed, since that a composition root may name a provider does not mean the provider may
/// name the composition root, so the two ends of the question must not be interchangeable.
pub struct DependedCrate<'a>(pub &'a str);
