//! The whole workspace's own set of policy violations.

use super::policy_violation::PolicyViolation;

/// The whole workspace's own set of policy violations — empty when the tool found none,
/// the same "clean is a real answer, not an absence" shape `nomos_cap_lint::
/// DiagnosticsPayload` already has for a member with no diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPayload
{
    pub violations: Vec<PolicyViolation>,
}
