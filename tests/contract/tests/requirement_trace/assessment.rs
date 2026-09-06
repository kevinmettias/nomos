//! What an assessment is: a verdict, the places it is about, and the record behind it.
//!
//! `P42-REQUIREMENT-TRACE-STALENESS-RULE-2` promoted this vocabulary into
//! `nomos_cap_requirement_trace`, the capability crate a real, gate-composed
//! `nomos-rules` rule now reads through — this module is a thin re-export of it, not a
//! second copy: two readers of one registry is how they come to disagree, the same reason
//! [`crate::registry`] and [`crate::predicates`] are thin wrappers below.

pub(crate) use nomos_cap_requirement_trace::{Assessment, Site, Verdict, REGISTRY};
