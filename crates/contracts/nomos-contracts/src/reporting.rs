//! The vocabulary a build's report is made of: a rule's judgment about one subject, and
//! whether the mechanism that claims to enforce that rule is real.
//!
//! Grouped together because [`Finding::gate`] *is* an [`GateCategory`] — a finding is
//! where enforcement's claim and reality meet the subject it was raised about. `lib.rs`
//! names this pairing directly: enforcement together with the finding vocabulary it is
//! reported through. Determinism, guarantee, authority and peer stand alone because
//! nothing else in the crate depends on their fields the way a finding depends on the
//! gate.

mod enforcement;
mod finding;

pub use enforcement::{EnforcementBreach, EnforcementReach, EnforcerRef, GateCategory};
pub use finding::{Applicability, DisplayLabel, EvidenceClass, Finding};
