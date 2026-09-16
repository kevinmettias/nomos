//! The name on the depended side of a directed relation.

/// The name that would be depended upon: distinct from [`crate::Depending`] for the reason the
/// relation is directed, since a statement that `a` may name `b` says nothing about `b` naming
/// `a`, so the two ends of the question must not be interchangeable.
///
/// Read the other end's doc for why the distinction is a type rather than a convention here.
pub struct Depended<'a>(pub &'a str);
