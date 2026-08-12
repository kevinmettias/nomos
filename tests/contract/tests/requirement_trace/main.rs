//! Which corpus requirements this build has been assessed against, and whether the
//! assessments still resolve.
//!
//! `OD-TRACE-001` measured the hole this closes. The v14 corpus carries 363 requirements at
//! `authority: canonical-normative-record`, fourteen are named anywhere in this repository,
//! and — the finding that made it a defect rather than a fact — **met and unmet have the
//! same shape from outside.** An identifier that appears nowhere, beside code that may or
//! may not honour it, reads identically whether somebody checked or nobody did. Six hand
//! comparisons cost a full manual audit each and three came back met, which is the worst
//! ratio a hand check can have: most of the work bought no change.
//!
//! An assessment is therefore **a declared entry committed to this repository**, compared
//! against the workspace by this suite, and never derived from the corpus at check time.
//! `OD-GATE-001` is what forces that: a test that cannot find its corpus returns early and
//! prints `ok`, CI has none of the three corpora, and a traceability surface computed from
//! the corpus would be green on every pull request by being unable to look — green about
//! the *entire* corpus, which is a far larger lie than the sixty-eight tests that hole
//! already covers. `OD-TRACE-002` records why no corpus comparison is added here at all,
//! which is a stronger statement than "not yet".
//!
//! # What the guard can and cannot see
//!
//! It sees that a named site has vanished. It cannot see that the code at that site drifted
//! out of satisfying the requirement while the path stayed valid. Semantic drift is meaning
//! and this workspace has no type for it, so a stale `Met` is wrong in one requirement —
//! against a status quo that is unreadable in all 363. `OD-TRACE-001` takes that trade
//! explicitly and this suite does not re-argue it.
//!
//! # Unassessed is a state, not a gap
//!
//! Absence of an entry means nobody looked, and that is the honest description of most of
//! the corpus today. So the registry is **a floor rather than a count**, the shape
//! `OD-SPEC-007` already chose for governing records: entries may be added freely, and
//! [`FEWEST_ASSESSMENTS`] is what a deletion has to walk past.
//!
//! [`FEWEST_ASSESSMENTS`]: crate::committed::FEWEST_ASSESSMENTS


mod committed;
mod common;
mod controls;
mod predicates;
mod reader;
mod registry;
