//! The Rust facade-surface family from code-standards.
//!
//! Three published rule documents, one tool. code-standards' own `check-facade-surface`
//! enforces `facade-chooses-flattening-or-namespace`, `facade-aliases-name-the-contract`
//! and `facade-consumers-use-the-facade-path` from one binary because all three read the
//! same two statement shapes out of Rust source text -- `pub mod <child>;` and
//! `pub use <child>::<item>;` -- and disagree only about what they conclude from them.
//! They are split into three functions here for the same reason the atomic-ordering family
//! is: this crate's unit of export is the rule, not the tool that happened to ship it.
//!
//! # Why this is not a capability
//!
//! `OD-RULES-011` routes a rule's *parameters* through a repository's own declaration, and
//! none of these three has one. There is no threshold, no vocabulary and no convention a
//! repository would plausibly state differently -- a facade either publishes a child twice
//! or it does not. So this is a leaf module, the same answer the marker-comment family
//! reached, and not a fifth `nomos.cap.*.policy` contract.
//!
//! # What each rule decides
//!
//! [`Check_A_Facade_Publishes_A_Child_One_Way`] reports a file that both declares
//! `pub mod child;` and re-exports through `pub use child::...`, which makes
//! `parent::child::Thing` and `parent::Thing` both public and leaves callers to divide
//! between them. Restricted visibility counts: the rule document says so explicitly, and a
//! `pub(crate)` facade is still a boundary.
//!
//! [`Check_A_Renamed_Facade_Re_Export_Names_The_Contract`] reports
//! `pub use child::Internal as Public;` carrying no `facade-alias: allow` reason. The
//! alias is a public name, so the rule asks for the contract it states rather than
//! forbidding it. The marker is read the way `concurrency_text` and `error_text` already
//! read theirs -- leading a `//` comment on the statement's own line, or on a contiguous
//! run of comment, attribute and blank lines immediately above it -- rather than by
//! reimplementing code-standards' own marker package a third time.
//!
//! [`Check_A_Consumer_Imports_Through_The_Facade`] is the only rule here that reads more
//! than one file: it first collects every `pub use <child>::<item>;` a source under a
//! `src` directory publishes, deriving that source's own module path from its location,
//! and then reports any plain `use` statement that names the child path the facade was
//! supposed to hide.
//!
//! # Two deliberate narrowings, both matching this crate rather than the Go tool
//!
//! Every judgment is line-local over comment-stripped text, so a `use` statement broken
//! across lines is not decided. code-standards' own regex nominally spans lines, but its
//! braced comparisons are written against a single-line spelling and do not survive the
//! newline either, so this is a narrowing in form more than in effect -- and it is the
//! convention every other text rule in this crate already follows.
//!
//! A path segment list of exactly two (`child::item`) is what makes a re-export a facade
//! export, exactly as the Go implementation requires. A braced re-export
//! (`pub use child::{One, Two};`) publishes a surface this port does not model, and is
//! left undecided rather than guessed at.
//!
//! # Measured against this workspace before it was written
//!
//! Zero double-publications; twelve consumer findings over ten import lines reaching around
//! a crate-root facade, two of those lines bypassing two facades at once; and fifty-nine
//! unexplained aliases -- the last because `pub use id::Id as EntityId;` is this
//! workspace's own settled idiom for a one-type module. That is a real disagreement
//! between two standards and not a defect in either, which is why this item lands the
//! rules and leaves composition into a real run to a later increment that can weigh it.
//!
//! One import line can be reported more than once on purpose: a finding names the facade it
//! reached around, so two facades publishing the same child and item are two different
//! things to say about one line rather than one thing said twice. code-standards' own tool
//! reports them the same way.
//!
//! None of the three can match this file. It re-exports from private sibling modules
//! declared with a bare `mod`, so it publishes no child namespace alongside them, and the
//! lines that do it rename nothing; and a `pub use` is never read as a consumer import. The
//! self-exemption `rust_text`, `security_text`, `concurrency_text` and `error_text` each
//! carry is therefore absent here on purpose rather than by oversight.
//!
//! # Split by responsibility
//!
//! [`double_publication`], [`renamed_alias`] and [`consumer_import`] each hold one rule's
//! reading and judgment; [`text`] holds the two statement shapes all three are written over,
//! which none of them owns. This file keeps only what the three share: the rule ids, the
//! facade export both cross-file rules describe, and the finding constructor.

use super::code_prefix::Code_Prefix;
use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

mod consumer_import;
mod double_publication;
mod renamed_alias;
mod text;

pub use consumer_import::Check_A_Consumer_Imports_Through_The_Facade;
pub use double_publication::Check_A_Facade_Publishes_A_Child_One_Way;
pub use renamed_alias::Check_A_Renamed_Facade_Re_Export_Names_The_Contract;

/// The code-standards double-publication rule id.
pub const FACADE_CHOOSES_FLATTENING_OR_NAMESPACE: &str = "facade-chooses-flattening-or-namespace";
/// The code-standards renamed-re-export rule id.
pub const FACADE_ALIASES_NAME_THE_CONTRACT: &str = "facade-aliases-name-the-contract";
/// The code-standards consumer-import rule id.
pub const FACADE_CONSUMERS_USE_THE_FACADE_PATH: &str = "facade-consumers-use-the-facade-path";

/// The literal marker a justified facade alias must lead its reason comment with.
const FACADE_ALIAS_MARKER: &str = "facade-alias: allow";

/// The directory whose contents are a crate's module tree.
const SOURCE_DIRECTORY: &str = "src";

/// A `pub use <child>::<item>;` a facade publishes, with both paths the rule compares.
struct FacadeExport
{
    /// The publishing module's own crate-relative path, `::`-joined; empty at a crate root.
    facade: String,
    /// The child module the item really lives in.
    child: String,
    /// The item's own name.
    item: String,
    /// The path callers are meant to use.
    canonical: String,
    /// The path the facade was hiding.
    bypass: String,
}

/// Every line of `text` with any `//` comment removed, so a commented-out declaration is
/// never read as a real one. The line count is preserved, so an index is still a line.
fn Code_Lines(text: &str) -> Vec<String>
{
    return text.lines().map(Code_Prefix).collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Finding_At(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
mod tests;
