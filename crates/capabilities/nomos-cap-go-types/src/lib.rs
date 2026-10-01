//! Zone: Capability Contract -- the `nomos.cap.go.discarded_values` contract: every value a Go file
//! assigns to the blank identifier, with its resolved type and whether that type is `error`.
//!
//! # Why this question, and only this one
//!
//! `nomos-rules`' `a-discarded-error-is-explained` judges a discarded error, and reading Go on its
//! face it cannot tell an error from anything else: it reports every `_ = f()` -- a discarded
//! `fmt.Sprintf` string among them -- and sees neither `n, _ := strconv.Atoi(s)` nor
//! `_, _ = w.Write(b)`, the forms most Go code discards an error in. Whether a discarded value is an
//! error is a question about its type, and `f` may be declared in another package, so it is a
//! question only the type checker answers. That is the trigger `OD-ROADMAP-004` set for a new family:
//! a verdict the existing reading does not reach.
//!
//! The contract answers that question and no wider one. Go's type information is far larger --
//! every expression, every method set -- and a contract shaped around all of it would be a family no
//! rule reads, which is what the composition root calls wiring something nobody reads. A second
//! question about Go types is a second capability when a rule asks it.
//!
//! No provider and no rule here: `nomos-lang-go-types` offers against this contract and
//! `nomos-rules` reads it, and neither names the other.

#![forbid(unsafe_code)]

mod contract;
mod payload;

pub use contract::{Capability, Capability_Contract, Ceiling, Payload_Schema, CAPABILITY, CONTRACT_VERSION, SCHEMA};
pub use payload::{DiscardedValue, DiscardedValuesPayload, Encode_Payload, Parse_Payload, PayloadRefusal};
