//! Capability Contract zone -- `nomos.cap.review.finding`'s contract, and its one
//! provider, over `CodeRabbit`'s own review comments as `gh api` reads them.
//!
//! # Why this is a new capability, not a second offer against an artifact-shaped contract
//!
//! [`contract`]'s own module doc carries the full measurement; the summary is that a
//! `CodeRabbit` review comment answers a different question than an external artifact's
//! title, state, locator and kind. `ARC-CONNECTOR-001` named Jira, Confluence and
//! `SharePoint` as the connectors expected to contend for a shared artifact contract --
//! every one an issue tracker or a document store answering that exact question with a
//! different vendor's bytes. A review finding is evidence about a diff (a file, a line, a
//! severity, a category), not a record with a title and a lifecycle state, and
//! `OD-CAPABILITY-002`'s own criterion is contention on the same question, not
//! resemblance through a shared connector shape. [`contract`], [`guarantee`], [`fetching`]
//! and [`translation`] are this crate's one offer against it.
//!
//! # The seam `ARC-CONNECTOR-001` draws, inside one crate
//!
//! [`fetching::Fetch_Review_Comment`] is the generic-connector-substrate side: it runs `gh
//! api repos/{repository}/pulls/comments/{id}` through a caller-supplied
//! [`nomos_platform::ProcessLauncher`] and hands back GitHub's own bytes, unread.
//! [`translation::Translate_Review_Comment`] is the vendor-to-canonical side: a pure
//! function from those bytes to this crate's own [`payload::finding_payload::
//! FindingPayload`], naming no vendor type in its return shape and refusing rather than
//! guessing at a comment-body convention it was not written against.
//! [`provider::Materialize_Review_Comment`] composes the two into the one fact
//! `contract::Ceiling`'s `IncrementalGranularity::None` allows per finding. This split is
//! what lets [`fixture::Sample_Review_Comment_Response`] -- this crate's one recorded
//! fixture, per `OD-CONNECTOR-002` -- stand in for a live call on every replay:
//! [`translation::Translate_Review_Comment`] reads it exactly as it would read
//! [`fetching::Fetch_Review_Comment`]'s own output.
//!
//! # Why this crate is Capability Contract zone, not Provider zone
//!
//! Every other real `ToolProvider` in this workspace splits its contract into its own
//! `nomos-cap-*` crate at Capability Contract zone specifically so `nomos-rules` (Rules
//! zone) can depend on the contract without depending on the Provider-zone crate that
//! performs the I/O -- `crates/rules/nomos-rules/src/checks/dependency/zones.rs`'s own
//! `Permits` forbids `Rules` from naming `Provider` at all. `OD-CAPABILITY-002` licenses
//! bundling contract and provider in one crate while there is only one provider; this
//! crate does exactly that, so it is classified Capability Contract zone rather than
//! Provider zone -- Provider zone would leave `nomos.cap.review.finding` structurally
//! unreachable by any rule, not merely undesirable.
//!
//! # What this crate does not do
//!
//! It does not write into any fact store -- a composition root calls
//! [`provider::Materialize_Review_Comment`] and writes the [`provider::ReviewFindingFact`]
//! it returns, the same division this workspace's other real providers draw. It does not
//! offer a write capability -- `OD-CONNECTOR-001`'s decision, held by omission: no
//! function here accepts anything to write back to GitHub or to `CodeRabbit`. It does not
//! decide a bears-on relation between a finding and any Nomos subject -- that is a future
//! connector's own work, the identical non-goal `ARC-CONNECTOR-001` already leaves
//! standing. It does not run `CodeRabbit`'s own CLI (`coderabbit review`) -- that tool
//! produces a *fresh* local review over uncommitted changes, an action this connector's
//! whole shape (`ARC-CONNECTOR-001`'s observation-only interface) has no room for; this
//! crate reads a review `CodeRabbit` has already posted, never one it performs itself.

#![forbid(unsafe_code)]

mod contract;
#[path = "review_finding_production.rs"]
mod determinism;
mod fetching;
mod fixture;
mod guarantee;
#[path = "review_finding_id.rs"]
mod identity;
mod payload;
mod provider;
mod translation;

pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use determinism::ReviewFindingProduction;
pub use fetching::{Fetch_Review_Comment, FetchError};
pub use fixture::Sample_Review_Comment_Response;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use identity::ReviewFindingId;
pub use payload::finding_payload::FindingPayload;
pub use payload::payload_refusal::PayloadRefusal;
pub use payload::{Encode_Payload, Parse_Payload};
pub use provider::{ConnectorError, Fact_Of, FactContext, Materialize_Review_Comment, ReviewFindingFact};
pub use translation::{Translate_Review_Comment, TranslationError};
