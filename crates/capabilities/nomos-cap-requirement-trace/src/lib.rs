//! Capability Contract zone -- `nomos.cap.requirement.trace`'s contract, and its one
//! provider, over this repository's own committed requirement assessments under
//! `tests/contract/requirements/`.
//!
//! # What this promotes, and from where
//!
//! `P42-REQUIREMENT-TRACE-STALENESS-RULE-2`'s own job: `tests/contract/tests/
//! requirement_trace/{assessment,registry,predicates}.rs` already correctly parsed an
//! entry, named its four verdicts and `OD-TRACE-003`'s fifth, and compared a committed
//! assessment's sites, gaps and records against the real tree — proven by
//! `tests/contract/tests/requirement_trace/controls.rs`'s own negative controls and
//! asserted against this repository's real committed corpus by `.../committed.rs`. None of
//! that logic is reinvented here: [`assessment`], [`registry`] and [`predicates`] are that
//! logic, moved, with every `std::fs` call turned into a [`nomos_platform::FileSystem`] port
//! call so this crate is reachable from a real, gate-composed rule and not only from a test
//! binary that always runs against the real disk.
//!
//! # Why this is a new capability, not a repository-policy sibling
//!
//! `nomos-repo-policy`'s five `nomos.cap.*.policy` providers all read one file
//! (`standards.json`) and hand a rule the *raw declaration*, because the rule that reads
//! each one can finish the judgment itself from facts already in hand (source text, or
//! nothing at all). This capability cannot work that way: telling a resolved citation from
//! a stale one means reading arbitrary files this crate's own [`registry::Entries`] does not
//! already have — the site a `.assessment` file names, the record registration a match
//! names, the `docs/records/` listing a match's document lives under — none of which a rule
//! in Rules zone can reach, since `nomos-rules` has no [`nomos_platform::FileSystem`] of its
//! own and Rules zone may not depend on Provider zone at all. So the fact this capability
//! answers is the *already-judged* comparison ([`payload::Problem`], one per stale or
//! incomplete citation), not the raw corpus — the same "the provider already decided"
//! relay shape `nomos.cap.lint.diagnostics` and `nomos.cap.review.finding` already use, not
//! `nomos.cap.goals.policy`'s "the rule still judges" shape.
//!
//! # Why this crate bundles its own provider rather than waiting beside it
//!
//! [`contract`]'s own module doc carries the full reasoning: `OD-CAPABILITY-002` licenses
//! bundling contract and provider in one crate while there is exactly one provider, the
//! shape `nomos-connector-coderabbit` already took for `nomos.cap.review.finding`. Reading
//! arbitrary files across the repository tree needs a [`nomos_platform::FileSystem`], which
//! `crates/rules/nomos-rules/src/checks/dependency/zones.rs`'s own `Permits` forbids Rules
//! zone from reaching except through a Capability Contract zone crate — the same constraint
//! every `nomos.cap.*.policy` provider is built around, applied here to a single-provider
//! capability instead of a five-provider family.
//!
//! # What is here
//!
//! The capability's identity, contract version, ceiling and summary ([`contract`]); its one
//! provider's own name and offer ([`guarantee`]); the canonical `Assessment`/`Verdict`/
//! `Site` vocabulary ([`assessment`]); the refusing `key: value` reader
//! ([`registry`]); the five comparisons against the real tree ([`predicates`]); the
//! payload shape and its canonical encoding ([`payload`]); and the one function that reads
//! `root`'s own committed corpus and materializes this capability's one fact
//! ([`provider`]).
//!
//! What is not here: the rule that reads this capability (`nomos-rules::
//! Check_Requirement_Trace_Staleness`) and the composition wiring that runs this provider
//! in a real `nomos check` (`nomos-check-orchestration`'s own `RequiredFact::
//! RequirementTrace` and `Materialize_Requirement_Trace_Section`). Both are this decision's
//! own next-door territory, not this crate's.

#![forbid(unsafe_code)]

mod assessment;
mod contract;
mod guarantee;
mod payload;
mod predicates;
mod provider;
mod registry;
mod requirement_trace_fact_production;

pub use assessment::{Assessment, Site, Verdict, REGISTRY};
pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use payload::{Encode_Payload, Parse_Payload, Problem, ProblemKind, Refusal, RequirementTracePayload};
pub use predicates::{Divergences_With_No_Record, Partials_With_No_Gap, Unresolved_Gaps, Unresolved_Records, Unresolved_Sites};
pub use provider::{Discover_Workspace, FactContext, Materialize_Workspace, TraceFact};
pub use registry::{Entries, Is_Requirement_Id, Parse};
pub use requirement_trace_fact_production::RequirementTraceFactProduction;
