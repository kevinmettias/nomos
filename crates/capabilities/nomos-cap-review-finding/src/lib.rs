//! Capability Contract zone -- `nomos.cap.review.finding`, the agreement itself, with no
//! provider beside it.
//!
//! One automated code review's own observed finding: a category, a severity, the file and
//! line it concerns, and the reviewer's own message, kept apart from any judgment about
//! whether the finding is valid or what it bears on. What is here is what the parties
//! agreed -- the capability's identity, the contract version, the ceiling, the summary
//! that says what the question means, the schema every answer is stamped with, and the
//! codec that writes and reads that schema. What is not here is anything a provider claims
//! for itself: a `ProviderId` is a provider's own name and a `Guarantee` is its own claim,
//! and both stay in the crate that makes them, which is `OD-CAPABILITY-002`'s division.
//!
//! # Why this is its own crate, when `OD-CAPABILITY-002`'s criterion has not fired
//!
//! It has not: `nomos.cap.review.finding` still has exactly one provider, and that record's
//! criterion for a contract earning a crate is provider *contention*. `OD-CAPABILITY-017`
//! then measured the bundled arrangement directly and found it defeated no boundary --
//! `nomos-rules` called no provider-side symbol, and no platform implementation was
//! reachable from where a rule sits, so the provider's object code could not act from
//! there. Neither measurement has been overturned.
//!
//! What moved is a sequencing decision the owner made in `OD-ROADMAP-005`, decision 5: an
//! external architecture review objected that a generic rule layer naming
//! `nomos-connector-coderabbit` puts a vendor's name below the provider boundary, and the
//! owner required the split built now rather than at contention. So this crate exists
//! because a contract a generic consumer reads should not be spelled with a vendor's crate
//! name, not because a second provider arrived.
//!
//! # What that buys, concretely
//!
//! `nomos-rules` and `nomos-check-orchestration` name this crate. Neither names
//! `nomos-connector-coderabbit` for the contract, and `nomos-connector-coderabbit`
//! re-exports no part of it -- a re-export would leave the old spelling working and so
//! would preserve the exact defect this crate was built to remove. The vendor crate is
//! Provider zone now, which is what `OD-CAPABILITY-015`'s criterion says once it declares
//! no contract, and `Permits` forbids `Rules` from naming it at all. That prohibition is
//! the guard the bundled arrangement had to be measured into place by hand.

#![forbid(unsafe_code)]

mod contract;
#[path = "review_finding_id.rs"]
mod identity;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use identity::ReviewFindingId;
pub use payload::finding_payload::FindingPayload;
pub use payload::payload_refusal::PayloadRefusal;
pub use payload::{Encode_Payload, Parse_Payload};
