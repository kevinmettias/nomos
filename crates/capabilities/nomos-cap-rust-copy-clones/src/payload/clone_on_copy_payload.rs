//! The whole analyzed crate's own set of clone-on-copy findings.

use super::cloned_copy_type::ClonedCopyType;

/// The whole analyzed crate's own set of clone-on-copy findings -- empty when the analysis
/// found none, the same "clean is a real answer, not an absence" shape
/// `nomos_cap_dependency_policy::PolicyPayload` already has for a workspace with no
/// violations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloneOnCopyPayload
{
    pub findings: Vec<ClonedCopyType>,
}
