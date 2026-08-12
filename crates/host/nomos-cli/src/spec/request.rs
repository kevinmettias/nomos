//! What each `nomos spec` verb was asked to do, parsed and nothing more.
//!
//! One type per verb, and none of them runs anything. The parse and the act are separate so
//! that a usage error is decided before any store is opened -- a verb that discovered its
//! own arguments were wrong halfway through would already have done something.
//!
//! Grouped rather than flat because these six were the largest single thing at this level
//! and they are all the same thing: the argument shape of one subcommand.

mod commit;
mod edit;
mod freshness;
mod record;
mod render;
mod table;

pub(crate) use commit::CommitRequest;
pub(crate) use edit::EditRequest;
pub(crate) use freshness::FreshnessRequest;
pub(crate) use record::RecordRequest;
pub(crate) use render::RenderRequest;
pub(crate) use table::TableRequest;
