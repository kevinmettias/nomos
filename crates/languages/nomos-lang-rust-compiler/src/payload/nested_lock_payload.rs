//! The whole analyzed crate's own set of nested-lock findings.

use super::nested_lock_finding::NestedLockFinding;

/// The whole analyzed crate's own set of nested-lock findings -- empty when this
/// provider's analysis found none, the same "clean is a real answer, not an absence"
/// shape [`crate::payload::clone_on_copy_payload::CloneOnCopyPayload`] already has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedLockPayload
{
    pub findings: Vec<NestedLockFinding>,
}
