//! What each `nomos spec` verb was asked to do, parsed and nothing more.
//!
//! One type per verb, and none of them runs anything. The parse and the act are separate so
//! that a usage error is decided before any store is opened -- a verb that discovered its
//! own arguments were wrong halfway through would already have done something.
//!
//! Grouped rather than flat because these six were the largest single thing at this level
//! and they are all the same thing: the argument shape of one subcommand.
//!
//! Moved verbatim from `nomos-cli::spec::request`, alongside [`crate::command::SpecCommand`]
//! that carries them. [`submit::SubmitRequest`] is the exception: it moved from
//! `nomos-cli::request` rather than `nomos-cli::spec::request`, and [`crate::run::Submit`]
//! that carries it is not a [`crate::command::SpecCommand`] variant -- `OD-HOST-005`'s own
//! resolution says a `nomos request submit` invocation is not a `nomos spec` verb by the
//! CLI's own naming, so it stays a sibling verb this crate answers rather than a tenth case
//! of [`SpecCommand`]'s own enum.

mod commit;
mod edit;
mod freshness;
mod record;
mod render;
mod submit;
mod table;

pub use commit::CommitRequest;
pub use edit::EditRequest;
pub use freshness::FreshnessRequest;
pub use record::RecordRequest;
pub use render::RenderRequest;
pub use submit::SubmitRequest;
pub use table::TableRequest;
