//! Provider zone -- the one provider of `nomos.cap.review.finding`
//! (`nomos-cap-review-finding`), over `CodeRabbit`'s own review comments as `gh api` reads
//! them.
//!
//! # Why the contract is one crate away
//!
//! It used to be here. `OD-CAPABILITY-002` licenses a contract living beside its only
//! provider until a second one contends for it, and `OD-CAPABILITY-017` measured that the
//! bundled arrangement defeated no boundary: `nomos-rules` called no provider-side symbol
//! of this crate, and no platform implementation was reachable from where a rule sits, so
//! the `gh api` machinery linked beside a rule could not act. Both findings stand.
//!
//! `OD-ROADMAP-005`'s fifth decision superseded the conclusion on a different ground. An
//! external architecture review objected that `nomos-rules`, a generic rule layer, had to
//! name a *vendor* crate to obtain a generic capability, and the owner required the split
//! built now rather than at contention. So [`nomos_cap_review_finding`] holds the
//! agreement -- capability identity, contract version, ceiling, payload schema, payload
//! codec and payload types -- and this crate holds what a provider claims for itself: its
//! own name, its own guarantee, its own recorded fixture, its own determinism declaration
//! and the code that reaches GitHub.
//!
//! **Nothing of the contract is re-exported here.** A re-export would leave
//! `nomos_connector_coderabbit::Capability()` spelling correctly, so a consumer could keep
//! naming a vendor crate for a generic contract and the defect the split exists to remove
//! would survive it. Consumers name [`nomos_cap_review_finding`] directly, exactly as
//! `nomos-lang-rust-clippy`'s consumers name `nomos-cap-lint`.
//!
//! This crate is Provider zone for the same reason: `OD-CAPABILITY-015`'s criterion is what
//! a crate *declares*, and this one declares no capability contract any more. `Permits`
//! forbids `Rules` from naming `Provider`, so the prohibition `OD-CAPABILITY-017` had to
//! establish by measurement is now the ordinary declared edge.
//!
//! # The seam `ARC-CONNECTOR-001` draws, inside one crate
//!
//! [`fetching::Fetch_Review_Comment`] is the generic-connector-substrate side: it runs `gh
//! api repos/{repository}/pulls/comments/{id}` through a caller-supplied
//! [`nomos_platform::ProgramLauncher`] and hands back GitHub's own bytes, unread.
//! [`translation::Translate_Review_Comment`] is the vendor-to-canonical side: a pure
//! function from those bytes to the contract's own
//! [`nomos_cap_review_finding::FindingPayload`], naming no vendor type in its return shape
//! and refusing rather than guessing at a comment-body convention it was not written
//! against. [`identity::Review_Comment_Identity`] is this crate's own mint, which travelled
//! with the provider rather than with the payload type it returns, because GitHub's
//! addressing scheme is this provider's knowledge and not the agreement's.
//! [`provider::Materialize_Review_Comment`] composes them into the one fact
//! `nomos_cap_review_finding::Ceiling`'s `IncrementalGranularity::None` allows per finding.
//! This split is what lets [`fixture::Sample_Review_Comment_Response`] -- this crate's one
//! recorded fixture, per `OD-CONNECTOR-002` -- stand in for a live call on every replay:
//! [`translation::Translate_Review_Comment`] reads it exactly as it would read
//! [`fetching::Fetch_Review_Comment`]'s own output.
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

#[path = "review_finding_production.rs"]
mod determinism;
mod fetching;
mod fixture;
mod guarantee;
#[path = "review_comment_identity.rs"]
mod identity;
mod provider;
mod translation;

pub use determinism::ReviewFindingProduction;
pub use fetching::{Fetch_Review_Comment, FetchError};
pub use fixture::Sample_Review_Comment_Response;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use identity::Review_Comment_Identity;
pub use provider::{ConnectorError, Fact_Of, FactContext, Materialize_Review_Comment, ReviewFindingFact};
pub use translation::{Translate_Review_Comment, TranslationError};
